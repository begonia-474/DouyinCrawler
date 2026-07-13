//! Rust 原生直播录制器。
//!
//! 录制行为对齐 f2：选择 Python 解析出的 FULL_HD1 HLS 地址，循环刷新
//! M3U8，按分片 URL 去重并顺序追加到 `_live.flv` 文件，直到直播结束
//! 或收到停止信号。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use log::warn;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE, REFERER};
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

const MAX_SEGMENT_COUNT: usize = 1000;

#[derive(Debug, Clone, Deserialize)]
pub struct ResolvedLive {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub web_rid: String,
    #[serde(default)]
    pub room_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub sec_user_id: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub m3u8_url: String,
    #[serde(default)]
    pub save_dir: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub suffix: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl ResolvedLive {
    pub fn full_path(&self) -> PathBuf {
        PathBuf::from(&self.save_dir).join(format!("{}{}", self.filename, self.suffix))
    }
}

#[derive(Debug, Clone)]
pub struct LiveRecordOutput {
    pub path: PathBuf,
    pub file_size: u64,
    pub started_at: i64,
    pub ended_at: i64,
    pub stopped: bool,
    pub skipped: bool,
}

impl LiveRecordOutput {
    pub fn duration_sec(&self) -> i64 {
        self.ended_at.saturating_sub(self.started_at)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LiveRecordError {
    #[error("直播流地址无效: {0}")]
    InvalidUrl(String),
    #[error("直播播放列表解析失败: {0}")]
    InvalidPlaylist(String),
    #[error("直播网络请求失败: {0}")]
    Network(#[from] reqwest::Error),
    #[error("直播文件写入失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("直播录制已取消")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveSegment {
    url: String,
    duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Playlist {
    Master(String),
    Media(Vec<LiveSegment>),
}

enum PlaylistLoad {
    Segments(Vec<LiveSegment>),
    Ended,
}

enum SegmentDownload {
    Completed(u64),
    Ended,
    Stopped,
}

pub struct LiveRecorder {
    client: Client,
    cancel_signal: Arc<AtomicBool>,
}

impl LiveRecorder {
    pub fn new(
        config: &crate::config::AppConfig,
        cancel_signal: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout as u64))
            .pool_max_idle_per_host(config.max_connections as usize);

        if !config.proxy.trim().is_empty() {
            let proxy =
                reqwest::Proxy::all(&config.proxy).map_err(|e| format!("直播代理配置无效: {e}"))?;
            builder = builder.proxy(proxy);
        }

        let client = builder
            .build()
            .map_err(|e| format!("创建直播 HTTP 客户端失败: {e}"))?;
        Ok(Self {
            client,
            cancel_signal,
        })
    }

    pub async fn record(
        &self,
        live: &ResolvedLive,
        progress: impl Fn(u64) + Send + Sync,
    ) -> Result<LiveRecordOutput, LiveRecordError> {
        if self.cancel_signal.load(Ordering::Relaxed) {
            return Err(LiveRecordError::Cancelled);
        }

        let playlist_url =
            Url::parse(&live.m3u8_url).map_err(|e| LiveRecordError::InvalidUrl(e.to_string()))?;
        let full_path = live.full_path();
        let started_at = unix_timestamp();

        if full_path.exists() {
            let file_size = tokio::fs::metadata(&full_path).await?.len();
            return Ok(LiveRecordOutput {
                path: full_path,
                file_size,
                started_at,
                ended_at: unix_timestamp(),
                stopped: false,
                skipped: true,
            });
        }

        let headers = build_headers(live)?;
        let mut downloaded = HashSet::new();
        let mut total_downloaded = 0_u64;
        let mut stopped = false;

        'recording: loop {
            if self.cancel_signal.load(Ordering::Relaxed) {
                stopped = true;
                break;
            }

            let segments = match self.load_playlist(playlist_url.clone(), &headers).await? {
                PlaylistLoad::Segments(segments) if !segments.is_empty() => segments,
                PlaylistLoad::Segments(_) | PlaylistLoad::Ended => break,
            };

            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&full_path)
                .await?;
            let mut sleep_duration = Duration::from_secs(1);
            let mut should_end = false;

            for segment in segments {
                sleep_duration = segment.duration;
                if self.cancel_signal.load(Ordering::Relaxed) {
                    stopped = true;
                    should_end = true;
                    break;
                }
                if downloaded.contains(&segment.url) {
                    continue;
                }

                match self
                    .download_segment(&segment.url, &headers, &mut file)
                    .await
                {
                    Ok(SegmentDownload::Completed(bytes)) => {
                        total_downloaded += bytes;
                        progress(total_downloaded);
                        downloaded.insert(segment.url.clone());
                    }
                    Ok(SegmentDownload::Ended) => {
                        should_end = true;
                        break;
                    }
                    Ok(SegmentDownload::Stopped) => {
                        stopped = true;
                        should_end = true;
                        break;
                    }
                    Err(error) => {
                        // 对齐 f2：单个 TS 分片失败时跳过，下一轮播放列表会再次尝试。
                        warn!("[LiveRecorder] 分片下载失败，稍后重试: {}", error);
                    }
                }

                // f2 在集合超过上限时整体清空，避免长时间录制持续占用内存。
                if downloaded.len() > MAX_SEGMENT_COUNT {
                    downloaded.clear();
                }
            }

            file.flush().await?;
            if should_end {
                break 'recording;
            }
            self.sleep_or_stop(sleep_duration).await;
        }

        let file_size = match tokio::fs::metadata(&full_path).await {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(error.into()),
        };

        Ok(LiveRecordOutput {
            path: full_path,
            file_size,
            started_at,
            ended_at: unix_timestamp(),
            stopped,
            skipped: false,
        })
    }

