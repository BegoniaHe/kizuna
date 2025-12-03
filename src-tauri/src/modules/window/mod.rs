// Window Module
//
// 窗口管理模块，采用六边形架构
//
// 层次结构:
// - domain: 领域层，包含窗口配置实体、值对象和领域事件
// - ports: 端口层，定义窗口管理的抽象接口和模式策略
// - infrastructure: 基础设施层，实现 Tauri 窗口适配器

pub mod domain;
pub mod infrastructure;
pub mod ports;

// 重新导出常用类型

// Domain
pub use domain::{
    ModeSizeConfig, WindowClosedEvent, WindowConfig, WindowCreatedEvent, WindowFocusChangedEvent,
    WindowLabel, WindowMode, WindowModeChangedEvent, WindowMovedEvent, WindowPosition,
    WindowResizedEvent, WindowSize, WindowState, WindowVisibilityChangedEvent,
};

// Ports
pub use ports::{
    CompactModeStrategy, NormalModeStrategy, PetModeStrategy, WindowError, WindowModeRegistry,
    WindowModeStrategy, WindowPort,
};

// Infrastructure
pub use infrastructure::TauriWindowAdapter;

use std::sync::Arc;
use tauri::AppHandle;

/// Window 模块容器
///
/// 管理窗口相关的依赖注入
pub struct WindowModule {
    adapter: Arc<dyn WindowPort>,
    mode_registry: WindowModeRegistry,
}

impl WindowModule {
    /// 使用 Tauri AppHandle 创建
    pub fn new(app_handle: AppHandle) -> Self {
        let mode_registry = WindowModeRegistry::new();
        let adapter = Arc::new(TauriWindowAdapter::new(app_handle));

        Self {
            adapter,
            mode_registry,
        }
    }

    /// 使用自定义适配器创建
    pub fn with_adapter(adapter: Arc<dyn WindowPort>) -> Self {
        Self {
            adapter,
            mode_registry: WindowModeRegistry::new(),
        }
    }

    /// 获取窗口适配器
    pub fn adapter(&self) -> &Arc<dyn WindowPort> {
        &self.adapter
    }

    /// 获取模式注册表
    pub fn mode_registry(&self) -> &WindowModeRegistry {
        &self.mode_registry
    }

    /// 创建新窗口
    pub async fn create_window(&self, config: WindowConfig) -> Result<WindowState, WindowError> {
        self.adapter.create(config).await
    }

    /// 获取窗口状态
    pub async fn get_window_state(
        &self,
        label: &WindowLabel,
    ) -> Result<Option<WindowState>, WindowError> {
        self.adapter.get_state(label).await
    }

    /// 列出所有窗口
    pub async fn list_windows(&self) -> Result<Vec<WindowState>, WindowError> {
        self.adapter.list_windows().await
    }

    /// 关闭窗口
    pub async fn close_window(&self, label: &WindowLabel) -> Result<(), WindowError> {
        self.adapter.close(label).await
    }

    /// 切换窗口模式
    pub async fn switch_mode(
        &self,
        label: &WindowLabel,
        mode: WindowMode,
    ) -> Result<WindowState, WindowError> {
        self.adapter.switch_mode(label, mode).await
    }

    /// 切换到桌面宠物模式
    pub async fn switch_to_pet_mode(&self) -> Result<WindowState, WindowError> {
        self.adapter
            .switch_mode(&WindowLabel::main(), WindowMode::Pet)
            .await
    }

    /// 切换到普通模式
    pub async fn switch_to_normal_mode(&self) -> Result<WindowState, WindowError> {
        self.adapter
            .switch_mode(&WindowLabel::main(), WindowMode::Normal)
            .await
    }

    /// 切换置顶状态
    pub async fn toggle_always_on_top(
        &self,
        label: &WindowLabel,
        always_on_top: bool,
    ) -> Result<(), WindowError> {
        self.adapter.set_always_on_top(label, always_on_top).await
    }

    /// 开始拖拽
    pub async fn start_dragging(&self, label: &WindowLabel) -> Result<(), WindowError> {
        self.adapter.start_dragging(label).await
    }

    /// 显示窗口
    pub async fn show(&self, label: &WindowLabel) -> Result<(), WindowError> {
        self.adapter.show(label).await
    }

    /// 隐藏窗口
    pub async fn hide(&self, label: &WindowLabel) -> Result<(), WindowError> {
        self.adapter.hide(label).await
    }

    /// 最小化窗口
    pub async fn minimize(&self, label: &WindowLabel) -> Result<(), WindowError> {
        self.adapter.minimize(label).await
    }

    /// 居中窗口
    pub async fn center(&self, label: &WindowLabel) -> Result<(), WindowError> {
        self.adapter.center(label).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_mode_registry() {
        let registry = WindowModeRegistry::new();

        let normal = registry.get(WindowMode::Normal);
        assert!(normal.is_some());
        assert!(!normal.unwrap().requires_transparent());

        let pet = registry.get(WindowMode::Pet);
        assert!(pet.is_some());
        assert!(pet.unwrap().requires_transparent());
    }
}
