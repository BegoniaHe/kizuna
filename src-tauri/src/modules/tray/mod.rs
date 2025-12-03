// Tray Module
//
// 系统托盘模块，管理托盘图标和菜单
//
// 功能：
// - 托盘图标显示
// - 托盘菜单（显示/隐藏窗口、退出等）
// - 托盘事件处理

pub mod domain;
pub mod infrastructure;
pub mod ports;

// 重新导出常用类型
pub use domain::*;
pub use infrastructure::*;
pub use ports::*;

use std::sync::Arc;
use tauri::AppHandle;

/// Tray 模块容器
pub struct TrayModule {
    handler: Arc<TauriTrayHandler>,
}

impl TrayModule {
    /// 创建 Tray 模块
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            handler: Arc::new(TauriTrayHandler::new(app_handle)),
        }
    }

    /// 获取托盘处理器
    pub fn handler(&self) -> &Arc<TauriTrayHandler> {
        &self.handler
    }

    /// 设置托盘图标
    pub fn set_icon(&self, icon_path: &str) -> Result<(), TrayError> {
        self.handler.set_icon(icon_path)
    }

    /// 设置托盘提示文本
    pub fn set_tooltip(&self, tooltip: &str) -> Result<(), TrayError> {
        self.handler.set_tooltip(tooltip)
    }

    /// 显示托盘
    pub fn show(&self) -> Result<(), TrayError> {
        self.handler.show()
    }

    /// 隐藏托盘
    pub fn hide(&self) -> Result<(), TrayError> {
        self.handler.hide()
    }
}
