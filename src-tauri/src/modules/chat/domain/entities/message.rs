use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::super::value_objects::{Emotion, MessageId, SessionId};

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// 用户消息
    User,
    /// AI 助手消息
    Assistant,
    /// 系统消息
    System,
}

impl MessageRole {
    /// 转换为 OpenAI 格式的角色名
    pub fn to_openai_role(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }
}

/// 消息实体
///
/// 聚合根的一部分，属于 Session 聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// 消息唯一标识
    id: MessageId,
    /// 所属会话 ID
    session_id: SessionId,
    /// 消息角色
    role: MessageRole,
    /// 消息内容
    content: String,
    /// Token 数量（可选）
    tokens: Option<u32>,
    /// 情感（仅 Assistant 消息）
    emotion: Option<Emotion>,
    /// 创建时间
    created_at: DateTime<Utc>,
}

impl Message {
    /// 创建用户消息
    pub fn new_user(session_id: SessionId, content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            session_id,
            role: MessageRole::User,
            content: content.into(),
            tokens: None,
            emotion: None,
            created_at: Utc::now(),
        }
    }

    /// 创建助手消息
    pub fn new_assistant(
        session_id: SessionId,
        content: impl Into<String>,
        emotion: Option<Emotion>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            session_id,
            role: MessageRole::Assistant,
            content: content.into(),
            tokens: None,
            emotion,
            created_at: Utc::now(),
        }
    }

    /// 创建系统消息
    pub fn new_system(session_id: SessionId, content: impl Into<String>) -> Self {
        Self {
            id: MessageId::new(),
            session_id,
            role: MessageRole::System,
            content: content.into(),
            tokens: None,
            emotion: None,
            created_at: Utc::now(),
        }
    }

    // Getters
    pub fn id(&self) -> MessageId {
        self.id
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn role(&self) -> MessageRole {
        self.role
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn tokens(&self) -> Option<u32> {
        self.tokens
    }

    pub fn emotion(&self) -> Option<Emotion> {
        self.emotion
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // Setters (内部使用)
    pub fn set_id(&mut self, id: MessageId) {
        self.id = id;
    }

    pub fn set_tokens(&mut self, tokens: u32) {
        self.tokens = Some(tokens);
    }

    pub fn set_emotion(&mut self, emotion: Emotion) {
        self.emotion = Some(emotion);
    }

    /// 追加内容（用于流式响应）
    pub fn append_content(&mut self, chunk: &str) {
        self.content.push_str(chunk);
    }

    /// 检测并设置情感
    pub fn detect_emotion(&mut self) {
        if self.role == MessageRole::Assistant {
            self.emotion = Some(Emotion::detect_from_text(&self.content));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_message() {
        let session_id = SessionId::new();
        let msg = Message::new_user(session_id, "Hello, AI!");

        assert_eq!(msg.role(), MessageRole::User);
        assert_eq!(msg.content(), "Hello, AI!");
        assert_eq!(msg.session_id(), session_id);
        assert!(msg.emotion().is_none());
    }

    #[test]
    fn test_create_assistant_message_with_emotion() {
        let session_id = SessionId::new();
        let msg = Message::new_assistant(session_id, "I'm happy to help!", Some(Emotion::Happy));

        assert_eq!(msg.role(), MessageRole::Assistant);
        assert_eq!(msg.emotion(), Some(Emotion::Happy));
    }

    #[test]
    fn test_append_content() {
        let session_id = SessionId::new();
        let mut msg = Message::new_assistant(session_id, "Hello", None);
        msg.append_content(" World!");

        assert_eq!(msg.content(), "Hello World!");
    }
}
