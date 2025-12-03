use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::infrastructure::AppState;
use crate::modules::config::domain::AppConfig as DomainAppConfig;
use crate::modules::ConfigModule;
use crate::shared::{AppResult, Preset};

// ============================================================================
// 响应 DTOs - 用于前端通信
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigResponse {
    pub general: GeneralConfigResponse,
    pub window: WindowConfigResponse,
    pub shortcuts: ShortcutConfigResponse,
    pub llm: LLMSettingsResponse,
    pub model: ModelConfigResponse,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfigResponse {
    pub language: String,
    pub theme: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfigResponse {
    pub default_mode: String,
    pub pet_mode_size: SizeResponse,
    pub pet_mode_position: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeResponse {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutConfigResponse {
    pub toggle_window: String,
    pub toggle_pet_mode: String,
    pub new_chat: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMSettingsResponse {
    pub default_provider: String,
    pub stream_response: bool,
    pub context_length: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfigResponse {
    pub default_type: String,
    pub auto_load_last: bool,
    pub physics_enabled: bool,
}

// ============================================================================
// 转换实现
// ============================================================================

impl From<DomainAppConfig> for AppConfigResponse {
    fn from(config: DomainAppConfig) -> Self {
        Self {
            general: GeneralConfigResponse {
                language: config.general.language.code().to_string(),
                theme: config.general.theme.as_str().to_string(),
                auto_start: config.general.auto_start,
                minimize_to_tray: config.general.minimize_to_tray,
            },
            window: WindowConfigResponse {
                default_mode: serde_json::to_string(&config.window.default_mode)
                    .unwrap_or_else(|_| "\"normal\"".to_string())
                    .trim_matches('"')
                    .to_string(),
                pet_mode_size: SizeResponse {
                    width: config.window.pet_mode_size.width,
                    height: config.window.pet_mode_size.height,
                },
                pet_mode_position: serde_json::to_string(&config.window.pet_mode_position)
                    .unwrap_or_else(|_| "\"remember\"".to_string())
                    .trim_matches('"')
                    .to_string(),
            },
            shortcuts: ShortcutConfigResponse {
                toggle_window: config.shortcuts.toggle_window.keys().to_string(),
                toggle_pet_mode: config.shortcuts.toggle_pet_mode.keys().to_string(),
                new_chat: config.shortcuts.new_chat.keys().to_string(),
            },
            llm: LLMSettingsResponse {
                default_provider: config.llm.default_provider.clone(),
                stream_response: config.llm.stream_response,
                context_length: config.llm.context_length,
            },
            model: ModelConfigResponse {
                default_type: config.model.default_type.clone(),
                auto_load_last: config.model.auto_load_last,
                physics_enabled: config.model.physics_enabled,
            },
        }
    }
}

// ============================================================================
// Config Commands
// ============================================================================

#[tauri::command]
pub async fn config_get_all(
    config_module: State<'_, ConfigModule>,
) -> AppResult<AppConfigResponse> {
    let config = config_module
        .get_all()
        .await
        .map_err(|e| crate::shared::AppError::ConfigError(e.to_string()))?;
    Ok(AppConfigResponse::from(config))
}

#[tauri::command]
pub async fn config_reset(config_module: State<'_, ConfigModule>) -> AppResult<()> {
    config_module
        .reset()
        .await
        .map_err(|e| crate::shared::AppError::ConfigError(e.to_string()))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePresetRequest {
    pub name: String,
    pub system_prompt: String,
    pub avatar: Option<String>,
    pub model_type: Option<String>,
    pub model_path: Option<String>,
}

#[tauri::command]
pub async fn preset_list(state: State<'_, AppState>) -> AppResult<Vec<Preset>> {
    let presets = state.presets.read().await;
    Ok(presets.values().cloned().collect())
}

#[tauri::command]
pub async fn preset_create(
    state: State<'_, AppState>,
    request: CreatePresetRequest,
) -> AppResult<Preset> {
    let mut preset = Preset::new(request.name, request.system_prompt);
    if let Some(avatar) = request.avatar {
        preset.avatar = Some(avatar);
    }
    if let Some(model_type) = request.model_type {
        preset.model_type = model_type;
    }
    if let Some(model_path) = request.model_path {
        preset.model_path = model_path;
    }

    let id = preset.id;
    let mut presets = state.presets.write().await;
    presets.insert(id, preset.clone());

    Ok(preset)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePresetRequest {
    pub id: Uuid,
}

#[tauri::command]
pub async fn preset_delete(
    state: State<'_, AppState>,
    request: DeletePresetRequest,
) -> AppResult<()> {
    let mut presets = state.presets.write().await;
    presets.remove(&request.id);
    Ok(())
}
