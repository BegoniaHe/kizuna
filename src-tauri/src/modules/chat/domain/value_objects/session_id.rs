use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// 会话唯一标识符
///
/// 值对象：通过值而非引用比较，不可变
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// 生成新的会话 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 从 UUID 创建
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 从字符串解析
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// 获取内部 UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<SessionId> for Uuid {
    fn from(id: SessionId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_equality() {
        let id1 = SessionId::new();
        let id2 = id1;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_session_id_parse() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = SessionId::parse(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }
}
