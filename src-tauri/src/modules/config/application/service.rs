// Config Service
//
// 配置服务门面，提供统一的 API

use async_trait::async_trait;
use std::sync::Arc;

use super::{
    CommandHandler, DeleteConfigValueCommand, DeleteConfigValueHandler, GetAllConfigHandler,
    GetAllConfigQuery, GetConfigValueHandler, GetConfigValueQuery, QueryHandler,
    ResetConfigCommand, ResetConfigHandler, SetConfigValueCommand, SetConfigValueHandler,
    UpdateConfigCommand, UpdateConfigHandler,
};
use crate::modules::config::domain::{AppConfig, PartialAppConfig};
use crate::modules::config::ports::{ConfigError, ConfigPort, ConfigRepository};

/// 配置服务实现
pub struct ConfigService {
    repository: Arc<dyn ConfigRepository>,
    // Handlers
    get_all_handler: GetAllConfigHandler,
    get_value_handler: GetConfigValueHandler,
    update_handler: UpdateConfigHandler,
    reset_handler: ResetConfigHandler,
    set_value_handler: SetConfigValueHandler,
    delete_value_handler: DeleteConfigValueHandler,
}

impl ConfigService {
    pub fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self {
            get_all_handler: GetAllConfigHandler::new(repository.clone()),
            get_value_handler: GetConfigValueHandler::new(repository.clone()),
            update_handler: UpdateConfigHandler::new(repository.clone()),
            reset_handler: ResetConfigHandler::new(repository.clone()),
            set_value_handler: SetConfigValueHandler::new(repository.clone()),
            delete_value_handler: DeleteConfigValueHandler::new(repository.clone()),
            repository,
        }
    }

    /// 获取仓储引用
    pub fn repository(&self) -> &Arc<dyn ConfigRepository> {
        &self.repository
    }
}

#[async_trait]
impl ConfigPort for ConfigService {
    async fn get_all(&self) -> Result<AppConfig, ConfigError> {
        let response = self.get_all_handler.handle(GetAllConfigQuery).await?;
        Ok(response.config)
    }

    async fn get<T: serde::de::DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, ConfigError> {
        let response = self
            .get_value_handler
            .handle(GetConfigValueQuery::new(key))
            .await?;

        match response.value {
            Some(value) => {
                let typed_value = serde_json::from_value(value)?;
                Ok(Some(typed_value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), ConfigError> {
        let json_value = serde_json::to_value(value)?;
        self.set_value_handler
            .handle(SetConfigValueCommand::new(key, json_value))
            .await?;
        Ok(())
    }

    async fn update(&self, partial: PartialAppConfig) -> Result<AppConfig, ConfigError> {
        let response = self
            .update_handler
            .handle(UpdateConfigCommand::new(partial))
            .await?;
        Ok(response.config)
    }

    async fn delete(&self, key: &str) -> Result<(), ConfigError> {
        self.delete_value_handler
            .handle(DeleteConfigValueCommand::new(key))
            .await?;
        Ok(())
    }

    async fn reset(&self) -> Result<AppConfig, ConfigError> {
        let response = self.reset_handler.handle(ResetConfigCommand).await?;
        Ok(response.config)
    }

    async fn exists(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self
            .get_value_handler
            .handle(GetConfigValueQuery::new(key))
            .await?;
        Ok(value.value.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::config::domain::{PartialGeneralConfig, Theme};
    use crate::modules::config::infrastructure::InMemoryConfigRepository;

    #[tokio::test]
    async fn test_config_service() {
        let repo = Arc::new(InMemoryConfigRepository::new());
        let service = ConfigService::new(repo);

        // 获取全部配置
        let config = service.get_all().await.unwrap();
        assert_eq!(config.general.theme, Theme::System);

        // 更新配置
        let updated = service
            .update(PartialAppConfig {
                general: Some(PartialGeneralConfig {
                    theme: Some(Theme::Dark),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(updated.general.theme, Theme::Dark);

        // 重置配置
        let reset = service.reset().await.unwrap();
        assert_eq!(reset.general.theme, Theme::System);
    }

    #[tokio::test]
    async fn test_config_service_get_set() {
        let repo = Arc::new(InMemoryConfigRepository::new());
        let service = ConfigService::new(repo);

        // 设置值
        service.set("custom.key", &"test_value").await.unwrap();

        // 获取值
        let value: Option<String> = service.get("custom.key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // 检查存在性
        assert!(service.exists("custom.key").await.unwrap());

        // 删除值
        service.delete("custom.key").await.unwrap();
    }
}
