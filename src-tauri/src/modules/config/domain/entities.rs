// Config Domain Entities
//
// 配置领域实体定义

use serde::{Deserialize, Serialize};

use super::value_objects::{Language, PositionStrategy, Shortcut, Size, Theme, WindowModeConfig};

/// 通用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub language: Language,
    pub theme: Theme,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            language: Language::default(),
            theme: Theme::default(),
            auto_start: false,
            minimize_to_tray: true,
        }
    }
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowConfig {
    pub default_mode: WindowModeConfig,
    pub pet_mode_size: Size,
    pub pet_mode_position: PositionStrategy,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            default_mode: WindowModeConfig::default(),
            pet_mode_size: Size::new(300, 400),
            pet_mode_position: PositionStrategy::default(),
        }
    }
}

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutConfig {
    pub toggle_window: Shortcut,
    pub toggle_pet_mode: Shortcut,
    pub new_chat: Shortcut,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            toggle_window: Shortcut::new("CommandOrControl+Shift+K"),
            toggle_pet_mode: Shortcut::new("CommandOrControl+Shift+P"),
            new_chat: Shortcut::new("CommandOrControl+Shift+N"),
        }
    }
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfig {
    pub default_provider: String,
    pub stream_response: bool,
    pub context_length: u32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            default_provider: String::new(),
            stream_response: true,
            context_length: 10,
        }
    }
}

/// LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub is_default: bool,
}

impl LLMProviderConfig {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider_type: provider_type.into(),
            base_url: String::new(),
            api_key: String::new(),
            models: Vec::new(),
            is_default: false,
        }
    }
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    pub default_type: String,
    pub auto_load_last: bool,
    pub physics_enabled: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default_type: "live2d".to_string(),
            auto_load_last: true,
            physics_enabled: true,
        }
    }
}

/// 应用配置聚合根
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub window: WindowConfig,
    pub shortcuts: ShortcutConfig,
    pub llm: LLMConfig,
    pub model: ModelConfig,
}

impl AppConfig {
    /// 创建新的默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 合并部分配置更新
    pub fn merge(&mut self, partial: PartialAppConfig) {
        if let Some(general) = partial.general {
            if let Some(language) = general.language {
                self.general.language = language;
            }
            if let Some(theme) = general.theme {
                self.general.theme = theme;
            }
            if let Some(auto_start) = general.auto_start {
                self.general.auto_start = auto_start;
            }
            if let Some(minimize_to_tray) = general.minimize_to_tray {
                self.general.minimize_to_tray = minimize_to_tray;
            }
        }

        if let Some(llm) = partial.llm {
            if let Some(default_provider) = llm.default_provider {
                self.llm.default_provider = default_provider;
            }
            if let Some(stream_response) = llm.stream_response {
                self.llm.stream_response = stream_response;
            }
            if let Some(context_length) = llm.context_length {
                self.llm.context_length = context_length;
            }
        }

        if let Some(model) = partial.model {
            if let Some(default_type) = model.default_type {
                self.model.default_type = default_type;
            }
            if let Some(auto_load_last) = model.auto_load_last {
                self.model.auto_load_last = auto_load_last;
            }
            if let Some(physics_enabled) = model.physics_enabled {
                self.model.physics_enabled = physics_enabled;
            }
        }
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // 验证快捷键
        if !self.shortcuts.toggle_window.is_valid() {
            errors.push("Invalid toggle window shortcut".to_string());
        }
        if !self.shortcuts.toggle_pet_mode.is_valid() {
            errors.push("Invalid toggle pet mode shortcut".to_string());
        }

        // 验证上下文长度
        if self.llm.context_length == 0 || self.llm.context_length > 100 {
            errors.push("Context length must be between 1 and 100".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 部分配置更新（用于合并）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartialAppConfig {
    pub general: Option<PartialGeneralConfig>,
    pub llm: Option<PartialLLMConfig>,
    pub model: Option<PartialModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartialGeneralConfig {
    pub language: Option<Language>,
    pub theme: Option<Theme>,
    pub auto_start: Option<bool>,
    pub minimize_to_tray: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartialLLMConfig {
    pub default_provider: Option<String>,
    pub stream_response: Option<bool>,
    pub context_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartialModelConfig {
    pub default_type: Option<String>,
    pub auto_load_last: Option<bool>,
    pub physics_enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.general.language.code(), "zh-CN");
        assert_eq!(config.general.theme, Theme::System);
        assert!(!config.general.auto_start);
    }

    #[test]
    fn test_app_config_merge() {
        let mut config = AppConfig::default();
        let partial = PartialAppConfig {
            general: Some(PartialGeneralConfig {
                theme: Some(Theme::Dark),
                ..Default::default()
            }),
            ..Default::default()
        };

        config.merge(partial);
        assert_eq!(config.general.theme, Theme::Dark);
        // 其他字段保持不变
        assert_eq!(config.general.language.code(), "zh-CN");
    }

    #[test]
    fn test_app_config_validate() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = AppConfig::default();
        invalid_config.llm.context_length = 0;
        assert!(invalid_config.validate().is_err());
    }
}
