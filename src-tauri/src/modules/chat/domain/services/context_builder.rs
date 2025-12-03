use super::super::entities::Message;

/// 上下文构建器
///
/// 领域服务：构建 LLM 请求的上下文（消息历史）
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    /// 最大上下文消息数
    max_messages: usize,
    /// 系统提示词
    system_prompt: Option<String>,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBuilder {
    /// 创建上下文构建器（默认最大 50 条消息）
    pub fn new() -> Self {
        Self {
            max_messages: 50,
            system_prompt: None,
        }
    }

    /// 创建指定最大消息数的上下文构建器
    pub fn with_max_messages(max_messages: usize) -> Self {
        Self {
            max_messages,
            system_prompt: None,
        }
    }

    /// 设置系统提示词
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// 构建上下文消息列表
    ///
    /// 返回适合发送给 LLM 的消息列表，包含：
    /// 1. 系统提示词（如果有）
    /// 2. 最近的 N 条对话消息
    /// 3. 当前用户消息
    pub fn build(&self, history: &[Message], current_message: &Message) -> Vec<ChatMessage> {
        let mut context = Vec::new();

        // 添加系统提示词
        if let Some(ref prompt) = self.system_prompt {
            context.push(ChatMessage {
                role: "system".to_string(),
                content: prompt.clone(),
            });
        }

        // 添加历史消息（最近的 N 条）
        let start = if history.len() > self.max_messages {
            history.len() - self.max_messages
        } else {
            0
        };

        for msg in &history[start..] {
            context.push(ChatMessage {
                role: msg.role().to_openai_role().to_string(),
                content: msg.content().to_string(),
            });
        }

        // 添加当前消息
        context.push(ChatMessage {
            role: current_message.role().to_openai_role().to_string(),
            content: current_message.content().to_string(),
        });

        context
    }

    /// 估算 Token 数量（粗略估算，1 token ≈ 4 个字符）
    pub fn estimate_tokens(messages: &[ChatMessage]) -> u32 {
        messages
            .iter()
            .map(|m| (m.content.len() as u32) / 4 + 4) // +4 for role overhead
            .sum()
    }
}

/// LLM 请求消息格式
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::domain::value_objects::SessionId;

    #[test]
    fn test_build_context() {
        let session_id = SessionId::new();
        let history = vec![
            Message::new_user(session_id, "你好"),
            Message::new_assistant(session_id, "你好！有什么可以帮你的？", None),
        ];
        let current = Message::new_user(session_id, "今天天气怎么样？");

        let builder =
            ContextBuilder::with_max_messages(10).with_system_prompt("你是一个友好的助手");
        let context = builder.build(&history, &current);

        assert_eq!(context.len(), 4); // system + 2 history + current
        assert_eq!(context[0].role, "system");
        assert_eq!(context[1].role, "user");
        assert_eq!(context[2].role, "assistant");
        assert_eq!(context[3].role, "user");
    }

    #[test]
    fn test_max_messages_limit() {
        let session_id = SessionId::new();
        let history: Vec<Message> = (0..20)
            .map(|i| Message::new_user(session_id, format!("Message {}", i)))
            .collect();
        let current = Message::new_user(session_id, "Current");

        let builder = ContextBuilder::with_max_messages(5);
        let context = builder.build(&history, &current);

        // 应该只有 5 条历史 + 1 条当前消息
        assert_eq!(context.len(), 6);
    }
}
