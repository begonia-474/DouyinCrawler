//! 下载引擎核心模块
//!
//! 使用 reqwest 实现流式 HTTP 下载，支持：
//! - 断点续传
//! - 进度回调
//! - 重试逻辑（指数退避 + jitter）
//! - 并发控制（Semaphore）
//! - 取消机制（AtomicBool）
//! - CDN fallback（URL 列表逐个尝试）

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use log::{info, warn};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;

/// 下载引擎配置
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// 最大并发下载任务数（对应 config.max_tasks）
    pub max_concurrent: usize,
    /// 最大重试次数
    pub max_retries: u32,
    /// 请求超时时间（秒）
    pub timeout: u64,
    /// 最大 TCP 连接数（单个 URL 的并发连接数）
    pub max_connections: usize,
    /// User-Agent
    pub user_agent: String,
    /// Referer
    pub referer: String,
    /// Cookie
    pub cookie: String,
    /// 代理地址（空字符串表示不使用代理）
    pub proxy: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            max_retries: 5,
            timeout: 5,
            max_connections: 5,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36".to_string(),
            referer: "https://www.douyin.com/".to_string(),
            cookie: String::new(),
            proxy: String::new(),
        }
    }
}

/// 下载项
#[derive(Debug, Clone)]
pub struct DownloadItem {
    /// 下载 URL（可能是列表，用于 CDN 降级）
    pub url: DownloadUrl,
    /// 最终文件路径
    pub save_path: PathBuf,
    /// 临时文件路径（.tmp）
    pub temp_path: PathBuf,
    /// 自定义 headers
    pub headers: HashMap<String, String>,
    /// 任务 ID
    pub task_id: String,
    /// 预期文件大小（Content-Length）
    pub file_size: Option<u64>,
}

/// 下载 URL（支持单个或多个）
#[derive(Debug, Clone)]
pub enum DownloadUrl {
    /// 单个 URL
    Single(String),
    /// 多个 URL（CDN 降级）
    Multiple(Vec<String>),
}

impl DownloadUrl {
    /// 获取第一个 URL
    pub fn first(&self) -> Option<&str> {
        match self {
            Self::Single(url) => Some(url),
            Self::Multiple(urls) => urls.first().map(|s| s.as_str()),
        }
    }

    /// 获取所有 URL
    pub fn all(&self) -> Vec<&str> {
        match self {
            Self::Single(url) => vec![url],
            Self::Multiple(urls) => urls.iter().map(|s| s.as_str()).collect(),
        }
    }
}

/// 下载结果
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// 任务 ID
    pub task_id: String,
    /// 下载完成的文件路径
    pub path: PathBuf,
    /// 实际文件大小
    pub file_size: u64,
    /// 是否跳过（文件已存在）
    pub skipped: bool,
}

/// 下载错误
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("所有 URL 都失败: {0}")]
    AllUrlsFailed(String),
    
    #[error("文件大小不匹配: 期望 {expected}, 实际 {actual}")]
    SizeMismatch { expected: u64, actual: u64 },
    
    #[error("下载已取消")]
    Cancelled,
    
    #[error("重试次数耗尽: {0}")]
    RetriesExhausted(String),
}

/// 下载引擎
pub struct DownloadEngine {
    /// HTTP 客户端
    client: Client,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 取消信号
    cancel_signal: Arc<AtomicBool>,
    /// 配置
    config: EngineConfig,
}

