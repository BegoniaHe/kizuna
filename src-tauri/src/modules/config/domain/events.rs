// Config Domain Events
//
// 配置领域事件定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::entities::AppConfig;
use super::value_objects::Theme;

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangedEvent {
    pub key: String,
    pub timestamp: DateTime<Utc>,
}

impl ConfigChangedEvent {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            timestamp: Utc::now(),
        }
    }
}

/// 主题变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeChangedEvent {
    pub old_theme: Theme,
    pub new_theme: Theme,
    pub timestamp: DateTime<Utc>,
}

impl ThemeChangedEvent {
    pub fn new(old_theme: Theme, new_theme: Theme) -> Self {
        Self {
            old_theme,
            new_theme,
            timestamp: Utc::now(),
        }
    }
}

/// 配置重置事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResetEvent {
    pub config: AppConfig,
    pub timestamp: DateTime<Utc>,
}

impl ConfigResetEvent {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            timestamp: Utc::now(),
        }
    }
}

/// 配置加载事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigLoadedEvent {
    pub source: ConfigSource,
    pub timestamp: DateTime<Utc>,
}

/// 配置来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigSource {
    Default,
    Store,
    File(String),
}

impl ConfigLoadedEvent {
    pub fn new(source: ConfigSource) -> Self {
        Self {
            source,
            timestamp: Utc::now(),
        }
    }
}
