use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, WebviewWindow};
use tokio::sync::RwLock;

use crate::infrastructure::{AppEvent, EventBus};
use crate::modules::window::{WindowConfig, WindowLabel, WindowMode, WindowState};
use crate::modules::WindowModule;
use crate::shared::{AppError, AppResult, WindowMode as SharedWindowMode};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TogglePetModeResponse {
    pub is_pet_mode: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAlwaysOnTopRequest {
    pub value: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWindowRequest {
    pub label: String,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub label: String,
    pub mode: String,
    pub is_visible: bool,
    pub is_focused: bool,
    pub width: u32,
    pub height: u32,
}

impl From<WindowState> for WindowInfo {
    fn from(state: WindowState) -> Self {
        Self {
            label: state.label.to_string(),
            mode: format!("{:?}", state.mode).to_lowercase(),
            is_visible: state.is_visible,
            is_focused: state.is_focused,
            width: state.current_size.width,
            height: state.current_size.height,
        }
    }
}

#[tauri::command]
pub async fn window_create(
    window_module: State<'_, WindowModule>,
    request: CreateWindowRequest,
) -> AppResult<WindowInfo> {
    let mode = match request.mode.as_deref() {
        Some("pet") => WindowMode::Pet,
        Some("compact") => WindowMode::Compact,
        _ => WindowMode::Normal,
    };

    let config = WindowConfig {
        label: WindowLabel::new(request.label),
        title: request.title.unwrap_or_else(|| "Kizuna".to_string()),
        mode,
        size: crate::modules::window::WindowSize::new(
            request.width.unwrap_or(800),
            request.height.unwrap_or(600),
        ),
        position: None,
        decorations: mode == WindowMode::Normal,
        always_on_top: mode == WindowMode::Pet,
        transparent: mode == WindowMode::Pet,
        resizable: mode != WindowMode::Pet,
        skip_taskbar: mode == WindowMode::Pet,
        visible: true,
    };

    let state = window_module
        .create_window(config)
        .await
        .map_err(|e| AppError::WindowError(e.to_string()))?;

    Ok(state.into())
}

#[tauri::command]
pub async fn window_list(window_module: State<'_, WindowModule>) -> AppResult<Vec<WindowInfo>> {
    let states = window_module
        .list_windows()
        .await
        .map_err(|e| AppError::WindowError(e.to_string()))?;

    Ok(states.into_iter().map(WindowInfo::from).collect())
}

#[tauri::command]
pub async fn window_close(window_module: State<'_, WindowModule>, label: String) -> AppResult<()> {
    window_module
        .close_window(&WindowLabel::new(label))
        .await
        .map_err(|e| AppError::WindowError(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn window_toggle_pet_mode(
    window: WebviewWindow,
    window_module: State<'_, WindowModule>,
    event_bus: State<'_, Arc<RwLock<EventBus>>>,
) -> AppResult<TogglePetModeResponse> {
    let is_decorated = window
        .is_decorated()
        .map_err(|e| AppError::WindowError(e.to_string()))?;

    if is_decorated {
        // 切换到桌面宠物模式
        window_module
            .switch_to_pet_mode()
            .await
            .map_err(|e| AppError::WindowError(e.to_string()))?;

        let event_bus = event_bus.read().await;
        event_bus.publish(AppEvent::WindowModeChanged {
            mode: SharedWindowMode::Pet,
        });

        Ok(TogglePetModeResponse { is_pet_mode: true })
    } else {
        // 切换到普通模式
        window_module
            .switch_to_normal_mode()
            .await
            .map_err(|e| AppError::WindowError(e.to_string()))?;

        let event_bus = event_bus.read().await;
        event_bus.publish(AppEvent::WindowModeChanged {
            mode: SharedWindowMode::Normal,
        });

        Ok(TogglePetModeResponse { is_pet_mode: false })
    }
}

#[tauri::command]
pub async fn window_set_always_on_top(
    window_module: State<'_, WindowModule>,
    request: SetAlwaysOnTopRequest,
) -> AppResult<()> {
    window_module
        .toggle_always_on_top(&WindowLabel::main(), request.value)
        .await
        .map_err(|e| AppError::WindowError(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn window_start_dragging(window_module: State<'_, WindowModule>) -> AppResult<()> {
    window_module
        .start_dragging(&WindowLabel::main())
        .await
        .map_err(|e| AppError::WindowError(e.to_string()))?;
    Ok(())
}