impl DownloadEngine {
    /// 创建新的下载引擎
    pub fn new(config: EngineConfig) -> Self {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .pool_max_idle_per_host(config.max_connections)
            .user_agent(&config.user_agent);

        // 代理配置
        if !config.proxy.is_empty() {
            if let Ok(proxy) = reqwest::Proxy::all(&config.proxy) {
                builder = builder.proxy(proxy);
            }
        }

        let client = builder.build().expect("Failed to create HTTP client");
        
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            cancel_signal: Arc::new(AtomicBool::new(false)),
            config,
        }
    }
    
    
    /// 设置取消信号（用于任务级取消控制）
    ///
    /// 将引擎的取消信号绑定到外部信号（如 AppState 中的 per-task 信号），
    /// 这样 cancel_task() 设置的信号会直接让引擎停止下载。
    pub fn with_cancel_signal(mut self, signal: Arc<AtomicBool>) -> Self {
        self.cancel_signal = signal;
        self
    }
    /// 下载单个文件
    pub async fn download(
        &self,
        item: &DownloadItem,
        progress: impl Fn(u64, u64) + Send + Sync,
    ) -> Result<DownloadResult, DownloadError> {
        // 检查取消信号
        if self.cancel_signal.load(Ordering::Relaxed) {
            return Err(DownloadError::Cancelled);
        }
        
        // 获取并发许可
        let _permit = self.semaphore.acquire().await
            .map_err(|e| DownloadError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        // 检查文件是否已存在
        if item.save_path.exists() {
            let metadata = tokio::fs::metadata(&item.save_path).await?;
            let file_size = metadata.len();
            
            info!(
                "[DownloadEngine] 文件已存在，跳过: {} ({} bytes)",
                item.save_path.display(),
                file_size
            );
            
            return Ok(DownloadResult {
                task_id: item.task_id.clone(),
                path: item.save_path.clone(),
                file_size,
                skipped: true,
            });
        }
        
        // CDN fallback：逐个 URL 尝试
        let urls = item.url.all();
        let mut last_error = None;
        
        for url in &urls {
            match self.try_download(url, item, &progress).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!(
                        "[DownloadEngine] URL 失败，尝试下一个: {} ({})",
                        url,
                        e
                    );
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            DownloadError::AllUrlsFailed("所有 URL 都失败".to_string())
        }))
    }
    
    /// 尝试下载单个 URL
    async fn try_download(
        &self,
        url: &str,
        item: &DownloadItem,
        progress: &(impl Fn(u64, u64) + Send + Sync),
    ) -> Result<DownloadResult, DownloadError> {
        let mut last_error = None;
        
        for attempt in 0..self.config.max_retries {
            // 检查取消信号
            if self.cancel_signal.load(Ordering::Relaxed) {
                return Err(DownloadError::Cancelled);
            }
            
            match self.do_download(url, item, progress).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let is_retryable = match &e {
                        DownloadError::Network(err) => {
                            err.is_timeout() || err.is_connect() || 
                            err.status().map_or(false, |s| {
                                s == reqwest::StatusCode::TOO_MANY_REQUESTS ||
                                s.is_server_error()
                            })
                        }
                        _ => false,
                    };
                    
                    if is_retryable && attempt < self.config.max_retries - 1 {
                        let wait = self.calculate_backoff(attempt);
                        warn!(
                            "[DownloadEngine] 下载失败，等待 {:?} 后重试 ({}/{}): {}",
                            wait,
                            attempt + 1,
                            self.config.max_retries,
                            e
                        );
                        tokio::time::sleep(wait).await;
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            DownloadError::RetriesExhausted("重试次数耗尽".to_string())
        }))
    }
    
    /// 执行实际下载
    async fn do_download(
        &self,
        url: &str,
        item: &DownloadItem,
        progress: &(impl Fn(u64, u64) + Send + Sync),
    ) -> Result<DownloadResult, DownloadError> {
        // 构建 headers
        let mut headers = reqwest::header::HeaderMap::new();
        
        // 添加自定义 headers
        for (key, value) in &item.headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(value),
            ) {
                headers.insert(name, val);
            }
        }
        
        // 添加默认 headers
        if !self.config.cookie.is_empty() {
            headers.insert(
                reqwest::header::COOKIE,
                reqwest::header::HeaderValue::from_str(&self.config.cookie)
                    .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
            );
        }
        headers.insert(
            reqwest::header::REFERER,
            reqwest::header::HeaderValue::from_str(&self.config.referer)
                .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
        );
        
        // 检查是否有已下载的部分（断点续传）
        let existing_size = if item.temp_path.exists() {
            let metadata = tokio::fs::metadata(&item.temp_path).await?;
            metadata.len()
        } else {
            0
        };
        
        // 如果有已下载的部分，添加 Range header
        if existing_size > 0 {
            headers.insert(
                reqwest::header::RANGE,
                reqwest::header::HeaderValue::from_str(&format!("bytes={}-", existing_size))
                    .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("bytes=0-")),
            );
        }
        
        // 发送请求
        let response = self.client
            .get(url)
            .headers(headers)
            .send()
            .await?;
        
        // 检查状态码
        let status = response.status();
        if status != reqwest::StatusCode::OK && status != reqwest::StatusCode::PARTIAL_CONTENT {
            return Err(DownloadError::Network(
                response.error_for_status().unwrap_err()
            ));
        }
        
        // 获取总大小
        let content_length = response.content_length().unwrap_or(0);
        let total_size = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            existing_size + content_length
        } else {
            content_length
        };
        
        // 如果不是 206，重置已下载大小
        let actual_existing_size = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            existing_size
        } else {
            0
        };
        
        // 打开文件
        let mut file = if actual_existing_size > 0 {
            // 追加模式
            File::options()
                .append(true)
                .open(&item.temp_path)
                .await?
        } else {
            // 创建模式
            File::create(&item.temp_path).await?
        };
        
        // 流式下载
        let mut downloaded = actual_existing_size;
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        
        while let Some(chunk_result) = stream.next().await {
            // 检查取消信号
            if self.cancel_signal.load(Ordering::Relaxed) {
                // 清理临时文件
                let _ = tokio::fs::remove_file(&item.temp_path).await;
                return Err(DownloadError::Cancelled);
            }
            
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            // 调用进度回调
            progress(downloaded, total_size);
        }
        
        file.flush().await?;
        drop(file);
        
        // 文件大小校验
        if item.file_size.is_some() && total_size > 0 {
            let actual_size = tokio::fs::metadata(&item.temp_path).await?.len();
            let expected_size = item.file_size.unwrap_or(total_size);
            
            if actual_size != expected_size {
                // 清理临时文件
                let _ = tokio::fs::remove_file(&item.temp_path).await;
                return Err(DownloadError::SizeMismatch {
                    expected: expected_size,
                    actual: actual_size,
                });
            }
        }
        
        // 重命名临时文件为最终文件
        tokio::fs::rename(&item.temp_path, &item.save_path).await?;
        
        let final_size = tokio::fs::metadata(&item.save_path).await?.len();
        
        Ok(DownloadResult {
            task_id: item.task_id.clone(),
            path: item.save_path.clone(),
            file_size: final_size,
            skipped: false,
        })
    }
    
    /// 计算指数退避时间
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base = 2u64.pow(attempt);
        let jitter = rand::random::<u64>() % 1000; // 0-999ms
        Duration::from_millis(base * 1000 + jitter)
    }
    
    /// 批量下载
    pub async fn batch_download(
        &self,
        items: Vec<DownloadItem>,
    ) -> Vec<Result<DownloadResult, DownloadError>> {
        let mut results = Vec::with_capacity(items.len());
        
        for item in items {
            let result = self.download(&item, |_, _| {}).await;
            results.push(result);
        }
        
        results
    }
    
    /// 发送取消信号
    pub fn cancel(&self) {
        self.cancel_signal.store(true, Ordering::Relaxed);
    }
    
    /// 重置取消信号
    pub fn reset_cancel(&self) {
        self.cancel_signal.store(false, Ordering::Relaxed);
    }
    
    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancel_signal.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_download_url_first() {
        let url = DownloadUrl::Single("https://example.com".to_string());
        assert_eq!(url.first(), Some("https://example.com"));
        
        let url = DownloadUrl::Multiple(vec![
            "https://cdn1.example.com".to_string(),
            "https://cdn2.example.com".to_string(),
        ]);
        assert_eq!(url.first(), Some("https://cdn1.example.com"));
    }
    
    #[test]
    fn test_download_url_all() {
        let url = DownloadUrl::Single("https://example.com".to_string());
        assert_eq!(url.all(), vec!["https://example.com"]);
        
        let url = DownloadUrl::Multiple(vec![
            "https://cdn1.example.com".to_string(),
            "https://cdn2.example.com".to_string(),
        ]);
        assert_eq!(
            url.all(),
            vec!["https://cdn1.example.com", "https://cdn2.example.com"]
        );
    }
    
    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.timeout, 5);
        assert_eq!(config.max_connections, 5);
    }
}
