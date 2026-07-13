//! Versioned Python -> Rust download resolver contracts.

use std::collections::HashMap;

use serde::Deserialize;

use crate::db::{UserInfo, VideoInfo};

pub const SINGLE_DOWNLOAD_CONTRACT_VERSION: u32 = 1;

/// Shared contract version for paged download plans (post/like/mix/collects).
pub const PAGED_DOWNLOAD_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKindV1 {
    Video,
    Image,
    LivePhoto,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SingleAccessoryKind {
    Music,
    Cover,
    Description,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleOutputSpec {
    pub filename: String,
    pub suffix: String,
    pub folder_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleAccessory {
    pub kind: SingleAccessoryKind,
    pub output: SingleOutputSpec,
    pub url: Option<String>,
    pub content: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaDownloadItemV1 {
    pub media_key: String,
    pub aweme_id: String,
    pub urls: Vec<String>,
    pub kind: MediaKindV1,
    pub output: SingleOutputSpec,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub accessories: Vec<SingleAccessory>,
    pub metadata: VideoInfo,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleDownloadPlanV1 {
    pub success: bool,
    pub contract_version: u32,
    pub mode: String,
    pub save_dir: String,
    pub items: Vec<MediaDownloadItemV1>,
    pub total: i64,
}

impl SingleDownloadPlanV1 {
    pub fn from_value(value: serde_json::Value) -> Result<Self, String> {
        let plan: Self = serde_json::from_value(value)
            .map_err(|error| format!("单视频解析契约反序列化失败: {error}"))?;
        plan.validate()?;
        Ok(plan)
    }

    fn validate(&self) -> Result<(), String> {
        if self.contract_version != SINGLE_DOWNLOAD_CONTRACT_VERSION {
            return Err(format!(
                "不支持的单视频解析契约版本: {}, 当前支持 {}",
                self.contract_version, SINGLE_DOWNLOAD_CONTRACT_VERSION
            ));
        }
        if !self.success {
            return Err("单视频解析契约标记为失败".to_string());
        }
        if self.mode != "one" {
            return Err(format!(
                "单视频解析契约 mode 必须为 one，实际为 {}",
                self.mode
            ));
        }
        if self.save_dir.trim().is_empty() {
            return Err("单视频解析契约 save_dir 不能为空".to_string());
        }
        if self.items.is_empty() {
            return Err("单视频解析契约 items 不能为空".to_string());
        }
        if self.total != self.items.len() as i64 {
            return Err(format!(
                "单视频解析契约 total 与 items 数量不一致: {} != {}",
                self.total,
                self.items.len()
            ));
        }
        let mut seen_keys = std::collections::HashSet::new();
        for item in &self.items {
            item.validate()?;
            if !seen_keys.insert(item.media_key.as_str()) {
                return Err(format!("单视频解析项 media_key 重复: {}", item.media_key));
            }
        }
        Ok(())
    }
}

/// Typed contract for a single page of a paged download mode (post/like/mix/collects).
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagedDownloadPlanV1 {
    pub success: bool,
    pub contract_version: u32,
    pub mode: String,
    pub save_dir: String,
    pub items: Vec<MediaDownloadItemV1>,
    pub next_cursor: Option<i64>,
    pub has_more: bool,
    #[serde(default)]
    pub page_aweme_ids: Vec<String>,
    #[serde(default)]
    pub user_profile: Option<UserInfo>,
}

impl PagedDownloadPlanV1 {
    pub fn from_value_for_mode(
        value: serde_json::Value,
        expected_mode: &str,
    ) -> Result<Self, String> {
        let plan: Self = serde_json::from_value(value)
            .map_err(|error| format!("分页解析契约反序列化失败: {error}"))?;
        plan.validate(expected_mode)?;
        Ok(plan)
    }

    fn validate(&self, expected_mode: &str) -> Result<(), String> {
        if self.contract_version != PAGED_DOWNLOAD_CONTRACT_VERSION {
            return Err(format!(
                "不支持的分页解析契约版本: {}, 当前支持 {}",
                self.contract_version, PAGED_DOWNLOAD_CONTRACT_VERSION
            ));
        }
        if !self.success {
            return Err("分页解析契约标记为失败".to_string());
        }
        if self.mode != expected_mode {
            return Err(format!(
                "分页解析契约 mode 不匹配: expected={expected_mode}, actual={}",
                self.mode
            ));
        }
        if self.save_dir.trim().is_empty() {
            return Err("分页解析契约 save_dir 不能为空".to_string());
        }
        if self.has_more && self.next_cursor.is_none() {
            return Err("has_more=true 但 next_cursor 为空".to_string());
        }
        if !self.has_more && self.next_cursor.is_some() {
            return Err("has_more=false 但 next_cursor 非空".to_string());
        }
        if self.has_more && self.items.is_empty() {
            return Err("items 为空但 has_more=true".to_string());
        }

        let mut page_ids = std::collections::HashSet::new();
        for aweme_id in &self.page_aweme_ids {
            if aweme_id.trim().is_empty() {
                return Err("page_aweme_ids 包含空标识".to_string());
            }
            if !page_ids.insert(aweme_id.as_str()) {
                return Err(format!("page_aweme_ids 重复: {aweme_id}"));
            }
        }
        if !self.items.is_empty() && self.page_aweme_ids.is_empty() {
            return Err("items 非空但 page_aweme_ids 为空".to_string());
        }

        let mut seen_keys = std::collections::HashSet::new();
        let mut item_aweme_ids = std::collections::HashSet::new();
        for item in &self.items {
            item.validate()?;
            if !seen_keys.insert(&item.media_key) {
                return Err(format!("重复 media_key: {}", item.media_key));
            }
            if !page_ids.contains(item.aweme_id.as_str()) {
                return Err(format!(
                    "分页项 aweme_id={} 不在 page_aweme_ids 中",
                    item.aweme_id
                ));
            }
            item_aweme_ids.insert(item.aweme_id.as_str());
        }
        for aweme_id in &self.page_aweme_ids {
            if !item_aweme_ids.contains(aweme_id.as_str()) {
                return Err(format!(
                    "page_aweme_ids 中的作品没有媒体项: {aweme_id}"
                ));
            }
        }
        Ok(())
    }
}

impl MediaDownloadItemV1 {
    pub fn media_index(&self) -> Result<i64, String> {
        let kind = match self.kind {
            MediaKindV1::Video => "video",
            MediaKindV1::Image => "image",
            MediaKindV1::LivePhoto => "live_photo",
        };
        let prefix = format!("{}:{kind}:", self.aweme_id);
        let index = self
            .media_key
            .strip_prefix(&prefix)
            .ok_or_else(|| format!("media_key 与 aweme_id/kind 不一致: {}", self.media_key))?
            .parse::<i64>()
            .map_err(|_| format!("media_key index 无效: {}", self.media_key))?;
        let valid = match self.kind {
            MediaKindV1::Video => index == 0 && self.output.suffix == ".mp4",
            MediaKindV1::LivePhoto => index >= 1 && self.output.suffix == ".mp4",
            MediaKindV1::Image => index >= 1 && self.output.suffix == ".webp",
        };
        if !valid {
            return Err(format!("media_key/kind/index/suffix 约定不一致: {}", self.media_key));
        }
        Ok(index)
    }

    fn validate(&self) -> Result<(), String> {
        if self.aweme_id.trim().is_empty() || self.media_key.trim().is_empty() {
            return Err("媒体项 aweme_id/media_key 不能为空".to_string());
        }
        self.media_index()?;
        if self.urls.is_empty() || self.urls.iter().any(|url| url.trim().is_empty()) {
            return Err(format!("媒体项 {} 至少需要一个下载地址", self.media_key));
        }
        if self.output.filename.trim().is_empty()
            || self.output.folder_name.as_ref().is_some_and(|value| value.trim().is_empty())
        {
            return Err(format!("媒体项 {} 的 output 不完整", self.media_key));
        }
        if self.metadata.aweme_id != self.aweme_id {
            return Err(format!("媒体项 {} 的 metadata 标识不一致", self.media_key));
        }
        for accessory in &self.accessories {
            if accessory.output.filename.trim().is_empty()
                || !accessory.output.suffix.starts_with('.')
            {
                return Err(format!("媒体项 {} 的附件 output 不完整", self.media_key));
            }
            match accessory.kind {
                SingleAccessoryKind::Music | SingleAccessoryKind::Cover
                    if accessory.url.as_deref().is_none_or(|url| url.trim().is_empty()) =>
                {
                    return Err(format!("媒体项 {} 的音乐或封面附件缺少 URL", self.media_key));
                }
                SingleAccessoryKind::Description if accessory.content.is_none() => {
                    return Err(format!("媒体项 {} 的文案附件缺少内容", self.media_key));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub type SingleDownloadItem = MediaDownloadItemV1;
pub type SingleMediaKind = MediaKindV1;

#[cfg(test)]
mod tests {
    use super::SingleDownloadPlanV1;

    fn rejection_error(value: serde_json::Value) -> String {
        match SingleDownloadPlanV1::from_value(value) {
            Ok(_) => panic!("invalid contract must be rejected"),
            Err(error) => error,
        }
    }

    fn valid_plan() -> serde_json::Value {
        serde_json::json!({
            "success": true,
            "contract_version": 1,
            "mode": "one",
            "save_dir": "/tmp/downloads",
            "total": 1,
            "items": [{
                "media_key": "7650450403901017571:video:0",
                "aweme_id": "7650450403901017571",
                "urls": ["https://cdn.example/video.mp4"],
                "kind": "video",
                "output": {
                    "filename": "7650450403901017571_video",
                    "suffix": ".mp4",
                    "folder_name": null
                },
                "headers": {"Referer": "https://www.douyin.com/"},
                "accessories": [],
                "metadata": {
                    "aweme_id": "7650450403901017571",
                    "desc": "contract test",
                    "author_nickname": "tester",
                    "author_sec_uid": "MS4wLjABAAAAcontract",
                    "author_uid": null,
                    "create_time": 1700000000,
                    "video_url": "https://cdn.example/video.mp4",
                    "cover_url": null,
                    "music_title": null,
                    "mix_id": null,
                    "mix_name": null
                }
            }]
        })
    }

    #[test]
    fn accepts_valid_single_download_contract() {
        let plan = SingleDownloadPlanV1::from_value(valid_plan()).unwrap();
        assert_eq!(plan.items[0].urls, vec!["https://cdn.example/video.mp4"]);
        assert_eq!(plan.total, 1);
    }

    #[test]
    fn rejects_unsupported_contract_version() {
        let mut value = valid_plan();
        value["contract_version"] = serde_json::json!(2);

        let error = rejection_error(value);
        assert!(error.contains("不支持的单视频解析契约版本"));
    }

    #[test]
    fn rejects_missing_required_output() {
        let mut value = valid_plan();
        value["items"][0].as_object_mut().unwrap().remove("output");

        let error = rejection_error(value);
        assert!(error.contains("output"));
    }

    #[test]
    fn rejects_empty_url_list() {
        let mut value = valid_plan();
        value["items"][0]["urls"] = serde_json::json!([]);

        let error = rejection_error(value);
        assert!(error.contains("至少需要一个下载地址"), "{error}");
    }

    #[test]
    fn rejects_wrong_mode() {
        let mut value = valid_plan();
        value["mode"] = serde_json::json!("post");

        let error = rejection_error(value);
        assert!(error.contains("mode 必须为 one"), "{error}");
    }

    #[test]
    fn rejects_blank_save_dir() {
        let mut value = valid_plan();
        value["save_dir"] = serde_json::json!("  ");

        let error = rejection_error(value);
        assert!(error.contains("save_dir 不能为空"), "{error}");
    }

    #[test]
    fn rejects_total_that_does_not_match_items() {
        let mut value = valid_plan();
        value["total"] = serde_json::json!(2);

        let error = rejection_error(value);
        assert!(error.contains("total 与 items 数量不一致"), "{error}");
    }

    #[test]
    fn rejects_metadata_for_another_aweme() {
        let mut value = valid_plan();
        value["items"][0]["metadata"]["aweme_id"] = serde_json::json!("other");

        let error = rejection_error(value);
        assert!(error.contains("metadata 标识不一致"), "{error}");
    }

    #[test]
    fn rejects_unknown_contract_fields() {
        let mut value = valid_plan();
        value["unexpected"] = serde_json::json!("not part of V1");

        let error = rejection_error(value);
        assert!(error.contains("unknown field `unexpected`"), "{error}");
    }

    // ============================================================
    // PagedDownloadPlanV1 tests
    // ============================================================

    use super::PagedDownloadPlanV1;

    fn paged_rejection_error(value: serde_json::Value) -> String {
        match PagedDownloadPlanV1::from_value_for_mode(value, "post") {
            Ok(_) => panic!("invalid paged contract must be rejected"),
            Err(error) => error,
        }
    }

    fn valid_paged_plan() -> serde_json::Value {
        serde_json::json!({
            "success": true,
            "contract_version": 1,
            "mode": "post",
            "save_dir": "/tmp/paged",
            "items": [{
                "media_key": "post-1:video:0",
                "aweme_id": "post-1",
                "urls": ["https://cdn.example/post1.mp4"],
                "kind": "video",
                "output": {
                    "filename": "post-1_video",
                    "suffix": ".mp4",
                    "folder_name": null
                },
                "headers": {},
                "accessories": [],
                "metadata": {
                    "aweme_id": "post-1",
                    "desc": "paged test",
                    "author_nickname": "tester",
                    "author_sec_uid": "MS4w.secret"
                }
            }],
            "next_cursor": 100,
            "has_more": true,
            "page_aweme_ids": ["post-1"]
        })
    }

    #[test]
    fn accepts_valid_paged_contract() {
        let plan = PagedDownloadPlanV1::from_value_for_mode(valid_paged_plan(), "post").unwrap();
        assert_eq!(plan.mode, "post");
        assert!(plan.has_more);
        assert_eq!(plan.next_cursor, Some(100));
    }

    #[test]
    fn rejects_paged_without_next_cursor_when_has_more() {
        let mut value = valid_paged_plan();
        value["next_cursor"] = serde_json::json!(null);

        let error = paged_rejection_error(value);
        assert!(error.contains("has_more=true 但 next_cursor 为空"), "{error}");
    }

    #[test]
    fn rejects_paged_duplicate_media_key() {
        let mut value = valid_paged_plan();
        let item = value["items"][0].clone();
        value["items"].as_array_mut().unwrap().push(item);

        let error = paged_rejection_error(value);
        assert!(error.contains("重复 media_key"), "{error}");
    }

    #[test]
    fn rejects_paged_item_not_in_page_aweme_ids() {
        let mut value = valid_paged_plan();
        value["page_aweme_ids"] = serde_json::json!(["other-aweme"]);

        let error = paged_rejection_error(value);
        assert!(error.contains("不在 page_aweme_ids 中"), "{error}");
    }

    #[test]
    fn rejects_paged_aweme_id_without_media_item() {
        let mut value = valid_paged_plan();
        value["page_aweme_ids"] = serde_json::json!(["post-1", "missing-media"]);

        let error = paged_rejection_error(value);
        assert!(error.contains("没有媒体项"), "{error}");
    }

    #[test]
    fn rejects_paged_unknown_mode() {
        let mut value = valid_paged_plan();
        value["mode"] = serde_json::json!("unknown");

        let error = paged_rejection_error(value);
        assert!(error.contains("mode 不匹配"), "{error}");
    }

    #[test]
    fn rejects_paged_unsupported_contract_version() {
        let mut value = valid_paged_plan();
        value["contract_version"] = serde_json::json!(2);

        let error = paged_rejection_error(value);
        assert!(error.contains("不支持的分页解析契约版本"), "{error}");
    }
}
