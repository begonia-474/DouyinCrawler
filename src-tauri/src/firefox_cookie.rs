//! Firefox Cookie 自动获取模块
//!
//! 从 Firefox 浏览器读取指定域名的 cookies
//! 仅支持 Firefox，因为 Chromium 系浏览器使用了 App Bound Encryption

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::command;

/// Firefox 配置文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirefoxProfile {
    pub name: String,
    pub path: String,
    pub is_default: bool,
    pub has_cookies: bool,
}

/// Cookie 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub host: String,
    pub name: String,
    pub value: String,
    pub path: String,
    pub expiry: i64,
}

/// 获取 Firefox Profiles 目录
fn get_firefox_profiles_dir() -> Result<PathBuf, String> {
    let app_data = dirs::config_dir()
        .ok_or_else(|| "无法获取 APPDATA 目录".to_string())?;

    #[cfg(target_os = "windows")]
    let profiles_dir = app_data.join("Mozilla").join("Firefox");

    #[cfg(target_os = "macos")]
    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取 HOME 目录".to_string())?
        .join("Library")
        .join("Application Support")
        .join("Firefox");

    #[cfg(target_os = "linux")]
    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取 HOME 目录".to_string())?
        .join(".mozilla")
        .join("firefox");

    Ok(profiles_dir)
}

/// 解析 profiles.ini 文件
fn parse_profiles_ini(profiles_dir: &Path) -> Result<Vec<FirefoxProfile>, String> {
    let profiles_ini = profiles_dir.join("profiles.ini");

    if !profiles_ini.exists() {
        return Err(format!("profiles.ini 不存在: {}", profiles_ini.display()));
    }

    let content = fs::read_to_string(&profiles_ini)
        .map_err(|e| format!("读取 profiles.ini 失败: {}", e))?;

    let mut profiles = Vec::new();
    let mut current_section = HashMap::new();
    let mut current_section_name = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') && line.ends_with(']') {
            // 保存上一个 section
            if !current_section.is_empty() && current_section_name.starts_with("Profile") {
                if let Some(profile) = parse_profile_section(&current_section, profiles_dir) {
                    profiles.push(profile);
                }
            }
            current_section.clear();
            current_section_name = line[1..line.len()-1].to_string();
        } else if let Some((key, value)) = line.split_once('=') {
            current_section.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // 处理最后一个 section
    if !current_section.is_empty() && current_section_name.starts_with("Profile") {
        if let Some(profile) = parse_profile_section(&current_section, profiles_dir) {
            profiles.push(profile);
        }
    }

    // 查找默认 profile
    let default_path = find_default_profile_path(&content);

    for profile in &mut profiles {
        if let Some(ref default_path) = default_path {
            if profile.path.ends_with(default_path) || profile.path.contains(default_path) {
                profile.is_default = true;
            }
        }
    }

    // 如果没有找到默认的，标记第一个为默认
    if !profiles.iter().any(|p| p.is_default) {
        if let Some(first) = profiles.first_mut() {
            first.is_default = true;
        }
    }

    Ok(profiles)
}

/// 解析单个 Profile section
fn parse_profile_section(section: &HashMap<String, String>, profiles_dir: &Path) -> Option<FirefoxProfile> {
    let name = section.get("Name")?.clone();
    let path_str = section.get("Path")?;
    let is_relative = section.get("IsRelative")
        .map(|v| v == "1")
        .unwrap_or(true);

    let path = if is_relative {
        profiles_dir.join(path_str)
    } else {
        PathBuf::from(path_str)
    };

    let has_cookies = path.join("cookies.sqlite").exists();

    Some(FirefoxProfile {
        name,
        path: path.to_string_lossy().to_string(),
        is_default: false,
        has_cookies,
    })
}

/// 从 profiles.ini 中查找 Default profile 路径
fn find_default_profile_path(content: &str) -> Option<String> {
    // 查找 [Install*] section 的 Default
    let mut in_install_section = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("[Install") {
            in_install_section = true;
            continue;
        }

        if line.starts_with('[') {
            in_install_section = false;
            continue;
        }

        if in_install_section {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "Default" {
                    return Some(value.trim().to_string());
                }
            }
        }
    }

    // 如果没找到 Install section，查找 General section 的 StartWithLastProfile
    // 或者查找 Default=1 的 Profile
    for line in content.lines() {
        let line = line.trim();
        if line == "Default=1" {
            // 找到这个标记前的 Path
            // 这需要更复杂的解析，暂时跳过
        }
    }

    None
}

/// 获取所有 Firefox profiles
pub fn get_firefox_profiles() -> Result<Vec<FirefoxProfile>, String> {
    let profiles_dir = get_firefox_profiles_dir()?;
    parse_profiles_ini(&profiles_dir)
}

