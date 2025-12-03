// Config Module
//
// 配置管理模块，采用六边形架构
//
// 层次结构:
// - domain: 领域层，包含配置实体、值对象和领域事件
// - ports: 端口层，定义配置读写的抽象接口
// - infrastructure: 基础设施层，实现具体的配置存储适配器
// - application: 应用层，实现 CQRS 命令和查询处理器

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ports;

// 重新导出常用类型

// Domain
pub use domain::{
    AppConfig, GeneralConfig, LLMConfig, LLMProviderConfig, Language, ModelConfig,
    PartialAppConfig, PartialGeneralConfig, PartialLLMConfig, PartialModelConfig, PositionStrategy,
    Shortcut, ShortcutConfig, Size, Theme, WindowConfig, WindowModeConfig,
};

pub use domain::{
    ConfigChangedEvent, ConfigLoadedEvent, ConfigResetEvent, ConfigSource, ThemeChangedEvent,
};

// Ports
pub use ports::{ConfigError, ConfigObserver, ConfigPort, ConfigRepository};

// Infrastructure
pub use infrastructure::{InMemoryConfigRepository, StoreConfigRepository};

// Application
pub use application::{
    CommandHandler, ConfigExistsHandler, ConfigExistsQuery, ConfigExistsResponse, ConfigService,
    DeleteConfigValueCommand, DeleteConfigValueHandler, DeleteConfigValueResponse,
    GetAllConfigHandler, GetAllConfigQuery, GetAllConfigResponse, GetConfigValueHandler,
    GetConfigValueQuery, GetConfigValueResponse, QueryHandler, ResetConfigCommand,
    ResetConfigHandler, ResetConfigResponse, SetConfigValueCommand, SetConfigValueHandler,
    SetConfigValueResponse, UpdateConfigCommand, UpdateConfigHandler, UpdateConfigResponse,
};

use std::sync::Arc;

/// Config 模块容器
///
/// 管理模块内的依赖注入
pub struct ConfigModule {
    service: ConfigService,
}

impl ConfigModule {
    /// 使用内存仓储创建（用于测试）
    pub fn new_in_memory() -> Self {
        let repository = Arc::new(InMemoryConfigRepository::new());
        Self {
            service: ConfigService::new(repository),
        }
    }

    /// 使用文件存储创建
    pub fn new_with_store(app_data_dir: std::path::PathBuf) -> Self {
        let repository = Arc::new(StoreConfigRepository::new(app_data_dir));
        Self {
            service: ConfigService::new(repository),
        }
    }

    /// 使用自定义仓储创建
    pub fn with_repository(repository: Arc<dyn ConfigRepository>) -> Self {
        Self {
            service: ConfigService::new(repository),
        }
    }

    /// 获取配置服务
    pub fn service(&self) -> &ConfigService {
        &self.service
    }

    /// 获取全部配置
    pub async fn get_all(&self) -> Result<AppConfig, ConfigError> {
        self.service.get_all().await
    }

    /// 更新配置
    pub async fn update(&self, partial: PartialAppConfig) -> Result<AppConfig, ConfigError> {
        self.service.update(partial).await
    }

    /// 重置配置
    pub async fn reset(&self) -> Result<AppConfig, ConfigError> {
        self.service.reset().await
    }

    /// 获取单个配置值
    pub async fn get<T: serde::de::DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, ConfigError> {
        self.service.get(key).await
    }

    /// 设置单个配置值
    pub async fn set<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), ConfigError> {
        self.service.set(key, value).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_module_integration() {
        let module = ConfigModule::new_in_memory();

        // 获取默认配置
        let config = module.get_all().await.unwrap();
        assert_eq!(config.general.theme, Theme::System);

        // 更新配置
        let updated = module
            .update(PartialAppConfig {
                general: Some(PartialGeneralConfig {
                    theme: Some(Theme::Dark),
                    auto_start: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(updated.general.theme, Theme::Dark);
        assert!(updated.general.auto_start);

        // 重置配置
        let reset = module.reset().await.unwrap();
        assert_eq!(reset.general.theme, Theme::System);
        assert!(!reset.general.auto_start);
    }
}
