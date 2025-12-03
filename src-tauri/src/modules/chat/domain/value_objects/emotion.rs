use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 情感类型
///
/// 值对象：表示 AI 响应中检测到的情感状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Emotion {
    /// 中性
    Neutral,
    /// 开心
    Happy,
    /// 悲伤
    Sad,
    /// 愤怒
    Angry,
    /// 惊讶
    Surprised,
    /// 思考
    Thinking,
}

impl Emotion {
    /// 获取所有可用的情感类型
    pub fn all() -> &'static [Emotion] {
        &[
            Emotion::Neutral,
            Emotion::Happy,
            Emotion::Sad,
            Emotion::Angry,
            Emotion::Surprised,
            Emotion::Thinking,
        ]
    }

    /// 转换为表情名称（用于模型表情映射）
    pub fn to_expression_name(&self) -> &'static str {
        match self {
            Emotion::Neutral => "neutral",
            Emotion::Happy => "smile",
            Emotion::Sad => "sad",
            Emotion::Angry => "angry",
            Emotion::Surprised => "surprised",
            Emotion::Thinking => "thinking",
        }
    }

    /// 检测文本中的情感（简单实现，后续可接入情感分析服务）
    pub fn detect_from_text(text: &str) -> Self {
        let text_lower = text.to_lowercase();

        // 简单的关键词匹配
        if text_lower.contains("开心")
            || text_lower.contains("高兴")
            || text_lower.contains("太好了")
            || text_lower.contains("哈哈")
            || text_lower.contains("😊")
            || text_lower.contains("😄")
        {
            return Emotion::Happy;
        }

        if text_lower.contains("难过")
            || text_lower.contains("伤心")
            || text_lower.contains("抱歉")
            || text_lower.contains("😢")
        {
            return Emotion::Sad;
        }

        if text_lower.contains("生气") || text_lower.contains("愤怒") || text_lower.contains("😠")
        {
            return Emotion::Angry;
        }

        if text_lower.contains("惊讶")
            || text_lower.contains("天哪")
            || text_lower.contains("居然")
            || text_lower.contains("😮")
        {
            return Emotion::Surprised;
        }

        if text_lower.contains("让我想想")
            || text_lower.contains("思考")
            || text_lower.contains("嗯")
            || text_lower.contains("🤔")
        {
            return Emotion::Thinking;
        }

        Emotion::Neutral
    }
}

impl Default for Emotion {
    fn default() -> Self {
        Self::Neutral
    }
}

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Emotion::Neutral => "neutral",
            Emotion::Happy => "happy",
            Emotion::Sad => "sad",
            Emotion::Angry => "angry",
            Emotion::Surprised => "surprised",
            Emotion::Thinking => "thinking",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Emotion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "neutral" => Ok(Emotion::Neutral),
            "happy" => Ok(Emotion::Happy),
            "sad" => Ok(Emotion::Sad),
            "angry" => Ok(Emotion::Angry),
            "surprised" => Ok(Emotion::Surprised),
            "thinking" => Ok(Emotion::Thinking),
            _ => Err(format!("Unknown emotion: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_detection() {
        assert_eq!(Emotion::detect_from_text("我很开心！"), Emotion::Happy);
        assert_eq!(Emotion::detect_from_text("这太难过了"), Emotion::Sad);
        assert_eq!(Emotion::detect_from_text("让我想想..."), Emotion::Thinking);
        assert_eq!(Emotion::detect_from_text("普通的文本"), Emotion::Neutral);
    }

    #[test]
    fn test_emotion_to_expression() {
        assert_eq!(Emotion::Happy.to_expression_name(), "smile");
        assert_eq!(Emotion::Neutral.to_expression_name(), "neutral");
    }
}
