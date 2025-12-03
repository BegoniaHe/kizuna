// Tauri Tray Handler
//
// 基于 Tauri 的托盘处理实现

use tauri::{AppHandle, Manager};

use crate::modules::tray::domain::TrayMenuConfig;
use crate::modules::tray::ports::{TrayError, TrayPort};

/// Tauri 托盘处理器
pub struct TauriTrayHandler {
    app_handle: AppHandle,
}

impl TauriTrayHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// 获取主窗口并执行操作
    fn with_main_window<F, R>(&self, f: F) -> Result<R, TrayError>
    where
        F: FnOnce(&tauri::WebviewWindow) -> Result<R, TrayError>,
    {
        let window =
            self.app_handle
                .get_webview_window("main")
                .ok_or(TrayError::OperationFailed(
                    "Main window not found".to_string(),
                ))?;
        f(&window)
    }

    /// 显示主窗口
    pub fn show_window(&self) -> Result<(), TrayError> {
        self.with_main_window(|window| {
            window
                .show()
                .map_err(|e| TrayError::OperationFailed(e.to_string()))?;
            window
                .set_focus()
                .map_err(|e| TrayError::OperationFailed(e.to_string()))?;
            Ok(())
        })
    }

    /// 隐藏主窗口
    pub fn hide_window(&self) -> Result<(), TrayError> {
        self.with_main_window(|window| {
            window
                .hide()
                .map_err(|e| TrayError::OperationFailed(e.to_string()))?;
            Ok(())
        })
    }

    /// 切换窗口显示/隐藏
    pub fn toggle_window(&self) -> Result<(), TrayError> {
        self.with_main_window(|window| {
            let is_visible = window
                .is_visible()
                .map_err(|e| TrayError::OperationFailed(e.to_string()))?;

            if is_visible {
                window
                    .hide()
                    .map_err(|e| TrayError::OperationFailed(e.to_string()))?;
            } else {
                window
                    .show()
                    .map_err(|e| TrayError::OperationFailed(e.to_string()))?;
                window
                    .set_focus()
                    .map_err(|e| TrayError::OperationFailed(e.to_string()))?;
            }
            Ok(())
        })
    }

    /// 退出应用
    pub fn quit(&self) {
        self.app_handle.exit(0);
    }
}

impl TrayPort for TauriTrayHandler {
    fn initialize(&self, _config: &TrayMenuConfig) -> Result<(), TrayError> {
        // Tauri 2.0 托盘在 setup 中初始化
        // 这里主要用于更新菜单
        Ok(())
    }

    fn set_icon(&self, _icon_path: &str) -> Result<(), TrayError> {
        // Tauri 2.0 使用不同的 API
        // 需要通过 tray.set_icon() 设置
        // TODO: 实现动态图标更新
        Ok(())
    }

    fn set_tooltip(&self, _tooltip: &str) -> Result<(), TrayError> {
        // TODO: 实现 tooltip 更新
        Ok(())
    }

    fn show(&self) -> Result<(), TrayError> {
        // TODO: 显示托盘
        Ok(())
    }

    fn hide(&self) -> Result<(), TrayError> {
        // TODO: 隐藏托盘
        Ok(())
    }

    fn update_menu(&self, _config: &TrayMenuConfig) -> Result<(), TrayError> {
        // TODO: 动态更新菜单
        Ok(())
    }

    fn set_menu_item_enabled(&self, _item_id: &str, _enabled: bool) -> Result<(), TrayError> {
        // TODO: 更新菜单项状态
        Ok(())
    }

    fn set_menu_item_title(&self, _item_id: &str, _title: &str) -> Result<(), TrayError> {
        // TODO: 更新菜单项标题
        Ok(())
    }
}
