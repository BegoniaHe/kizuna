// Chat Infrastructure Layer
// 基础设施层包含端口的具体实现

pub mod adapters;
pub mod repositories;

// 重导出常用类型
pub use adapters::llm::{
    DynamicLLMAdapter, DynamicLLMConfig, LLMAdapterRegistry, MockLLMAdapter, OpenAIAdapter,
};
pub use repositories::{
    FileMessageRepository, FileSessionRepository, InMemoryMessageRepository,
    InMemorySessionRepository,
};
