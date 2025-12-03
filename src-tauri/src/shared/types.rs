use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: Uuid,
    pub title: String,
    pub preset_id: Option<Uuid>,
    pub model_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(title: Option<String>, preset_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.unwrap_or_else(|| "新对话".to_string()),
            preset_id,
            model_config: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Thinking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub tokens: Option<u32>,
    pub emotion: Option<Emotion>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new_user(session_id: Uuid, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role: MessageRole::User,
            content,
            tokens: None,
            emotion: None,
            created_at: Utc::now(),
        }
    }

    pub fn new_assistant(session_id: Uuid, content: String, emotion: Option<Emotion>) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role: MessageRole::Assistant,
            content,
            tokens: None,
            emotion,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageChunk {
    pub session_id: Uuid,
    pub content: String,
    pub tokens: Option<u32>,
    /// 口型音素序列 (A/E/I/O/U/N/closed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phonemes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode {
    Normal,
    Pet,
    Compact,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub system_prompt: String,
    pub model_type: String,
    pub model_path: String,
    pub default_expression: String,
    pub emotion_mapping: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Preset {
    pub fn new(name: String, system_prompt: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            avatar: None,
            system_prompt,
            model_type: "live2d".to_string(),
            model_path: String::new(),
            default_expression: "neutral".to_string(),
            emotion_mapping: serde_json::json!({}),
            created_at: Utc::now(),
        }
    }
}
