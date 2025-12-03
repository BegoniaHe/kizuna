// Tray Port
//
// 托盘管理端口定义

use thiserror::Error;

use crate::modules::tray::domain::{TrayAction, TrayMenuConfig};

/// 托盘错误类型
#[derive(Error, Debug)]
pub enum TrayError {
    #[error("Tray not initialized")]
    NotInitialized,

    #[error("Tray operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid icon path: {0}")]
    InvalidIcon(String),

    #[error("Menu item not found: {0}")]
    MenuItemNotFound(String),
}

/// 托盘端口 - 定义托盘操作抽象
pub trait TrayPort: Send + Sync {
    /// 初始化托盘
    fn initialize(&self, config: &TrayMenuConfig) -> Result<(), TrayError>;

    /// 设置托盘图标
    fn set_icon(&self, icon_path: &str) -> Result<(), TrayError>;

    /// 设置托盘提示文本
    fn set_tooltip(&self, tooltip: &str) -> Result<(), TrayError>;

    /// 显示托盘
    fn show(&self) -> Result<(), TrayError>;

    /// 隐藏托盘
    fn hide(&self) -> Result<(), TrayError>;

    /// 更新菜单
    fn update_menu(&self, config: &TrayMenuConfig) -> Result<(), TrayError>;

    /// 启用/禁用菜单项
    fn set_menu_item_enabled(&self, item_id: &str, enabled: bool) -> Result<(), TrayError>;

    /// 更新菜单项标题
    fn set_menu_item_title(&self, item_id: &str, title: &str) -> Result<(), TrayError>;
}

/// 托盘动作处理器
pub trait TrayActionHandler: Send + Sync {
    /// 处理托盘动作
    fn handle_action(&self, action: TrayAction);
}
