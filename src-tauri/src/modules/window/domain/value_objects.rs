// Window Value Objects
//
// 窗口相关的值对象定义

use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// 窗口模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WindowMode {
    #[default]
    Normal,
    Pet,
    Compact,
    Fullscreen,
}

impl WindowMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            WindowMode::Normal => "normal",
            WindowMode::Pet => "pet",
            WindowMode::Compact => "compact",
            WindowMode::Fullscreen => "fullscreen",
        }
    }

    pub fn is_pet_mode(&self) -> bool {
        matches!(self, WindowMode::Pet)
    }

    pub fn requires_decorations(&self) -> bool {
        matches!(self, WindowMode::Normal | WindowMode::Fullscreen)
    }

    pub fn requires_always_on_top(&self) -> bool {
        matches!(self, WindowMode::Pet | WindowMode::Compact)
    }
}

impl From<&str> for WindowMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pet" => WindowMode::Pet,
            "compact" => WindowMode::Compact,
            "fullscreen" => WindowMode::Fullscreen,
            _ => WindowMode::Normal,
        }
    }
}

/// 窗口尺寸
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// 确保尺寸在有效范围内
    pub fn clamp(&self, min: WindowSize, max: WindowSize) -> Self {
        Self {
            width: self.width.clamp(min.width, max.width),
            height: self.height.clamp(min.height, max.height),
        }
    }
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
        }
    }
}

/// 窗口位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

impl WindowPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Default for WindowPosition {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// 窗口标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowLabel(String);

impl WindowLabel {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 主窗口标识
    pub fn main() -> Self {
        Self("main".to_string())
    }

    /// 设置窗口标识
    pub fn settings() -> Self {
        Self("settings".to_string())
    }
}

impl Default for WindowLabel {
    fn default() -> Self {
        Self::main()
    }
}

impl From<&str> for WindowLabel {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for WindowLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_mode() {
        assert_eq!(WindowMode::from("pet"), WindowMode::Pet);
        assert_eq!(WindowMode::from("normal"), WindowMode::Normal);
        assert!(WindowMode::Pet.is_pet_mode());
        assert!(!WindowMode::Normal.is_pet_mode());
    }

    #[test]
    fn test_window_size_clamp() {
        let size = WindowSize::new(100, 100);
        let min = WindowSize::new(200, 200);
        let max = WindowSize::new(1000, 1000);

        let clamped = size.clamp(min, max);
        assert_eq!(clamped.width, 200);
        assert_eq!(clamped.height, 200);
    }
}
