use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::super::value_objects::SessionId;
use super::Message;

/// 会话实体 - 聚合根
///
/// Session 是 Chat 模块的聚合根，管理消息集合
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// 会话唯一标识
    id: SessionId,
    /// 会话标题
    title: String,
    /// 关联的预设 ID（可选）
    preset_id: Option<Uuid>,
    /// 模型配置（JSON 格式）
    model_config: Option<serde_json::Value>,
    /// 创建时间
    created_at: DateTime<Utc>,
    /// 更新时间
    updated_at: DateTime<Utc>,
}

impl Session {
    /// 创建新会话
    pub fn new(title: Option<String>, preset_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            title: title.unwrap_or_else(|| "新对话".to_string()),
            preset_id,
            model_config: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 从已有 ID 创建（用于从存储恢复）
    pub fn from_id(id: SessionId, title: String, preset_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            preset_id,
            model_config: None,
            created_at: now,
            updated_at: now,
        }
    }

    // Getters
    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn preset_id(&self) -> Option<Uuid> {
        self.preset_id
    }

    pub fn model_config(&self) -> Option<&serde_json::Value> {
        self.model_config.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // 业务方法

    /// 更新标题
    pub fn update_title(&mut self, new_title: String) {
        self.title = new_title;
        self.touch();
    }

    /// 更新 preset
    pub fn update_preset(&mut self, preset_id: Option<uuid::Uuid>) {
        self.preset_id = preset_id;
        self.touch();
    }

    /// 重命名会话
    pub fn rename(&mut self, new_title: impl Into<String>) {
        self.title = new_title.into();
        self.touch();
    }

    /// 设置模型配置
    pub fn set_model_config(&mut self, config: serde_json::Value) {
        self.model_config = Some(config);
        self.touch();
    }

    /// 更新修改时间
    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// 根据消息内容生成标题（取第一条用户消息的前 20 个字符）
    pub fn generate_title_from_message(message: &Message) -> String {
        let content = message.content();
        let title: String = content.chars().take(20).collect();
        if content.chars().count() > 20 {
            format!("{}...", title)
        } else {
            title
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let session = Session::new(Some("Test Session".to_string()), None);
        assert_eq!(session.title(), "Test Session");
        assert!(session.preset_id().is_none());
    }

    #[test]
    fn test_session_rename() {
        let mut session = Session::default();
        let old_updated_at = session.updated_at();

        // 确保时间差异
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.rename("New Title");
        assert_eq!(session.title(), "New Title");
        assert!(session.updated_at() > old_updated_at);
    }

    #[test]
    fn test_default_session_title() {
        let session = Session::default();
        assert_eq!(session.title(), "新对话");
    }
}
