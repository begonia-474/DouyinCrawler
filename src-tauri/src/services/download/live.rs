//! Rust-owned HLS/FLV live recorder.
//!
//! Python owns room resolution, f2 naming and the FULL_HD1 choice. This module
//! validates that typed plan, appends deduplicated HLS segments in order, and
//! returns a typed outcome only after the output file has been flushed/closed.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE, REFERER};
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use serde_json::Value;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

const LIVE_CONTRACT_VERSION: u32 = 1;
const MAX_PLAYLIST_DEPTH: usize = 4;
const MAX_SEGMENT_COUNT: usize = 1000;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveOutputV1 {
    pub save_dir: String,
    pub filename: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LivePlanV1 {
    pub success: bool,
    pub contract_version: u32,
    pub mode: String,
    pub web_rid: String,
    pub room_id: String,
    pub title: String,
    pub nickname: String,
    pub sec_user_id: String,
    pub user_id: Option<String>,
    pub cover_url: String,
    pub user_count: i64,
    pub m3u8_url: String,
    pub output: LiveOutputV1,
    pub headers: HashMap<String, String>,
}

impl LivePlanV1 {
    pub fn from_value(value: Value) -> Result<Self, String> {
        let plan: Self = serde_json::from_value(value)
            .map_err(|error| format!("live V1 contract 解析失败: {error}"))?;
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.success {
            return Err("live V1 contract success 必须为 true".to_string());
        }
        if self.contract_version != LIVE_CONTRACT_VERSION {
            return Err(format!(
                "不支持的 live contract_version: {}",
                self.contract_version
            ));
        }
        if self.mode != "live" {
            return Err(format!("live contract mode 无效: {}", self.mode));
        }
        for (name, value) in [
            ("web_rid", self.web_rid.as_str()),
            ("room_id", self.room_id.as_str()),
            ("nickname", self.nickname.as_str()),
            ("sec_user_id", self.sec_user_id.as_str()),
            ("m3u8_url", self.m3u8_url.as_str()),
            ("output.save_dir", self.output.save_dir.as_str()),
            ("output.filename", self.output.filename.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("live contract {name} 不能为空"));
            }
        }
        if self.user_count < 0 {
            return Err("live contract user_count 不能为负数".to_string());
        }
        if self
            .user_id
            .as_deref()
            .is_some_and(|user_id| user_id.trim().is_empty())
        {
            return Err("live contract user_id 提供时不能为空".to_string());
        }
        let url =
            Url::parse(&self.m3u8_url).map_err(|error| format!("FULL_HD1 URL 无效: {error}"))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err("FULL_HD1 URL 仅支持 http/https".to_string());
        }
        if self.output.suffix != ".flv" {
            return Err("live output suffix 必须为 .flv".to_string());
        }
        if self.output.filename.contains(['/', '\\', '\0'])
            || matches!(self.output.filename.as_str(), "." | "..")
            || Path::new(&self.output.filename).components().count() != 1
            || !matches!(
                Path::new(&self.output.filename).components().next(),
                Some(Component::Normal(_))
            )
        {
            return Err("live output filename 必须是有效的单一路径组件".to_string());
        }
        build_headers(self).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn full_path(&self) -> PathBuf {
        PathBuf::from(&self.output.save_dir)
            .join(format!("{}{}", self.output.filename, self.output.suffix))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveCompletionReason {
    StreamEnded,
    UserStopped,
    ExistingOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveFailureKind {
    InvalidPlan,
    InvalidPlaylist,
    NetworkRetryExhausted,
    SegmentRetryExhausted,
    FileCreate,
    FileWrite,
    FileFlush,
    FileMetadata,
    StoppedBeforeUsableBytes,
}

#[derive(Debug, Clone)]
pub struct LiveOutputFacts {
    pub path: PathBuf,
    pub file_size: u64,
    pub started_at: i64,
    pub ended_at: i64,
}

impl LiveOutputFacts {
    pub fn duration_sec(&self) -> i64 {
        self.ended_at.saturating_sub(self.started_at)
    }
}

#[derive(Debug, Clone)]
pub enum LiveRecorderOutcome {
    Completed {
        reason: LiveCompletionReason,
        output: LiveOutputFacts,
    },
    Failed {
        kind: LiveFailureKind,
        error: String,
        output: LiveOutputFacts,
    },
}

impl LiveRecorderOutcome {
    pub fn output(&self) -> &LiveOutputFacts {
        match self {
            Self::Completed { output, .. } | Self::Failed { output, .. } => output,
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum RecorderError {
    #[error("直播计划无效: {0}")]
    InvalidPlan(String),
    #[error("直播播放列表无效: {0}")]
    InvalidPlaylist(String),
    #[error("直播播放列表请求重试耗尽: {0}")]
    NetworkRetryExhausted(String),
    #[error("直播分片请求重试耗尽: {0}")]
    SegmentRetryExhausted(String),
    #[error("创建直播文件失败: {0}")]
    FileCreate(String),
    #[error("写入直播文件失败: {0}")]
    FileWrite(String),
    #[error("刷新直播文件失败: {0}")]
    FileFlush(String),
    #[error("读取直播文件信息失败: {0}")]
    FileMetadata(String),
}

impl RecorderError {
    fn kind(&self) -> LiveFailureKind {
        match self {
            Self::InvalidPlan(_) => LiveFailureKind::InvalidPlan,
            Self::InvalidPlaylist(_) => LiveFailureKind::InvalidPlaylist,
            Self::NetworkRetryExhausted(_) => LiveFailureKind::NetworkRetryExhausted,
            Self::SegmentRetryExhausted(_) => LiveFailureKind::SegmentRetryExhausted,
            Self::FileCreate(_) => LiveFailureKind::FileCreate,
            Self::FileWrite(_) => LiveFailureKind::FileWrite,
            Self::FileFlush(_) => LiveFailureKind::FileFlush,
            Self::FileMetadata(_) => LiveFailureKind::FileMetadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiveSegment {
    url: String,
    duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Playlist {
    Master(String),
    Media {
        segments: Vec<LiveSegment>,
        ended: bool,
    },
}

enum SegmentFetch {
    Bytes(Vec<u8>),
    Stopped,
}

pub struct LiveRecorder {
    client: Client,
    cancel_signal: Arc<AtomicBool>,
    max_attempts: u32,
    retry_delay: Duration,
    #[cfg(test)]
    test_writer_failure: Option<TestWriterFailure>,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestWriterFailure {
    Write,
    Flush,
}

impl LiveRecorder {
    pub fn new(
        config: &crate::config::AppConfig,
        cancel_signal: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout.max(1) as u64))
            .pool_max_idle_per_host(config.max_connections.max(1) as usize);
        if !config.proxy.trim().is_empty() {
            let proxy =
                reqwest::Proxy::all(&config.proxy).map_err(|e| format!("直播代理配置无效: {e}"))?;
            builder = builder.proxy(proxy);
        }
        Ok(Self {
            client: builder
                .build()
                .map_err(|e| format!("创建直播 HTTP 客户端失败: {e}"))?,
            cancel_signal,
            max_attempts: config.max_retries.max(1),
            retry_delay: Duration::from_millis(200),
            #[cfg(test)]
            test_writer_failure: None,
        })
    }

    #[cfg(test)]
    fn with_test_writer_failure(mut self, failure: TestWriterFailure) -> Self {
        self.test_writer_failure = Some(failure);
        self
    }

    pub async fn record(
        &self,
        plan: &LivePlanV1,
        progress: impl Fn(u64) + Send + Sync,
    ) -> LiveRecorderOutcome {
        let started_at = unix_timestamp();
        let path = plan.full_path();
        if let Err(error) = plan.validate() {
            return self
                .failure(RecorderError::InvalidPlan(error), path, started_at, None)
                .await;
        }

        match tokio::fs::metadata(&path).await {
            Ok(metadata) if metadata.len() > 0 => {
                return LiveRecorderOutcome::Completed {
                    reason: LiveCompletionReason::ExistingOutput,
                    output: LiveOutputFacts {
                        path,
                        file_size: metadata.len(),
                        started_at,
                        ended_at: unix_timestamp(),
                    },
                };
            }
            Ok(_) => {
                return self
                    .failure(
                        RecorderError::InvalidPlan("目标文件已存在但为空".to_string()),
                        path,
                        started_at,
                        None,
                    )
                    .await;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return self
                    .failure(
                        RecorderError::FileMetadata(error.to_string()),
                        path,
                        started_at,
                        None,
                    )
                    .await;
            }
        }

        let headers = match build_headers(plan) {
            Ok(headers) => headers,
            Err(error) => {
                return self.failure(error, path, started_at, None).await;
            }
        };
        let playlist_url = match Url::parse(&plan.m3u8_url) {
            Ok(url) => url,
            Err(error) => {
                return self
                    .failure(
                        RecorderError::InvalidPlan(error.to_string()),
                        path,
                        started_at,
                        None,
                    )
                    .await;
            }
        };

        let mut file: Option<File> = None;
        let mut downloaded = HashSet::new();
        let mut total_downloaded = 0_u64;

        loop {
            if self.cancel_signal.load(Ordering::Relaxed) {
                return self.stopped(path, started_at, file, total_downloaded).await;
            }

            let playlist = match self.load_playlist(playlist_url.clone(), &headers).await {
                Ok(playlist) => playlist,
                Err(error) => return self.failure(error, path, started_at, file).await,
            };
            let Playlist::Media { segments, ended } = playlist else {
                unreachable!("load_playlist resolves nested master playlists")
            };

            if segments.is_empty() {
                if ended && total_downloaded > 0 {
                    return self
                        .completed(LiveCompletionReason::StreamEnded, path, started_at, file)
                        .await;
                }
                return self
                    .failure(
                        RecorderError::InvalidPlaylist(if ended {
                            "ENDLIST 前没有可用分片".to_string()
                        } else {
                            "播放列表没有分片且缺少 #EXT-X-ENDLIST".to_string()
                        }),
                        path,
                        started_at,
                        file,
                    )
                    .await;
            }

            let mut sleep_duration = Duration::from_secs(1);
            for segment in segments {
                sleep_duration = segment.duration;
                if downloaded.contains(&segment.url) {
                    continue;
                }
                if self.cancel_signal.load(Ordering::Relaxed) {
                    return self.stopped(path, started_at, file, total_downloaded).await;
                }
                let bytes = match self.fetch_segment(&segment.url, &headers).await {
                    Ok(SegmentFetch::Bytes(bytes)) => bytes,
                    Ok(SegmentFetch::Stopped) => {
                        return self.stopped(path, started_at, file, total_downloaded).await;
                    }
                    Err(error) => return self.failure(error, path, started_at, file).await,
                };
                if file.is_none() {
                    if let Some(parent) = path.parent() {
                        if let Err(error) = tokio::fs::create_dir_all(parent).await {
                            return self
                                .failure(
                                    RecorderError::FileCreate(error.to_string()),
                                    path,
                                    started_at,
                                    file,
                                )
                                .await;
                        }
                    }
                    match OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(&path)
                        .await
                    {
                        Ok(opened) => file = Some(opened),
                        Err(error) => {
                            return self
                                .failure(
                                    RecorderError::FileCreate(error.to_string()),
                                    path,
                                    started_at,
                                    file,
                                )
                                .await;
                        }
                    }
                }
                #[cfg(test)]
                if self.test_writer_failure == Some(TestWriterFailure::Write) {
                    return self
                        .failure(
                            RecorderError::FileWrite("injected write failure".to_string()),
                            path,
                            started_at,
                            file,
                        )
                        .await;
                }
                if let Err(error) = file.as_mut().unwrap().write_all(&bytes).await {
                    return self
                        .failure(
                            RecorderError::FileWrite(error.to_string()),
                            path,
                            started_at,
                            file,
                        )
                        .await;
                }
                total_downloaded += bytes.len() as u64;
                progress(total_downloaded);
                downloaded.insert(segment.url);
                if downloaded.len() > MAX_SEGMENT_COUNT {
                    downloaded.clear();
                }
            }

            #[cfg(test)]
            if file.is_some() && self.test_writer_failure == Some(TestWriterFailure::Flush) {
                return self
                    .failure(
                        RecorderError::FileFlush("injected flush failure".to_string()),
                        path,
                        started_at,
                        file,
                    )
                    .await;
            }
            if let Some(opened) = file.as_mut() {
                if let Err(error) = opened.flush().await {
                    return self
                        .failure(
                            RecorderError::FileFlush(error.to_string()),
                            path,
                            started_at,
                            file,
                        )
                        .await;
                }
            }
            if ended {
                return self
                    .completed(LiveCompletionReason::StreamEnded, path, started_at, file)
                    .await;
            }
            self.sleep_or_stop(sleep_duration).await;
        }
    }

    async fn completed(
        &self,
        reason: LiveCompletionReason,
        path: PathBuf,
        started_at: i64,
        file: Option<File>,
    ) -> LiveRecorderOutcome {
        match self.close_and_measure(path.clone(), started_at, file).await {
            Ok(output) if output.file_size > 0 => LiveRecorderOutcome::Completed { reason, output },
            Ok(output) => LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::InvalidPlaylist,
                error: "直播结束但没有产生可用字节".to_string(),
                output,
            },
            Err((error, output)) => LiveRecorderOutcome::Failed {
                kind: error.kind(),
                error: error.to_string(),
                output,
            },
        }
    }

    async fn stopped(
        &self,
        path: PathBuf,
        started_at: i64,
        file: Option<File>,
        _written: u64,
    ) -> LiveRecorderOutcome {
        match self.close_and_measure(path, started_at, file).await {
            Ok(output) if output.file_size > 0 => LiveRecorderOutcome::Completed {
                reason: LiveCompletionReason::UserStopped,
                output,
            },
            Ok(output) => LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::StoppedBeforeUsableBytes,
                error: "用户停止录制时尚未产生可用字节".to_string(),
                output,
            },
            Err((error, output)) => LiveRecorderOutcome::Failed {
                kind: error.kind(),
                error: error.to_string(),
                output,
            },
        }
    }

    async fn failure(
        &self,
        primary: RecorderError,
        path: PathBuf,
        started_at: i64,
        file: Option<File>,
    ) -> LiveRecorderOutcome {
        match self.close_and_measure(path, started_at, file).await {
            Ok(output) => LiveRecorderOutcome::Failed {
                kind: primary.kind(),
                error: primary.to_string(),
                output,
            },
            Err((close_error, output)) => LiveRecorderOutcome::Failed {
                kind: close_error.kind(),
                error: format!("{primary}; {close_error}"),
                output,
            },
        }
    }

    async fn close_and_measure(
        &self,
        path: PathBuf,
        started_at: i64,
        mut file: Option<File>,
    ) -> Result<LiveOutputFacts, (RecorderError, LiveOutputFacts)> {
        if let Some(opened) = file.as_mut() {
            if let Err(error) = opened.flush().await {
                let output = LiveOutputFacts {
                    path,
                    file_size: 0,
                    started_at,
                    ended_at: unix_timestamp(),
                };
                return Err((RecorderError::FileFlush(error.to_string()), output));
            }
        }
        drop(file);
        let ended_at = unix_timestamp();
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => Ok(LiveOutputFacts {
                path,
                file_size: metadata.len(),
                started_at,
                ended_at,
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LiveOutputFacts {
                path,
                file_size: 0,
                started_at,
                ended_at,
            }),
            Err(error) => {
                let output = LiveOutputFacts {
                    path,
                    file_size: 0,
                    started_at,
                    ended_at,
                };
                Err((RecorderError::FileMetadata(error.to_string()), output))
            }
        }
    }

    async fn load_playlist(
        &self,
        initial_url: Url,
        headers: &HeaderMap,
    ) -> Result<Playlist, RecorderError> {
        let mut current_url = initial_url;
        for _ in 0..MAX_PLAYLIST_DEPTH {
            let content = self.fetch_playlist_text(&current_url, headers).await?;
            match parse_playlist(&current_url, &content)? {
                Playlist::Master(url) => {
                    current_url = Url::parse(&url)
                        .map_err(|error| RecorderError::InvalidPlaylist(error.to_string()))?;
                }
                media @ Playlist::Media { .. } => return Ok(media),
            }
        }
        Err(RecorderError::InvalidPlaylist(
            "嵌套播放列表超过最大层级".to_string(),
        ))
    }

    async fn fetch_playlist_text(
        &self,
        url: &Url,
        headers: &HeaderMap,
    ) -> Result<String, RecorderError> {
        let mut last_error = String::new();
        for attempt in 0..self.max_attempts {
            if self.cancel_signal.load(Ordering::Relaxed) {
                return Ok("#EXTM3U\n".to_string());
            }
            match self
                .client
                .get(url.clone())
                .headers(headers.clone())
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => match response.text().await {
                    Ok(content) => return Ok(content),
                    Err(error) => last_error = error.to_string(),
                },
                Ok(response) if is_retryable_status(response.status()) => {
                    last_error = format!("HTTP {}", response.status());
                }
                Ok(response) => {
                    return Err(RecorderError::NetworkRetryExhausted(format!(
                        "HTTP {}",
                        response.status()
                    )));
                }
                Err(error) => last_error = error.to_string(),
            }
            if attempt + 1 < self.max_attempts {
                tokio::time::sleep(self.retry_delay).await;
            }
        }
        Err(RecorderError::NetworkRetryExhausted(last_error))
    }

    async fn fetch_segment(
        &self,
        url: &str,
        headers: &HeaderMap,
    ) -> Result<SegmentFetch, RecorderError> {
        let mut last_error = String::new();
        for attempt in 0..self.max_attempts {
            if self.cancel_signal.load(Ordering::Relaxed) {
                return Ok(SegmentFetch::Stopped);
            }
            match self.client.get(url).headers(headers.clone()).send().await {
                Ok(response) if response.status().is_success() => {
                    let mut bytes = Vec::new();
                    let mut stream_error = None;
                    let mut stream = response.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        if self.cancel_signal.load(Ordering::Relaxed) {
                            return Ok(SegmentFetch::Stopped);
                        }
                        match chunk {
                            Ok(chunk) => bytes.extend_from_slice(&chunk),
                            Err(error) => {
                                stream_error = Some(error.to_string());
                                break;
                            }
                        }
                    }
                    if let Some(error) = stream_error {
                        last_error = error;
                    } else if !bytes.is_empty() {
                        return Ok(SegmentFetch::Bytes(bytes));
                    } else {
                        last_error = "分片响应为空".to_string();
                    }
                }
                Ok(response) if is_retryable_status(response.status()) => {
                    last_error = format!("HTTP {}", response.status());
                }
                Ok(response) => {
                    return Err(RecorderError::SegmentRetryExhausted(format!(
                        "HTTP {}",
                        response.status()
                    )));
                }
                Err(error) => last_error = error.to_string(),
            }
            if attempt + 1 < self.max_attempts {
                tokio::time::sleep(self.retry_delay).await;
            }
        }
        Err(RecorderError::SegmentRetryExhausted(last_error))
    }

    async fn sleep_or_stop(&self, duration: Duration) {
        let deadline = tokio::time::Instant::now() + duration;
        while tokio::time::Instant::now() < deadline {
            if self.cancel_signal.load(Ordering::Relaxed) {
                return;
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            tokio::time::sleep(remaining.min(Duration::from_millis(100))).await;
        }
    }
}

fn is_retryable_status(status: StatusCode) -> bool {
    matches!(status, StatusCode::NOT_FOUND | StatusCode::GATEWAY_TIMEOUT)
        || status.is_server_error()
        || matches!(
            status,
            StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS
        )
}

fn build_headers(plan: &LivePlanV1) -> Result<HeaderMap, RecorderError> {
    let mut headers = HeaderMap::new();
    for (key, value) in &plan.headers {
        let name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|error| RecorderError::InvalidPlan(format!("请求头名称无效: {error}")))?;
        let value = HeaderValue::from_str(value)
            .map_err(|error| RecorderError::InvalidPlan(format!("请求头值无效: {error}")))?;
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

fn parse_playlist(base_url: &Url, content: &str) -> Result<Playlist, RecorderError> {
    let lines: Vec<&str> = content.lines().map(str::trim).collect();
    if !lines.contains(&"#EXTM3U") {
        return Err(RecorderError::InvalidPlaylist(
            "缺少 #EXTM3U 标记".to_string(),
        ));
    }
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with("#EXT-X-STREAM-INF") {
            let nested = lines[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_empty() && !candidate.starts_with('#'))
                .ok_or_else(|| {
                    RecorderError::InvalidPlaylist("主播放列表缺少子播放列表地址".to_string())
                })?;
            return Ok(Playlist::Master(
                base_url
                    .join(nested)
                    .map_err(|error| RecorderError::InvalidPlaylist(error.to_string()))?
                    .to_string(),
            ));
        }
    }

    let ended = lines.contains(&"#EXT-X-ENDLIST");
    let mut segments = Vec::new();
    let mut duration = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("#EXTINF:") {
            let seconds = value
                .split(',')
                .next()
                .unwrap_or("0")
                .parse::<f64>()
                .map_err(|error| RecorderError::InvalidPlaylist(error.to_string()))?;
            duration = Some(Duration::from_secs_f64(seconds.max(0.0)));
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let segment_duration = duration
            .take()
            .ok_or_else(|| RecorderError::InvalidPlaylist("分片地址前缺少 #EXTINF".to_string()))?;
        segments.push(LiveSegment {
            url: base_url
                .join(line)
                .map_err(|error| RecorderError::InvalidPlaylist(error.to_string()))?
                .to_string(),
            duration: segment_duration,
        });
    }
    if duration.is_some() {
        return Err(RecorderError::InvalidPlaylist(
            "#EXTINF 后缺少分片地址".to_string(),
        ));
    }
    Ok(Playlist::Media { segments, ended })
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
    use std::collections::VecDeque;
    use std::sync::atomic::AtomicUsize;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Clone)]
    struct Response {
        status: u16,
        body: Vec<u8>,
        declared_length: Option<usize>,
        chunk_size: Option<usize>,
        chunk_delay: Duration,
    }

    struct ScriptServer {
        address: std::net::SocketAddr,
        request_count: Arc<AtomicUsize>,
        task: tokio::task::JoinHandle<()>,
    }

    impl ScriptServer {
        async fn start(routes: HashMap<String, Vec<Response>>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let total: usize = routes.values().map(Vec::len).sum();
            let routes: HashMap<String, VecDeque<Response>> = routes
                .into_iter()
                .map(|(path, values)| (path, values.into()))
                .collect();
            let routes = Arc::new(tokio::sync::Mutex::new(routes));
            let request_count = Arc::new(AtomicUsize::new(0));
            let server_request_count = request_count.clone();
            let task = tokio::spawn(async move {
                for _ in 0..total {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    server_request_count.fetch_add(1, Ordering::Relaxed);
                    let mut request = vec![0_u8; 4096];
                    let read = socket.read(&mut request).await.unwrap();
                    let path = String::from_utf8_lossy(&request[..read])
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("/")
                        .to_string();
                    let response = routes
                        .lock()
                        .await
                        .get_mut(&path)
                        .and_then(VecDeque::pop_front)
                        .unwrap_or(Response {
                            status: 404,
                            body: Vec::new(),
                            declared_length: None,
                            chunk_size: None,
                            chunk_delay: Duration::ZERO,
                        });
                    let reason = if response.status == 200 {
                        "OK"
                    } else {
                        "Error"
                    };
                    let head = format!(
                        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        response.status,
                        reason,
                        response.declared_length.unwrap_or(response.body.len())
                    );
                    socket.write_all(head.as_bytes()).await.unwrap();
                    if let Some(chunk_size) = response.chunk_size {
                        for chunk in response.body.chunks(chunk_size) {
                            if socket.write_all(chunk).await.is_err() {
                                break;
                            }
                            tokio::time::sleep(response.chunk_delay).await;
                        }
                    } else {
                        let _ = socket.write_all(&response.body).await;
                    }
                }
            });
            Self {
                address,
                request_count,
                task,
            }
        }

        fn url(&self, path: &str) -> String {
            format!("http://{}{}", self.address, path)
        }

        fn request_count(&self) -> usize {
            self.request_count.load(Ordering::Relaxed)
        }
    }

    fn response(status: u16, body: impl Into<Vec<u8>>) -> Response {
        Response {
            status,
            body: body.into(),
            declared_length: None,
            chunk_size: None,
            chunk_delay: Duration::ZERO,
        }
    }

    fn chunked_response(body: impl Into<Vec<u8>>, chunk_size: usize, delay: Duration) -> Response {
        Response {
            status: 200,
            body: body.into(),
            declared_length: None,
            chunk_size: Some(chunk_size),
            chunk_delay: delay,
        }
    }

    fn truncated_response(body: impl Into<Vec<u8>>, declared_length: usize) -> Response {
        Response {
            status: 200,
            body: body.into(),
            declared_length: Some(declared_length),
            chunk_size: None,
            chunk_delay: Duration::ZERO,
        }
    }

    fn plan(url: String, save_dir: PathBuf) -> LivePlanV1 {
        LivePlanV1 {
            success: true,
            contract_version: 1,
            mode: "live".to_string(),
            web_rid: "web-1".to_string(),
            room_id: "room-1".to_string(),
            title: "test".to_string(),
            nickname: "anchor".to_string(),
            sec_user_id: "sec-1".to_string(),
            user_id: Some("uid-1".to_string()),
            cover_url: String::new(),
            user_count: 1,
            m3u8_url: url,
            output: LiveOutputV1 {
                save_dir: save_dir.to_string_lossy().to_string(),
                filename: "record_live".to_string(),
                suffix: ".flv".to_string(),
            },
            headers: HashMap::new(),
        }
    }

    fn recorder(cancel: Arc<AtomicBool>) -> LiveRecorder {
        let config = crate::config::AppConfig {
            timeout: 2,
            max_retries: 2,
            ..crate::config::AppConfig::default()
        };
        LiveRecorder::new(&config, cancel).unwrap()
    }

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("douyin-{name}-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn strict_live_contract_rejects_unknown_fields_and_wrong_version() {
        let mut value = serde_json::json!({
            "success": true, "contract_version": 1, "mode": "live",
            "web_rid": "web", "room_id": "room", "title": "title",
            "nickname": "anchor", "sec_user_id": "sec", "user_id": null,
            "cover_url": "", "user_count": 1,
            "m3u8_url": "https://example.com/live.m3u8",
            "output": {"save_dir": "/tmp", "filename": "x_live", "suffix": ".flv"},
            "headers": {}
        });
        value["unknown"] = Value::Bool(true);
        assert!(LivePlanV1::from_value(value).is_err());

        let mut value = serde_json::json!({
            "success": true, "contract_version": 2, "mode": "live",
            "web_rid": "web", "room_id": "room", "title": "title",
            "nickname": "anchor", "sec_user_id": "sec", "user_id": null,
            "cover_url": "", "user_count": 1,
            "m3u8_url": "https://example.com/live.m3u8",
            "output": {"save_dir": "/tmp", "filename": "x_live", "suffix": ".flv"},
            "headers": {}
        });
        assert!(LivePlanV1::from_value(value.take()).is_err());
    }

    #[tokio::test]
    async fn endlist_records_ordered_deduplicated_bytes() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXTINF:0.01,\na.ts\n#EXTINF:0.01,\nb.ts\n#EXT-X-ENDLIST\n".to_vec(),
                )],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
            ("/b.ts".to_string(), vec![response(200, b"BBB".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-endlist");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        let LiveRecorderOutcome::Completed { reason, output } = outcome else {
            panic!("expected completed outcome");
        };
        assert_eq!(reason, LiveCompletionReason::StreamEnded);
        assert_eq!(output.file_size, 6);
        assert_eq!(tokio::fs::read(&output.path).await.unwrap(), b"AAABBB");
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn changing_playlists_do_not_redownload_prior_segments() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![
                    response(200, b"#EXTM3U\n#EXTINF:0.01,\na.ts\n".to_vec()),
                    response(
                        200,
                        b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXTINF:0.01,\nb.ts\n#EXT-X-ENDLIST\n"
                            .to_vec(),
                    ),
                ],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
            ("/b.ts".to_string(), vec![response(200, b"BBB".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-changing-playlist");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        let LiveRecorderOutcome::Completed { output, .. } = outcome else {
            panic!("expected completion");
        };
        assert_eq!(tokio::fs::read(&output.path).await.unwrap(), b"AAABBB");
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn master_playlist_resolves_first_media_playlist() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/master.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nmedia.m3u8\n".to_vec(),
                )],
            ),
            (
                "/media.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXT-X-ENDLIST\n".to_vec(),
                )],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-master");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/master.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(outcome, LiveRecorderOutcome::Completed { .. }));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn playlist_404_retries_then_succeeds() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![
                    response(404, Vec::new()),
                    response(
                        200,
                        b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXT-X-ENDLIST\n".to_vec(),
                    ),
                ],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-retry");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(outcome, LiveRecorderOutcome::Completed { .. }));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn permanent_playlist_404_is_error_not_eof() {
        let server = ScriptServer::start(HashMap::from([(
            "/live.m3u8".to_string(),
            vec![response(404, Vec::new()), response(404, Vec::new())],
        )]))
        .await;
        let dir = temp_dir("live-404");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::NetworkRetryExhausted,
                ..
            }
        ));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn empty_playlist_is_error() {
        let server = ScriptServer::start(HashMap::from([(
            "/live.m3u8".to_string(),
            vec![response(200, b"#EXTM3U\n".to_vec())],
        )]))
        .await;
        let dir = temp_dir("live-empty");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::InvalidPlaylist,
                ..
            }
        ));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn permanent_segment_failure_is_typed_error() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXT-X-ENDLIST\n".to_vec(),
                )],
            ),
            (
                "/a.ts".to_string(),
                vec![response(504, Vec::new()), response(504, Vec::new())],
            ),
        ]))
        .await;
        let dir = temp_dir("live-segment-failure");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::SegmentRetryExhausted,
                ..
            }
        ));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn truncated_segment_is_discarded_before_retry() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXT-X-ENDLIST\n".to_vec(),
                )],
            ),
            (
                "/a.ts".to_string(),
                vec![
                    truncated_response(b"BAD".to_vec(), 6),
                    response(200, b"GOOD".to_vec()),
                ],
            ),
        ]))
        .await;
        let dir = temp_dir("live-truncated-segment");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        let LiveRecorderOutcome::Completed { output, .. } = outcome else {
            panic!("expected retry completion");
        };
        assert_eq!(tokio::fs::read(&output.path).await.unwrap(), b"GOOD");
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn injected_write_failure_is_typed_error() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXT-X-ENDLIST\n".to_vec(),
                )],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-write-failure");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .with_test_writer_failure(TestWriterFailure::Write)
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::FileWrite,
                ..
            }
        ));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn injected_flush_failure_is_typed_error_even_after_bytes() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:0.01,\na.ts\n#EXT-X-ENDLIST\n".to_vec(),
                )],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-flush-failure");
        let outcome = recorder(Arc::new(AtomicBool::new(false)))
            .with_test_writer_failure(TestWriterFailure::Flush)
            .record(&plan(server.url("/live.m3u8"), dir.clone()), |_| {})
            .await;
        server.task.await.unwrap();
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::FileFlush,
                ..
            }
        ));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn stop_after_first_segment_keeps_usable_partial_file() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(
                    200,
                    b"#EXTM3U\n#EXTINF:1,\na.ts\n#EXTINF:1,\nb.ts\n".to_vec(),
                )],
            ),
            ("/a.ts".to_string(), vec![response(200, b"AAA".to_vec())]),
        ]))
        .await;
        let dir = temp_dir("live-stop");
        let cancel = Arc::new(AtomicBool::new(false));
        let progress_cancel = cancel.clone();
        let outcome = recorder(cancel)
            .record(&plan(server.url("/live.m3u8"), dir.clone()), move |_| {
                progress_cancel.store(true, Ordering::Relaxed);
            })
            .await;
        server.task.await.unwrap();
        let LiveRecorderOutcome::Completed { reason, output } = outcome else {
            panic!("expected stopped completion");
        };
        assert_eq!(reason, LiveCompletionReason::UserStopped);
        assert_eq!(output.file_size, 3);
        assert_eq!(tokio::fs::read(&output.path).await.unwrap(), b"AAA");
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn stop_before_bytes_is_typed_error() {
        let dir = temp_dir("live-stop-empty");
        let cancel = Arc::new(AtomicBool::new(true));
        let outcome = recorder(cancel)
            .record(
                &plan("http://127.0.0.1:9/live.m3u8".to_string(), dir.clone()),
                |_| {},
            )
            .await;
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::StoppedBeforeUsableBytes,
                ..
            }
        ));
        let _ = tokio::fs::remove_dir_all(dir).await;
    }

    #[tokio::test]
    async fn stop_during_chunked_segment_does_not_commit_partial_segment() {
        let server = ScriptServer::start(HashMap::from([
            (
                "/live.m3u8".to_string(),
                vec![response(200, b"#EXTM3U\n#EXTINF:1,\na.ts\n".to_vec())],
            ),
            (
                "/a.ts".to_string(),
                vec![chunked_response(
                    b"AAABBB".to_vec(),
                    3,
                    Duration::from_millis(100),
                )],
            ),
        ]))
        .await;
        let dir = temp_dir("live-stop-transfer");
        let cancel = Arc::new(AtomicBool::new(false));
        let task_cancel = cancel.clone();
        let live_plan = plan(server.url("/live.m3u8"), dir.clone());
        let record_task =
            tokio::spawn(async move { recorder(task_cancel).record(&live_plan, |_| {}).await });
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        while server.request_count() < 2 {
            assert!(tokio::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
        cancel.store(true, Ordering::Relaxed);
        let outcome = record_task.await.unwrap();
        server.task.await.unwrap();
        assert!(matches!(
            outcome,
            LiveRecorderOutcome::Failed {
                kind: LiveFailureKind::StoppedBeforeUsableBytes,
                ..
            }
        ));
        assert!(!dir.join("record_live.flv").exists());
        let _ = tokio::fs::remove_dir_all(dir).await;
    }
}
