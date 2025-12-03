use super::super::entities::Message;
use super::super::value_objects::Emotion;

/// 情感分析服务
///
/// 领域服务：分析消息内容，提取情感信息
#[derive(Debug, Clone, Default)]
pub struct EmotionAnalyzer;

impl EmotionAnalyzer {
    /// 创建新的情感分析器
    pub fn new() -> Self {
        Self
    }

    /// 分析文本的情感
    pub fn analyze(&self, text: &str) -> Option<Emotion> {
        Some(Emotion::detect_from_text(text))
    }

    /// 分析单条消息的情感
    pub fn analyze_message(&self, message: &Message) -> Emotion {
        Emotion::detect_from_text(message.content())
    }

    /// 分析响应内容的情感（用于流式响应完成后）
    pub fn analyze_text(text: &str) -> Emotion {
        Emotion::detect_from_text(text)
    }

    /// 批量分析消息情感
    pub fn analyze_batch(messages: &[Message]) -> Vec<(Message, Emotion)> {
        messages
            .iter()
            .map(|msg| (msg.clone(), Emotion::detect_from_text(msg.content())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::chat::domain::value_objects::SessionId;

    #[test]
    fn test_analyze_happy_message() {
        let analyzer = EmotionAnalyzer::new();
        let msg = Message::new_assistant(SessionId::new(), "太好了！我很高兴能帮到你！", None);
        let emotion = analyzer.analyze_message(&msg);
        assert_eq!(emotion, Emotion::Happy);
    }

    #[test]
    fn test_analyze_neutral_message() {
        let analyzer = EmotionAnalyzer::new();
        let msg = Message::new_assistant(SessionId::new(), "这是一个普通的回复。", None);
        let emotion = analyzer.analyze_message(&msg);
        assert_eq!(emotion, Emotion::Neutral);
    }
}
