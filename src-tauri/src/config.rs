//! 配置管理模块
//!
//! 从 config/app.json 读取和写入配置

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use log::{info, warn, error};

fn default_page_counts() -> u32 { 20 }
fn default_timeout() -> u32 { 5 }
fn default_max_connections() -> u32 { 5 }
fn default_max_retries() -> u32 { 5 }
fn default_max_tasks() -> u32 { 10 }

/// 应用配置
#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct AppConfig {
    pub cookie: String,
    pub download_path: String,
    pub naming: String,
    pub encryption: String,
    pub proxy: String,
    pub app_name: String,
    pub folderize: bool,
    pub music: bool,
    pub cover: bool,
    pub desc: bool,
    pub interval: Option<String>,
    #[serde(default = "default_page_counts")]
    pub page_counts: u32,
    #[serde(default)]
    pub max_counts: u32,
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_max_tasks")]
    pub max_tasks: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cookie: String::new(),
            download_path: "Download".to_string(),
            naming: "{create}_{desc}".to_string(),
            encryption: "ab".to_string(),
            proxy: String::new(),
            app_name: "douyin".to_string(),
            folderize: false,
            music: false,
            cover: false,
            desc: false,
            interval: None,
            page_counts: 20,
            max_counts: 0,
            timeout: 5,
            max_connections: 5,
            max_retries: 5,
            max_tasks: 10,
        }
    }
}

/// 完整配置文件结构
#[derive(Serialize, Deserialize, Clone, Debug, specta::Type)]
pub struct ConfigFile {
    pub douyin: Option<AppConfig>,
    pub tiktok: Option<AppConfig>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            douyin: Some(AppConfig::default()),
            tiktok: Some(AppConfig {
                app_name: "tiktok".to_string(),
                ..AppConfig::default()
            }),
        }
    }
}

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: ConfigFile,
}

impl ConfigManager {
    #[cfg(test)]
    pub(crate) fn for_test(config: AppConfig) -> Self {
        Self {
            config_path: std::env::temp_dir().join("douyin-crawler-test-config.json"),
            config: ConfigFile {
                douyin: Some(config),
                tiktok: None,
            },
        }
    }

    /// 创建配置管理器
    pub fn new() -> Self {
        let config_dir = Self::get_config_dir();
        let config_path = config_dir.join("app.json");

        let config = Self::load_config(&config_path);

        Self {
            config_path,
            config,
        }
    }

    /// 获取配置目录
    fn get_config_dir() -> PathBuf {
        // 生产模式优先使用平台标准配置目录
        #[cfg(not(debug_assertions))]
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("DouyinCrawler");
            let _ = std::fs::create_dir_all(&app_config_dir);
            info!("使用 AppConfig 目录: {:?}", app_config_dir);
            return app_config_dir;
        }

        // 开发模式：优先使用项目根目录下的 config 目录
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = manifest_dir.parent().unwrap_or(&manifest_dir);
        let config_dir = project_root.join("config");

        if config_dir.exists() {
            return config_dir;
        }

        // 回退：当前工作目录
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let current_config_dir = current_dir.join("config");

        if current_config_dir.exists() {
            return current_config_dir;
        }

        // 都不存在时，创建在项目根目录
        info!("创建新 config 目录: {:?}", config_dir);
        let _ = std::fs::create_dir_all(&config_dir);
        config_dir
    }

    /// 加载配置
    fn load_config(config_path: &Path) -> ConfigFile {
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => {
                    match serde_json::from_str::<ConfigFile>(&content) {
                        Ok(mut config) => {
                            // 清理 cookie 中可能残留的换行符
                            if let Some(ref mut dc) = config.douyin {
                                let original_len = dc.cookie.len();
                                dc.cookie = dc.cookie
                                    .chars()
                                    .filter(|c| *c != '\n' && *c != '\r')
                                    .collect::<String>()
                                    .split_whitespace()
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                if dc.cookie.len() != original_len {
                                    info!("[config] cookie 已清理换行符 ({} -> {})", original_len, dc.cookie.len());
                                }
                                info!("[config] 已加载配置: {:?}", config_path);
                                info!("[config] douyin.cookie loaded (len={})", dc.cookie.len());
                                info!("[config] page_counts={}, max_counts={}, timeout={}, max_connections={}, max_retries={}, max_tasks={}",
                                    dc.page_counts, dc.max_counts, dc.timeout, dc.max_connections, dc.max_retries, dc.max_tasks);
                            } else {
                                warn!("[config] douyin 配置为空!");
                            }
                            config
                        }
                        Err(e) => {
                            error!("解析配置文件失败: {}", e);
                            ConfigFile::default()
                        }
                    }
                }
                Err(e) => {
                    error!("读取配置文件失败: {}", e);
                    ConfigFile::default()
                }
            }
        } else {
            warn!("配置文件不存在: {:?}", config_path);
            ConfigFile::default()
        }
    }

    /// 获取 douyin 配置
    pub fn get_douyin_config(&self) -> AppConfig {
        self.config.douyin.clone().unwrap_or_default()
    }


    /// 更新 douyin 配置
    pub fn update_douyin_config(&mut self, updates: &HashMap<String, String>) -> Result<(), String> {
        let mut config = self.get_douyin_config();

        for (key, value) in updates {
            match key.as_str() {
                "cookie" => {
                    // 清理粘贴时可能带入的换行符
                    let cleaned: String = value
                        .chars()
                        .filter(|c| *c != '\n' && *c != '\r')
                        .collect::<String>()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
                    info!("[config] 更新 cookie (原始 len={}, 清理后 len={})", value.len(), cleaned.len());
                    config.cookie = cleaned;
                }
                "download_path" => config.download_path = value.clone(),
                "naming" => config.naming = value.clone(),
                "encryption" => config.encryption = value.clone(),
                "proxy" => config.proxy = value.clone(),
                "app_name" => config.app_name = value.clone(),
                "folderize" => config.folderize = value == "true",
                "music" => config.music = value == "true",
                "cover" => config.cover = value == "true",
                "desc" => config.desc = value == "true",
                "interval" => config.interval = Some(value.clone()),
                "page_counts" => config.page_counts = value.parse().unwrap_or(20),
                "max_counts" => config.max_counts = value.parse().unwrap_or(0),
                "timeout" => config.timeout = value.parse().unwrap_or(5),
                "max_connections" => config.max_connections = value.parse().unwrap_or(5),
                "max_retries" => config.max_retries = value.parse().unwrap_or(5),
                "max_tasks" => config.max_tasks = value.parse().unwrap_or(10),
                _ => {}
            }
        }

        self.config.douyin = Some(config);
        self.save_config()
    }

    /// 保存配置
    fn save_config(&self) -> Result<(), String> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        info!("已保存配置: {:?}", self.config_path);
        Ok(())
    }
}