/// 读取 Firefox cookies
fn read_cookies_from_profile(profile: &FirefoxProfile) -> Result<Vec<Cookie>, String> {
    let cookies_db = Path::new(&profile.path).join("cookies.sqlite");

    if !cookies_db.exists() {
        return Err(format!("Cookies 数据库不存在: {}", cookies_db.display()));
    }

    // 复制数据库避免锁定问题
    let temp_db = Path::new(&profile.path).join("cookies_temp.sqlite");
    fs::copy(&cookies_db, &temp_db)
        .map_err(|e| format!("复制 cookies 数据库失败: {}", e))?;

    let result = (|| -> Result<Vec<Cookie>, String> {
        let conn = Connection::open(&temp_db)
            .map_err(|e| format!("打开 cookies 数据库失败: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT host, name, value, path, expiry FROM moz_cookies")
            .map_err(|e| format!("准备查询失败: {}", e))?;

        let cookies = stmt
            .query_map([], |row| {
                Ok(Cookie {
                    host: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    path: row.get(3)?,
                    expiry: row.get(4)?,
                })
            })
            .map_err(|e| format!("查询 cookies 失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(cookies)
    })();

    // 清理临时文件
    let _ = fs::remove_file(&temp_db);

    result
}

/// 获取指定域名的 cookies
pub fn get_cookies_by_domain(
    profile: &FirefoxProfile,
    domain: &str,
) -> Result<HashMap<String, String>, String> {
    let cookies = read_cookies_from_profile(profile)?;

    let mut result = HashMap::new();

    for cookie in cookies {
        let matches = if domain.starts_with('.') {
            // .douyin.com 匹配 .douyin.com 和 *.douyin.com
            cookie.host == domain || cookie.host.ends_with(domain)
        } else {
            // 精确匹配
            cookie.host == domain || cookie.host == format!(".{}", domain)
        };

        if matches {
            result.insert(cookie.name, cookie.value);
        }
    }

    Ok(result)
}

/// 获取抖音的完整 cookie 字符串
pub fn get_douyin_cookie_string(profile: &FirefoxProfile) -> Result<String, String> {
    let domains = vec![
        ".douyin.com",
        "www.douyin.com",
        ".iesdouyin.com",
        "login.douyin.com",
    ];

    let mut all_cookies = HashMap::new();

    for domain in domains {
        let cookies = get_cookies_by_domain(profile, domain)?;
        all_cookies.extend(cookies);
    }

    if all_cookies.is_empty() {
        return Err("未找到抖音的 cookies".to_string());
    }

    let cookie_str: String = all_cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ");

    Ok(cookie_str)
}

/// 获取默认 profile 的抖音 cookie
pub fn get_douyin_cookie() -> Result<(String, HashMap<String, String>), String> {
    let profiles = get_firefox_profiles()?;

    // 优先使用有 cookies 的默认 profile
    let profile = profiles
        .iter()
        .find(|p| p.is_default && p.has_cookies)
        .or_else(|| profiles.iter().find(|p| p.has_cookies))
        .ok_or_else(|| "未找到有 cookies 的 Firefox 配置文件".to_string())?;

    let cookies = get_cookies_by_domain(profile, ".douyin.com")?;

    // 合并其他域名的 cookies
    let mut all_cookies = cookies;
    for domain in &["www.douyin.com", ".iesdouyin.com", "login.douyin.com"] {
        let domain_cookies = get_cookies_by_domain(profile, domain)?;
        all_cookies.extend(domain_cookies);
    }

    let cookie_str: String = all_cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ");

    Ok((cookie_str, all_cookies))
}

// ============ Tauri Commands ============

/// Tauri 命令: 获取 Firefox profiles 列表
#[command]
pub fn get_firefox_profiles_command() -> Result<Vec<FirefoxProfile>, String> {
    get_firefox_profiles()
}

/// Tauri 命令: 获取抖音 cookie
#[command]
pub fn get_douyin_cookie_command() -> Result<String, String> {
    let (cookie_str, _) = get_douyin_cookie()?;
    Ok(cookie_str)
}

/// Tauri 命令: 从指定 profile 获取 cookie
#[command]
pub fn get_douyin_cookie_from_profile_command(profile_name: String) -> Result<String, String> {
    let profiles = get_firefox_profiles()?;

    let profile = profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| format!("未找到名为 '{}' 的 Firefox 配置文件", profile_name))?;

    if !profile.has_cookies {
        return Err(format!("配置文件 '{}' 没有 cookies 数据库", profile_name));
    }

    let cookies = get_douyin_cookie_string(profile)?;
    Ok(cookies)
}

/// Tauri 命令: 获取指定域名的 cookie
#[command]
pub fn get_firefox_cookie_command(domain: String) -> Result<HashMap<String, String>, String> {
    let profiles = get_firefox_profiles()?;

    let profile = profiles
        .iter()
        .find(|p| p.is_default && p.has_cookies)
        .or_else(|| profiles.iter().find(|p| p.has_cookies))
        .ok_or_else(|| "未找到有 cookies 的 Firefox 配置文件".to_string())?;

    get_cookies_by_domain(profile, &domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_firefox_profiles() {
        let profiles = get_firefox_profiles();
        println!("Firefox profiles: {:?}", profiles);
        assert!(profiles.is_ok());
    }

    #[test]
    fn test_get_douyin_cookie() {
        let result = get_douyin_cookie();
        println!("Douyin cookie result: {:?}", result);
        // 这个测试需要 Firefox 登录了抖音才能通过
    }
}
