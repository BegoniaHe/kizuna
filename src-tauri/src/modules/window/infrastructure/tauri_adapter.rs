// Tauri Window Adapter
//
// 基于 Tauri 的窗口管理适配器实现

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewWindow};
use tokio::sync::RwLock;

use crate::modules::window::domain::{
    WindowConfig, WindowLabel, WindowMode, WindowPosition, WindowSize, WindowState,
};
use crate::modules::window::ports::{WindowError, WindowModeRegistry, WindowPort};

/// Tauri 窗口适配器
pub struct TauriWindowAdapter {
    app_handle: AppHandle,
    mode_registry: WindowModeRegistry,
    states: Arc<RwLock<HashMap<String, WindowState>>>,
}

impl TauriWindowAdapter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            mode_registry: WindowModeRegistry::new(),
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_mode_registry(app_handle: AppHandle, mode_registry: WindowModeRegistry) -> Self {
        Self {
            app_handle,
            mode_registry,
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取 Tauri 窗口句柄
    fn get_window(&self, label: &WindowLabel) -> Result<WebviewWindow, WindowError> {
        self.app_handle
            .get_webview_window(label.as_str())
            .ok_or_else(|| WindowError::NotFound(label.to_string()))
    }

    /// 从 Tauri 窗口创建状态
    async fn create_state_from_window(
        &self,
        window: &WebviewWindow,
        label: WindowLabel,
        mode: WindowMode,
    ) -> Result<WindowState, WindowError> {
        let size = window
            .outer_size()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        let position = window
            .outer_position()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        let is_visible = window
            .is_visible()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        let is_focused = window
            .is_focused()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        let is_minimized = window
            .is_minimized()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        let is_maximized = window
            .is_maximized()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;

        let state = WindowState {
            label,
            mode,
            is_visible,
            is_focused,
            is_minimized,
            is_maximized,
            current_size: WindowSize::new(size.width, size.height),
            current_position: WindowPosition::new(position.x, position.y),
        };

        Ok(state)
    }
}

#[async_trait]
impl WindowPort for TauriWindowAdapter {
    async fn create(&self, config: WindowConfig) -> Result<WindowState, WindowError> {
        // 检查窗口是否已存在
        if self
            .app_handle
            .get_webview_window(config.label.as_str())
            .is_some()
        {
            return Err(WindowError::AlreadyExists(config.label.to_string()));
        }

        // 获取模式配置
        let mut effective_config = config.clone();
        let mode = effective_config.mode;
        self.mode_registry.apply_mode(&mut effective_config, mode)?;

        // 创建新窗口
        let window = tauri::WebviewWindowBuilder::new(
            &self.app_handle,
            config.label.as_str(),
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title(&config.title)
        .inner_size(
            effective_config.size.width as f64,
            effective_config.size.height as f64,
        )
        .decorations(effective_config.decorations)
        .always_on_top(effective_config.always_on_top)
        .resizable(effective_config.resizable)
        .skip_taskbar(effective_config.skip_taskbar)
        .center()
        .build()
        .map_err(|e| WindowError::OperationFailed(e.to_string()))?;

        // 如果指定了位置，设置窗口位置
        if let Some(position) = config.position {
            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: position.x,
                    y: position.y,
                }))
                .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        }

        let state = self
            .create_state_from_window(&window, config.label.clone(), effective_config.mode)
            .await?;

        let mut states = self.states.write().await;
        states.insert(config.label.to_string(), state.clone());

        Ok(state)
    }

    async fn get_state(&self, label: &WindowLabel) -> Result<Option<WindowState>, WindowError> {
        let states = self.states.read().await;
        Ok(states.get(label.as_str()).cloned())
    }

    async fn list_windows(&self) -> Result<Vec<WindowState>, WindowError> {
        let states = self.states.read().await;
        Ok(states.values().cloned().collect())
    }

    async fn switch_mode(
        &self,
        label: &WindowLabel,
        mode: WindowMode,
    ) -> Result<WindowState, WindowError> {
        let window = self.get_window(label)?;

        // 获取模式配置
        let mut config = WindowConfig::main_window();
        self.mode_registry.apply_mode(&mut config, mode)?;

        // 应用窗口设置
        window
            .set_decorations(config.decorations)
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;

        window
            .set_always_on_top(config.always_on_top)
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;

        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: config.size.width,
                height: config.size.height,
            }))
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;

        // 如果是普通模式，居中窗口
        if mode == WindowMode::Normal {
            window
                .center()
                .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        }

        // 更新状态
        let state = self
            .create_state_from_window(&window, label.clone(), mode)
            .await?;

        let mut states = self.states.write().await;
        states.insert(label.to_string(), state.clone());

        Ok(state)
    }

    async fn set_size(&self, label: &WindowLabel, size: WindowSize) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: size.width,
                height: size.height,
            }))
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn set_position(
        &self,
        label: &WindowLabel,
        position: WindowPosition,
    ) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: position.x,
                y: position.y,
            }))
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn set_always_on_top(
        &self,
        label: &WindowLabel,
        always_on_top: bool,
    ) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn set_decorations(
        &self,
        label: &WindowLabel,
        decorations: bool,
    ) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .set_decorations(decorations)
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn show(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .show()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn hide(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .hide()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn close(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .close()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;

        let mut states = self.states.write().await;
        states.remove(label.as_str());

        Ok(())
    }

    async fn minimize(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .minimize()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn maximize(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .maximize()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn unmaximize(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .unmaximize()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn center(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .center()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn start_dragging(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .start_dragging()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn set_focus(&self, label: &WindowLabel) -> Result<(), WindowError> {
        let window = self.get_window(label)?;
        window
            .set_focus()
            .map_err(|e| WindowError::OperationFailed(e.to_string()))?;
        Ok(())
    }
}