    async fn load_playlist(
        &self,
        initial_url: Url,
        headers: &HeaderMap,
    ) -> Result<PlaylistLoad, LiveRecordError> {
        let mut current_url = initial_url;
        for _ in 0..4 {
            let response = self
                .client
                .get(current_url.clone())
                .headers(headers.clone())
                .send()
                .await?;
            if matches!(
                response.status(),
                StatusCode::NOT_FOUND | StatusCode::GATEWAY_TIMEOUT
            ) {
                return Ok(PlaylistLoad::Ended);
            }
            let response = response.error_for_status()?;
            let content = response.text().await?;
            match parse_playlist(&current_url, &content)? {
                Playlist::Master(url) => {
                    current_url =
                        Url::parse(&url).map_err(|e| LiveRecordError::InvalidUrl(e.to_string()))?;
                }
                Playlist::Media(segments) => return Ok(PlaylistLoad::Segments(segments)),
            }
        }
        Err(LiveRecordError::InvalidPlaylist(
            "嵌套播放列表超过最大层级".to_string(),
        ))
    }

    async fn download_segment(
        &self,
        url: &str,
        headers: &HeaderMap,
        file: &mut File,
    ) -> Result<SegmentDownload, LiveRecordError> {
        let response = self.client.get(url).headers(headers.clone()).send().await?;
        if matches!(
            response.status(),
            StatusCode::NOT_FOUND | StatusCode::GATEWAY_TIMEOUT
        ) {
            return Ok(SegmentDownload::Ended);
        }
        let response = response.error_for_status()?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0_u64;

        while let Some(chunk) = stream.next().await {
            if self.cancel_signal.load(Ordering::Relaxed) {
                return Ok(SegmentDownload::Stopped);
            }
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
        }

        Ok(SegmentDownload::Completed(downloaded))
    }

