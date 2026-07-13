//! Versioned Python -> Rust download resolver contracts.

use std::collections::HashMap;

use serde::Deserialize;

use crate::db::VideoInfo;

pub const SINGLE_DOWNLOAD_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SingleMediaKind {
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
pub struct SingleDownloadItem {
    pub aweme_id: String,
    pub urls: Vec<String>,
    pub kind: SingleMediaKind,
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
    pub items: Vec<SingleDownloadItem>,
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
        for item in &self.items {
            if item.aweme_id.trim().is_empty() {
                return Err("单视频解析项 aweme_id 不能为空".to_string());
            }
            if item.urls.is_empty() || item.urls.iter().any(|url| url.trim().is_empty()) {
                return Err(format!(
                    "单视频解析项 {} 至少需要一个下载地址",
                    item.aweme_id
                ));
            }
            if item.output.filename.trim().is_empty() || item.output.suffix.trim().is_empty() {
                return Err(format!("单视频解析项 {} 的 output 不完整", item.aweme_id));
            }
            if !item.output.suffix.starts_with('.') {
                return Err(format!(
                    "单视频解析项 {} 的 output.suffix 必须以 . 开头",
                    item.aweme_id
                ));
            }
            if item
                .output
                .folder_name
                .as_ref()
                .is_some_and(|folder| folder.trim().is_empty())
            {
                return Err(format!(
                    "单视频解析项 {} 的 output.folder_name 不能为空字符串",
                    item.aweme_id
                ));
            }
            if item.metadata.aweme_id != item.aweme_id {
                return Err(format!(
                    "单视频解析项 {} 的 metadata 标识不一致",
                    item.aweme_id
                ));
            }
            for accessory in &item.accessories {
                if accessory.output.filename.trim().is_empty()
                    || accessory.output.suffix.trim().is_empty()
                {
                    return Err(format!(
                        "单视频解析项 {} 的附件 output 不完整",
                        item.aweme_id
                    ));
                }
                if !accessory.output.suffix.starts_with('.') {
                    return Err(format!(
                        "单视频解析项 {} 的附件 output.suffix 必须以 . 开头",
                        item.aweme_id
                    ));
                }
                match accessory.kind {
                    SingleAccessoryKind::Music | SingleAccessoryKind::Cover
                        if accessory
                            .url
                            .as_deref()
                            .is_none_or(|url| url.trim().is_empty()) =>
                    {
                        return Err(format!(
                            "单视频解析项 {} 的音乐或封面附件缺少 URL",
                            item.aweme_id
                        ));
                    }
                    SingleAccessoryKind::Description if accessory.content.as_deref().is_none() => {
                        return Err(format!("单视频解析项 {} 的文案附件缺少内容", item.aweme_id));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

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
}
