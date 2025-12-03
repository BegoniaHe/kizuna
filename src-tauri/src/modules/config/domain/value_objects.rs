// Config Value Objects
//
// 配置相关的值对象定义

use serde::{Deserialize, Serialize};

/// 主题类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

impl From<&str> for Theme {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }
}

/// 语言类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language(String);

impl Language {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn code(&self) -> &str {
        &self.0
    }
}

impl Default for Language {
    fn default() -> Self {
        Self("zh-CN".to_string())
    }
}

impl From<String> for Language {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 快捷键定义
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut(String);

impl Shortcut {
    pub fn new(keys: impl Into<String>) -> Self {
        Self(keys.into())
    }

    pub fn keys(&self) -> &str {
        &self.0
    }

    /// 验证快捷键格式是否有效
    pub fn is_valid(&self) -> bool {
        // 简单验证：非空且包含 + 或是单个按键
        !self.0.is_empty() && (self.0.contains('+') || self.0.len() <= 5)
    }
}

impl Default for Shortcut {
    fn default() -> Self {
        Self(String::new())
    }
}

/// 窗口模式配置键
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowModeConfig {
    Normal,
    Pet,
    Compact,
}

impl Default for WindowModeConfig {
    fn default() -> Self {
        WindowModeConfig::Normal
    }
}

/// 位置记忆策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PositionStrategy {
    Remember,
    Center,
    TopRight,
    BottomRight,
}

impl Default for PositionStrategy {
    fn default() -> Self {
        PositionStrategy::Remember
    }
}

/// 尺寸配置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self {
            width: 300,
            height: 400,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_str() {
        assert_eq!(Theme::from("light"), Theme::Light);
        assert_eq!(Theme::from("dark"), Theme::Dark);
        assert_eq!(Theme::from("system"), Theme::System);
        assert_eq!(Theme::from("invalid"), Theme::System);
    }

    #[test]
    fn test_shortcut_validation() {
        assert!(Shortcut::new("CommandOrControl+Shift+K").is_valid());
        assert!(Shortcut::new("F1").is_valid());
        assert!(!Shortcut::new("").is_valid());
    }
}