    async fn sleep_or_stop(&self, duration: Duration) {
        let deadline = tokio::time::Instant::now() + duration;
        while tokio::time::Instant::now() < deadline {
            if self.cancel_signal.load(Ordering::Relaxed) {
                return;
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            tokio::time::sleep(remaining.min(Duration::from_millis(200))).await;
        }
    }
}

fn build_headers(live: &ResolvedLive) -> Result<HeaderMap, LiveRecordError> {
    let mut headers = HeaderMap::new();
    for (key, value) in &live.headers {
        let name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|e| LiveRecordError::InvalidPlaylist(format!("请求头名称无效: {e}")))?;
        let value = HeaderValue::from_str(value)
            .map_err(|e| LiveRecordError::InvalidPlaylist(format!("请求头值无效: {e}")))?;
        headers.insert(name, value);
    }
    if !headers.contains_key(REFERER) {
        headers.insert(REFERER, HeaderValue::from_static("https://www.douyin.com/"));
    }
    if !headers.contains_key(COOKIE) {
        headers.insert(COOKIE, HeaderValue::from_static(""));
    }
    Ok(headers)
}

fn parse_playlist(base_url: &Url, content: &str) -> Result<Playlist, LiveRecordError> {
    let lines: Vec<&str> = content.lines().map(str::trim).collect();

    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("#EXT-X-STREAM-INF") {
            let nested = lines[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_empty() && !candidate.starts_with('#'))
                .ok_or_else(|| {
                    LiveRecordError::InvalidPlaylist("主播放列表缺少子播放列表地址".to_string())
                })?;
            let url = base_url.join(nested).map_err(|e| {
                LiveRecordError::InvalidPlaylist(format!("子播放列表地址无效: {e}"))
            })?;
            return Ok(Playlist::Master(url.to_string()));
        }
    }

