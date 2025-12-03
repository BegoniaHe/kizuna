// Tray Domain Entities
//
// 托盘领域实体定义

use serde::{Deserialize, Serialize};

/// 托盘菜单项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayMenuItem {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub shortcut: Option<String>,
}

impl TrayMenuItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            enabled: true,
            shortcut: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// 托盘菜单分隔符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayMenuSeparator;

/// 托盘菜单元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrayMenuElement {
    Item(TrayMenuItem),
    Separator,
}

/// 托盘菜单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayMenuConfig {
    pub items: Vec<TrayMenuElement>,
}

impl TrayMenuConfig {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(mut self, item: TrayMenuItem) -> Self {
        self.items.push(TrayMenuElement::Item(item));
        self
    }

    pub fn add_separator(mut self) -> Self {
        self.items.push(TrayMenuElement::Separator);
        self
    }
}

impl Default for TrayMenuConfig {
    fn default() -> Self {
        Self::new()
            .add_item(TrayMenuItem::new("show", "显示窗口").with_shortcut("Cmd+Shift+K"))
            .add_item(TrayMenuItem::new("pet_mode", "桌面宠物模式").with_shortcut("Cmd+Shift+P"))
            .add_separator()
            .add_item(TrayMenuItem::new("settings", "设置"))
            .add_separator()
            .add_item(TrayMenuItem::new("quit", "退出").with_shortcut("Cmd+Q"))
    }
}

/// 托盘配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayConfig {
    pub icon_path: String,
    pub tooltip: String,
    pub menu: TrayMenuConfig,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            icon_path: "icons/icon.png".to_string(),
            tooltip: "Kizuna".to_string(),
            menu: TrayMenuConfig::default(),
        }
    }
}
