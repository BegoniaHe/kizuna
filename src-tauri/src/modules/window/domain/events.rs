// Window Domain Events
//
// 窗口领域事件定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::value_objects::{WindowLabel, WindowMode, WindowPosition, WindowSize};

/// 窗口模式变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowModeChangedEvent {
    pub label: WindowLabel,
    pub old_mode: WindowMode,
    pub new_mode: WindowMode,
    pub timestamp: DateTime<Utc>,
}

impl WindowModeChangedEvent {
    pub fn new(label: WindowLabel, old_mode: WindowMode, new_mode: WindowMode) -> Self {
        Self {
            label,
            old_mode,
            new_mode,
            timestamp: Utc::now(),
        }
    }
}

/// 窗口尺寸变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowResizedEvent {
    pub label: WindowLabel,
    pub old_size: WindowSize,
    pub new_size: WindowSize,
    pub timestamp: DateTime<Utc>,
}

impl WindowResizedEvent {
    pub fn new(label: WindowLabel, old_size: WindowSize, new_size: WindowSize) -> Self {
        Self {
            label,
            old_size,
            new_size,
            timestamp: Utc::now(),
        }
    }
}

/// 窗口位置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowMovedEvent {
    pub label: WindowLabel,
    pub old_position: WindowPosition,
    pub new_position: WindowPosition,
    pub timestamp: DateTime<Utc>,
}

impl WindowMovedEvent {
    pub fn new(
        label: WindowLabel,
        old_position: WindowPosition,
        new_position: WindowPosition,
    ) -> Self {
        Self {
            label,
            old_position,
            new_position,
            timestamp: Utc::now(),
        }
    }
}

/// 窗口焦点变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowFocusChangedEvent {
    pub label: WindowLabel,
    pub is_focused: bool,
    pub timestamp: DateTime<Utc>,
}

impl WindowFocusChangedEvent {
    pub fn new(label: WindowLabel, is_focused: bool) -> Self {
        Self {
            label,
            is_focused,
            timestamp: Utc::now(),
        }
    }
}

/// 窗口可见性变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowVisibilityChangedEvent {
    pub label: WindowLabel,
    pub is_visible: bool,
    pub timestamp: DateTime<Utc>,
}

impl WindowVisibilityChangedEvent {
    pub fn new(label: WindowLabel, is_visible: bool) -> Self {
        Self {
            label,
            is_visible,
            timestamp: Utc::now(),
        }
    }
}

/// 窗口创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowCreatedEvent {
    pub label: WindowLabel,
    pub mode: WindowMode,
    pub timestamp: DateTime<Utc>,
}

impl WindowCreatedEvent {
    pub fn new(label: WindowLabel, mode: WindowMode) -> Self {
        Self {
            label,
            mode,
            timestamp: Utc::now(),
        }
    }
}

/// 窗口关闭事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowClosedEvent {
    pub label: WindowLabel,
    pub timestamp: DateTime<Utc>,
}

impl WindowClosedEvent {
    pub fn new(label: WindowLabel) -> Self {
        Self {
            label,
            timestamp: Utc::now(),
        }
    }
}
