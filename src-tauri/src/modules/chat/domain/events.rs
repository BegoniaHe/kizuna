use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::{Emotion, MessageId, SessionId};

/// 领域事件基础 trait
pub trait DomainEvent: Clone + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn timestamp(&self) -> DateTime<Utc>;
}

/// 消息发送事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSentEvent {
    pub session_id: SessionId,
    pub message_id: MessageId,
    pub content: String,
    pub is_user: bool,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for MessageSentEvent {
    fn event_type(&self) -> &'static str {
        "message.sent"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// 消息块接收事件（流式响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageChunkEvent {
    pub session_id: SessionId,
    pub content: String,
    pub tokens: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for MessageChunkEvent {
    fn event_type(&self) -> &'static str {
        "message.chunk"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// 消息完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageCompleteEvent {
    pub session_id: SessionId,
    pub message_id: MessageId,
    pub content: String,
    pub emotion: Option<Emotion>,
    pub total_tokens: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for MessageCompleteEvent {
    fn event_type(&self) -> &'static str {
        "message.complete"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// 情感检测事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmotionDetectedEvent {
    pub session_id: SessionId,
    pub emotion: Emotion,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for EmotionDetectedEvent {
    fn event_type(&self) -> &'static str {
        "emotion.detected"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// 会话创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCreatedEvent {
    pub session_id: SessionId,
    pub title: String,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for SessionCreatedEvent {
    fn event_type(&self) -> &'static str {
        "session.created"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// 会话删除事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDeletedEvent {
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for SessionDeletedEvent {
    fn event_type(&self) -> &'static str {
        "session.deleted"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

/// 聊天领域事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatDomainEvent {
    MessageSent(MessageSentEvent),
    MessageChunk(MessageChunkEvent),
    MessageComplete(MessageCompleteEvent),
    EmotionDetected(EmotionDetectedEvent),
    SessionCreated(SessionCreatedEvent),
    SessionDeleted(SessionDeletedEvent),
}

impl ChatDomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            ChatDomainEvent::MessageSent(e) => e.event_type(),
            ChatDomainEvent::MessageChunk(e) => e.event_type(),
            ChatDomainEvent::MessageComplete(e) => e.event_type(),
            ChatDomainEvent::EmotionDetected(e) => e.event_type(),
            ChatDomainEvent::SessionCreated(e) => e.event_type(),
            ChatDomainEvent::SessionDeleted(e) => e.event_type(),
        }
    }
}
