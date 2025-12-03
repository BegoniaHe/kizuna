// Window Port
//
// 窗口管理端口定义

use async_trait::async_trait;
use thiserror::Error;

use crate::modules::window::domain::{
    WindowConfig, WindowLabel, WindowMode, WindowPosition, WindowSize, WindowState,
};

/// 窗口错误类型
#[derive(Error, Debug)]
pub enum WindowError {
    #[error("Window not found: {0}")]
    NotFound(String),

    #[error("Window operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid window configuration: {0}")]
    InvalidConfig(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Window already exists: {0}")]
    AlreadyExists(String),
}

/// 窗口管理端口
#[async_trait]
pub trait WindowPort: Send + Sync {
    /// 创建窗口
    async fn create(&self, config: WindowConfig) -> Result<WindowState, WindowError>;

    /// 获取窗口状态
    async fn get_state(&self, label: &WindowLabel) -> Result<Option<WindowState>, WindowError>;

    /// 列出所有窗口
    async fn list_windows(&self) -> Result<Vec<WindowState>, WindowError>;

    /// 切换窗口模式
    async fn switch_mode(
        &self,
        label: &WindowLabel,
        mode: WindowMode,
    ) -> Result<WindowState, WindowError>;

    /// 设置窗口尺寸
    async fn set_size(&self, label: &WindowLabel, size: WindowSize) -> Result<(), WindowError>;

    /// 设置窗口位置
    async fn set_position(
        &self,
        label: &WindowLabel,
        position: WindowPosition,
    ) -> Result<(), WindowError>;

    /// 设置窗口置顶
    async fn set_always_on_top(
        &self,
        label: &WindowLabel,
        always_on_top: bool,
    ) -> Result<(), WindowError>;

    /// 设置窗口装饰
    async fn set_decorations(
        &self,
        label: &WindowLabel,
        decorations: bool,
    ) -> Result<(), WindowError>;

    /// 显示窗口
    async fn show(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 隐藏窗口
    async fn hide(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 关闭窗口
    async fn close(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 最小化窗口
    async fn minimize(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 最大化窗口
    async fn maximize(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 取消最大化
    async fn unmaximize(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 居中窗口
    async fn center(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 开始拖拽窗口
    async fn start_dragging(&self, label: &WindowLabel) -> Result<(), WindowError>;

    /// 设置窗口焦点
    async fn set_focus(&self, label: &WindowLabel) -> Result<(), WindowError>;
}

/// 窗口模式策略 trait
pub trait WindowModeStrategy: Send + Sync {
    /// 获取模式类型
    fn mode(&self) -> WindowMode;

    /// 应用模式到窗口配置
    fn apply(&self, config: &mut WindowConfig);

    /// 获取模式的默认尺寸
    fn default_size(&self) -> WindowSize;

    /// 模式是否需要透明背景
    fn requires_transparent(&self) -> bool;
}

/// 普通模式策略
pub struct NormalModeStrategy {
    size: WindowSize,
}

impl NormalModeStrategy {
    pub fn new() -> Self {
        Self {
            size: WindowSize::new(1200, 800),
        }
    }

    pub fn with_size(size: WindowSize) -> Self {
        Self { size }
    }
}

impl Default for NormalModeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowModeStrategy for NormalModeStrategy {
    fn mode(&self) -> WindowMode {
        WindowMode::Normal
    }

    fn apply(&self, config: &mut WindowConfig) {
        config.mode = WindowMode::Normal;
        config.size = self.size;
        config.decorations = true;
        config.always_on_top = false;
        config.transparent = false;
        config.skip_taskbar = false;
        config.resizable = true;
    }

    fn default_size(&self) -> WindowSize {
        self.size
    }

    fn requires_transparent(&self) -> bool {
        false
    }
}

/// 桌面宠物模式策略
pub struct PetModeStrategy {
    size: WindowSize,
}

impl PetModeStrategy {
    pub fn new() -> Self {
        Self {
            size: WindowSize::new(300, 400),
        }
    }

    pub fn with_size(size: WindowSize) -> Self {
        Self { size }
    }
}

impl Default for PetModeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowModeStrategy for PetModeStrategy {
    fn mode(&self) -> WindowMode {
        WindowMode::Pet
    }

    fn apply(&self, config: &mut WindowConfig) {
        config.mode = WindowMode::Pet;
        config.size = self.size;
        config.decorations = false;
        config.always_on_top = true;
        config.transparent = true;
        config.skip_taskbar = true;
        config.resizable = false;
    }

    fn default_size(&self) -> WindowSize {
        self.size
    }

    fn requires_transparent(&self) -> bool {
        true
    }
}

/// 紧凑模式策略
pub struct CompactModeStrategy {
    size: WindowSize,
}

impl CompactModeStrategy {
    pub fn new() -> Self {
        Self {
            size: WindowSize::new(400, 600),
        }
    }

    pub fn with_size(size: WindowSize) -> Self {
        Self { size }
    }
}

impl Default for CompactModeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowModeStrategy for CompactModeStrategy {
    fn mode(&self) -> WindowMode {
        WindowMode::Compact
    }

    fn apply(&self, config: &mut WindowConfig) {
        config.mode = WindowMode::Compact;
        config.size = self.size;
        config.decorations = false;
        config.always_on_top = true;
        config.transparent = false;
        config.skip_taskbar = false;
        config.resizable = true;
    }

    fn default_size(&self) -> WindowSize {
        self.size
    }

    fn requires_transparent(&self) -> bool {
        false
    }
}

/// 窗口模式策略注册表
pub struct WindowModeRegistry {
    strategies: std::collections::HashMap<WindowMode, Box<dyn WindowModeStrategy>>,
}

impl WindowModeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            strategies: std::collections::HashMap::new(),
        };

        // 注册默认策略
        registry.register(Box::new(NormalModeStrategy::new()));
        registry.register(Box::new(PetModeStrategy::new()));
        registry.register(Box::new(CompactModeStrategy::new()));

        registry
    }

    pub fn register(&mut self, strategy: Box<dyn WindowModeStrategy>) {
        self.strategies.insert(strategy.mode(), strategy);
    }

    pub fn get(&self, mode: WindowMode) -> Option<&dyn WindowModeStrategy> {
        self.strategies.get(&mode).map(|s| s.as_ref())
    }

    pub fn apply_mode(
        &self,
        config: &mut WindowConfig,
        mode: WindowMode,
    ) -> Result<(), WindowError> {
        let strategy = self
            .strategies
            .get(&mode)
            .ok_or_else(|| WindowError::InvalidConfig(format!("Unknown mode: {:?}", mode)))?;

        strategy.apply(config);
        Ok(())
    }
}

impl Default for WindowModeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
