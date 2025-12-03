// Window Domain Entities
//
// 窗口领域实体定义

use serde::{Deserialize, Serialize};

use super::value_objects::{WindowLabel, WindowMode, WindowPosition, WindowSize};

/// 窗口配置实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
    pub label: WindowLabel,
    pub title: String,
    pub mode: WindowMode,
    pub size: WindowSize,
    pub position: Option<WindowPosition>,
    pub always_on_top: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub skip_taskbar: bool,
    pub resizable: bool,
    pub visible: bool,
}

impl WindowConfig {
    /// 创建默认主窗口配置
    pub fn main_window() -> Self {
        Self {
            label: WindowLabel::main(),
            title: "Kizuna".to_string(),
            mode: WindowMode::Normal,
            size: WindowSize::new(1200, 800),
            position: None,
            always_on_top: false,
            decorations: true,
            transparent: false,
            skip_taskbar: false,
            resizable: true,
            visible: true,
        }
    }

    /// 创建桌面宠物模式配置
    pub fn pet_mode() -> Self {
        Self {
            label: WindowLabel::main(),
            title: "Kizuna".to_string(),
            mode: WindowMode::Pet,
            size: WindowSize::new(300, 400),
            position: None,
            always_on_top: true,
            decorations: false,
            transparent: true,
            skip_taskbar: true,
            resizable: false,
            visible: true,
        }
    }

    /// 创建紧凑模式配置
    pub fn compact_mode() -> Self {
        Self {
            label: WindowLabel::main(),
            title: "Kizuna".to_string(),
            mode: WindowMode::Compact,
            size: WindowSize::new(400, 600),
            position: None,
            always_on_top: true,
            decorations: false,
            transparent: false,
            skip_taskbar: false,
            resizable: true,
            visible: true,
        }
    }

    /// 应用模式预设
    pub fn apply_mode(&mut self, mode: WindowMode) {
        self.mode = mode;

        match mode {
            WindowMode::Normal => {
                self.decorations = true;
                self.always_on_top = false;
                self.transparent = false;
                self.skip_taskbar = false;
                self.resizable = true;
            }
            WindowMode::Pet => {
                self.decorations = false;
                self.always_on_top = true;
                self.transparent = true;
                self.skip_taskbar = true;
                self.resizable = false;
            }
            WindowMode::Compact => {
                self.decorations = false;
                self.always_on_top = true;
                self.transparent = false;
                self.skip_taskbar = false;
                self.resizable = true;
            }
            WindowMode::Fullscreen => {
                self.decorations = false;
                self.always_on_top = false;
                self.transparent = false;
                self.skip_taskbar = false;
                self.resizable = false;
            }
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::main_window()
    }
}

/// 窗口状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub label: WindowLabel,
    pub mode: WindowMode,
    pub is_visible: bool,
    pub is_focused: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub current_size: WindowSize,
    pub current_position: WindowPosition,
}

impl WindowState {
    pub fn new(label: WindowLabel) -> Self {
        Self {
            label,
            mode: WindowMode::Normal,
            is_visible: true,
            is_focused: false,
            is_minimized: false,
            is_maximized: false,
            current_size: WindowSize::default(),
            current_position: WindowPosition::default(),
        }
    }
}

/// 模式尺寸配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeSizeConfig {
    pub normal: WindowSize,
    pub pet: WindowSize,
    pub compact: WindowSize,
}

impl Default for ModeSizeConfig {
    fn default() -> Self {
        Self {
            normal: WindowSize::new(1200, 800),
            pet: WindowSize::new(300, 400),
            compact: WindowSize::new(400, 600),
        }
    }
}

impl ModeSizeConfig {
    pub fn get_size(&self, mode: WindowMode) -> WindowSize {
        match mode {
            WindowMode::Normal => self.normal,
            WindowMode::Pet => self.pet,
            WindowMode::Compact => self.compact,
            WindowMode::Fullscreen => self.normal, // 全屏使用系统尺寸
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_modes() {
        let mut config = WindowConfig::main_window();
        assert!(config.decorations);
        assert!(!config.always_on_top);

        config.apply_mode(WindowMode::Pet);
        assert!(!config.decorations);
        assert!(config.always_on_top);
        assert!(config.transparent);
    }

    #[test]
    fn test_pet_mode_config() {
        let config = WindowConfig::pet_mode();
        assert_eq!(config.mode, WindowMode::Pet);
        assert!(!config.decorations);
        assert!(config.always_on_top);
        assert!(config.transparent);
        assert!(config.skip_taskbar);
    }
}