    let mut segments = Vec::new();
    let mut duration = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("#EXTINF:") {
            let seconds = value
                .split(',')
                .next()
                .unwrap_or("0")
                .parse::<f64>()
                .map_err(|e| LiveRecordError::InvalidPlaylist(format!("分片时长无效: {e}")))?;
            duration = Some(Duration::from_secs_f64(seconds.max(0.0)));
            continue;
        }

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(segment_duration) = duration.take() {
            let url = base_url
                .join(line)
                .map_err(|e| LiveRecordError::InvalidPlaylist(format!("分片地址无效: {e}")))?;
            segments.push(LiveSegment {
                url: url.to_string(),
                duration: segment_duration,
            });
        }
    }

    Ok(Playlist::Media(segments))
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn parses_media_playlist_with_absolute_segment_urls() {
        let base = Url::parse("https://example.com/live/index.m3u8").unwrap();
        let content = "#EXTM3U\n#EXT-X-TARGETDURATION:2\n#EXTINF:1.5,\nsegment-1.ts\n#EXTINF:2.0,\n../segment-2.ts\n";

        let playlist = parse_playlist(&base, content).unwrap();
        let Playlist::Media(segments) = playlist else {
            panic!("expected media playlist");
        };

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].url, "https://example.com/live/segment-1.ts");
        assert_eq!(segments[0].duration.as_millis(), 1500);
        assert_eq!(segments[1].url, "https://example.com/segment-2.ts");
        assert_eq!(segments[1].duration.as_secs(), 2);
    }

    #[test]
    fn selects_first_nested_playlist_like_f2() {
        let base = Url::parse("https://example.com/master.m3u8").unwrap();
        let content = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nlow/index.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=2000\nhigh/index.m3u8\n";

        let playlist = parse_playlist(&base, content).unwrap();
        assert_eq!(
            playlist,
            Playlist::Master("https://example.com/low/index.m3u8".to_string())
        );
    }

    #[tokio::test]
    async fn records_hls_segments_until_playlist_is_empty() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let playlist_requests = Arc::new(AtomicUsize::new(0));
        let server_counter = playlist_requests.clone();

        let server = tokio::spawn(async move {
            for _ in 0..4 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = vec![0_u8; 2048];
                let read = socket.read(&mut request).await.unwrap();
                let request = String::from_utf8_lossy(&request[..read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");

                let body: Vec<u8> = match path {
                    "/live.m3u8" => {
                        if server_counter.fetch_add(1, Ordering::Relaxed) == 0 {
                            b"#EXTM3U\n#EXTINF:0.01,\nsegment-1.ts\n#EXTINF:0.01,\nsegment-2.ts\n"
                                .to_vec()
                        } else {
                            b"#EXTM3U\n".to_vec()
                        }
                    }
                    "/segment-1.ts" => b"AAA".to_vec(),
                    "/segment-2.ts" => b"BBB".to_vec(),
                    _ => Vec::new(),
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
                socket.write_all(&body).await.unwrap();
            }
        });

        let temp_dir =
            std::env::temp_dir().join(format!("douyin-live-test-{}", uuid::Uuid::new_v4()));
        let config = crate::config::AppConfig {
            timeout: 2,
            ..crate::config::AppConfig::default()
        };
        let recorder = LiveRecorder::new(&config, Arc::new(AtomicBool::new(false))).unwrap();
        let live = ResolvedLive {
            success: true,
            error: None,
            web_rid: "web-1".to_string(),
            room_id: "room-1".to_string(),
            title: "test".to_string(),
            nickname: "anchor".to_string(),
            sec_user_id: String::new(),
            cover_url: String::new(),
            m3u8_url: format!("http://{address}/live.m3u8"),
            save_dir: temp_dir.to_string_lossy().to_string(),
            filename: "record_live".to_string(),
            suffix: ".flv".to_string(),
            headers: HashMap::new(),
        };

        let output = recorder.record(&live, |_| {}).await.unwrap();
        server.await.unwrap();

        assert_eq!(output.file_size, 6);
        assert_eq!(tokio::fs::read(&output.path).await.unwrap(), b"AAABBB");
        assert!(!output.stopped);

        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }

    #[tokio::test]
    async fn stop_signal_keeps_already_recorded_segments() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for _ in 0..2 {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = vec![0_u8; 2048];
                let read = socket.read(&mut request).await.unwrap();
                let request = String::from_utf8_lossy(&request[..read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");
                let body: Vec<u8> = match path {
                    "/live.m3u8" => {
                        b"#EXTM3U\n#EXTINF:1,\nsegment-1.ts\n#EXTINF:1,\nsegment-2.ts\n".to_vec()
                    }
                    "/segment-1.ts" => b"AAA".to_vec(),
                    _ => Vec::new(),
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
                socket.write_all(&body).await.unwrap();
            }
        });

        let temp_dir =
            std::env::temp_dir().join(format!("douyin-live-stop-test-{}", uuid::Uuid::new_v4()));
        let config = crate::config::AppConfig {
            timeout: 2,
            ..crate::config::AppConfig::default()
        };
        let stop_signal = Arc::new(AtomicBool::new(false));
        let recorder = LiveRecorder::new(&config, stop_signal.clone()).unwrap();
        let live = ResolvedLive {
            success: true,
            error: None,
            web_rid: "web-1".to_string(),
            room_id: "room-1".to_string(),
            title: "test".to_string(),
            nickname: "anchor".to_string(),
            sec_user_id: String::new(),
            cover_url: String::new(),
            m3u8_url: format!("http://{address}/live.m3u8"),
            save_dir: temp_dir.to_string_lossy().to_string(),
            filename: "record_live".to_string(),
            suffix: ".flv".to_string(),
            headers: HashMap::new(),
        };
        let progress_signal = stop_signal.clone();

        let output = recorder
            .record(&live, move |_| {
                progress_signal.store(true, Ordering::Relaxed);
            })
            .await
            .unwrap();
        server.await.unwrap();

        assert!(output.stopped);
        assert_eq!(output.file_size, 3);
        assert_eq!(tokio::fs::read(&output.path).await.unwrap(), b"AAA");

        let _ = tokio::fs::remove_dir_all(temp_dir).await;
    }
}
