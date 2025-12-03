// Tray Domain Events
//
// 托盘领域事件定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 托盘点击事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayClickEvent {
    pub button: TrayMouseButton,
    pub timestamp: DateTime<Utc>,
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrayMouseButton {
    Left,
    Right,
    Middle,
}

impl TrayClickEvent {
    pub fn left_click() -> Self {
        Self {
            button: TrayMouseButton::Left,
            timestamp: Utc::now(),
        }
    }

    pub fn right_click() -> Self {
        Self {
            button: TrayMouseButton::Right,
            timestamp: Utc::now(),
        }
    }
}

/// 托盘菜单点击事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayMenuClickEvent {
    pub item_id: String,
    pub timestamp: DateTime<Utc>,
}

impl TrayMenuClickEvent {
    pub fn new(item_id: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            timestamp: Utc::now(),
        }
    }
}

/// 托盘动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrayAction {
    ShowWindow,
    HideWindow,
    ToggleWindow,
    TogglePetMode,
    OpenSettings,
    Quit,
    Custom(String),
}

impl From<&str> for TrayAction {
    fn from(s: &str) -> Self {
        match s {
            "show" | "show_window" => TrayAction::ShowWindow,
            "hide" | "hide_window" => TrayAction::HideWindow,
            "toggle" | "toggle_window" => TrayAction::ToggleWindow,
            "pet_mode" | "toggle_pet_mode" => TrayAction::TogglePetMode,
            "settings" | "open_settings" => TrayAction::OpenSettings,
            "quit" | "exit" => TrayAction::Quit,
            other => TrayAction::Custom(other.to_string()),
        }
    }
}
