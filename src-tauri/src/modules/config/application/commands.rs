// Config Commands
//
// 配置相关的命令处理器

use async_trait::async_trait;
use std::sync::Arc;

use crate::modules::config::domain::{AppConfig, PartialAppConfig};
use crate::modules::config::ports::{ConfigError, ConfigRepository};

/// 命令处理器 trait
#[async_trait]
pub trait CommandHandler<C> {
    type Output;
    type Error;

    async fn handle(&self, command: C) -> Result<Self::Output, Self::Error>;
}

// ============================================================================
// Update Config Command
// ============================================================================

/// 更新配置命令
#[derive(Debug, Clone)]
pub struct UpdateConfigCommand {
    pub partial: PartialAppConfig,
}

impl UpdateConfigCommand {
    pub fn new(partial: PartialAppConfig) -> Self {
        Self { partial }
    }
}

/// 更新配置响应
#[derive(Debug, Clone)]
pub struct UpdateConfigResponse {
    pub config: AppConfig,
}

/// 更新配置命令处理器
pub struct UpdateConfigHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl UpdateConfigHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CommandHandler<UpdateConfigCommand> for UpdateConfigHandler {
    type Output = UpdateConfigResponse;
    type Error = ConfigError;

    async fn handle(&self, command: UpdateConfigCommand) -> Result<Self::Output, Self::Error> {
        // 加载当前配置
        let mut config = self.repository.load().await?;

        // 合并更新
        config.merge(command.partial);

        // 验证配置
        config
            .validate()
            .map_err(|errors| ConfigError::ValidationError { errors })?;

        // 保存配置
        self.repository.save(&config).await?;

        Ok(UpdateConfigResponse { config })
    }
}

// ============================================================================
// Reset Config Command
// ============================================================================

/// 重置配置命令
#[derive(Debug, Clone)]
pub struct ResetConfigCommand;

/// 重置配置响应
#[derive(Debug, Clone)]
pub struct ResetConfigResponse {
    pub config: AppConfig,
}

/// 重置配置命令处理器
pub struct ResetConfigHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl ResetConfigHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CommandHandler<ResetConfigCommand> for ResetConfigHandler {
    type Output = ResetConfigResponse;
    type Error = ConfigError;

    async fn handle(&self, _command: ResetConfigCommand) -> Result<Self::Output, Self::Error> {
        // 清除现有配置
        self.repository.clear().await?;

        // 加载默认配置
        let config = self.repository.load().await?;

        Ok(ResetConfigResponse { config })
    }
}

// ============================================================================
// Set Config Value Command
// ============================================================================

/// 设置配置值命令
#[derive(Debug, Clone)]
pub struct SetConfigValueCommand {
    pub key: String,
    pub value: serde_json::Value,
}

impl SetConfigValueCommand {
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            value,
        }
    }
}

/// 设置配置值响应
#[derive(Debug, Clone)]
pub struct SetConfigValueResponse {
    pub success: bool,
}

/// 设置配置值命令处理器
pub struct SetConfigValueHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl SetConfigValueHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CommandHandler<SetConfigValueCommand> for SetConfigValueHandler {
    type Output = SetConfigValueResponse;
    type Error = ConfigError;

    async fn handle(&self, command: SetConfigValueCommand) -> Result<Self::Output, Self::Error> {
        self.repository
            .set_value(&command.key, command.value)
            .await?;

        Ok(SetConfigValueResponse { success: true })
    }
}

// ============================================================================
// Delete Config Value Command
// ============================================================================

/// 删除配置值命令
#[derive(Debug, Clone)]
pub struct DeleteConfigValueCommand {
    pub key: String,
}

impl DeleteConfigValueCommand {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

/// 删除配置值响应
#[derive(Debug, Clone)]
pub struct DeleteConfigValueResponse {
    pub success: bool,
}

/// 删除配置值命令处理器
pub struct DeleteConfigValueHandler {
    repository: Arc<dyn ConfigRepository>,
}

impl DeleteConfigValueHandler {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl CommandHandler<DeleteConfigValueCommand> for DeleteConfigValueHandler {
    type Output = DeleteConfigValueResponse;
    type Error = ConfigError;

    async fn handle(&self, command: DeleteConfigValueCommand) -> Result<Self::Output, Self::Error> {
        self.repository.delete_value(&command.key).await?;

        Ok(DeleteConfigValueResponse { success: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::config::domain::PartialGeneralConfig;
    use crate::modules::config::domain::Theme;
    use crate::modules::config::infrastructure::InMemoryConfigRepository;

    #[tokio::test]
    async fn test_update_config() {
        let repo = Arc::new(InMemoryConfigRepository::new());
        let handler = UpdateConfigHandler::new(repo.clone());

        let command = UpdateConfigCommand::new(PartialAppConfig {
            general: Some(PartialGeneralConfig {
                theme: Some(Theme::Dark),
                ..Default::default()
            }),
            ..Default::default()
        });

        let response = handler.handle(command).await.unwrap();
        assert_eq!(response.config.general.theme, Theme::Dark);
    }

    #[tokio::test]
    async fn test_reset_config() {
        let repo = Arc::new(InMemoryConfigRepository::new());

        // 先更新配置
        let update_handler = UpdateConfigHandler::new(repo.clone());
        update_handler
            .handle(UpdateConfigCommand::new(PartialAppConfig {
                general: Some(PartialGeneralConfig {
                    auto_start: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }))
            .await
            .unwrap();

        // 重置配置
        let reset_handler = ResetConfigHandler::new(repo);
        let response = reset_handler.handle(ResetConfigCommand).await.unwrap();

        assert!(!response.config.general.auto_start);
    }
}
