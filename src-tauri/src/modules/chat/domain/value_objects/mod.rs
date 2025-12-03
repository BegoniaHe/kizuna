// Chat Domain - Value Objects
// 值对象是不可变的，通过值而非标识来比较

mod emotion;
mod message_id;
mod session_id;

pub use emotion::*;
pub use message_id::*;
pub use session_id::*;
