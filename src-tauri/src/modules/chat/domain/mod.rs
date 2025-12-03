// Chat Domain Layer
// 领域层包含业务实体、值对象、领域服务和领域事件

pub mod entities;
pub mod events;
pub mod services;
pub mod value_objects;

// 重导出常用类型
pub use entities::{Message, MessageRole, Session};
pub use events::*;
pub use services::{ChatMessage, ContextBuilder, EmotionAnalyzer};
pub use value_objects::{Emotion, MessageId, SessionId};
