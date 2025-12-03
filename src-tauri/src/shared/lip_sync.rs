//! 口型同步工具 - 将中文文本转换为口型音素序列
//!
//! 使用 rust-pinyin 库将中文转换为拼音，然后映射到 AEIOU 口型系统

use pinyin::ToPinyin;

/// 口型音素类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phoneme {
    /// 张大嘴 (a, ai, ao, an, ang)
    A,
    /// 半开嘴 (e, ei, en, eng, er)
    E,
    /// 扁嘴 (i, ia, ie, iu, in, ing, iao, ian, iang, iong)
    I,
    /// 圆嘴 (o, ou, ong)
    O,
    /// 嘟嘴 (u, ua, uo, ui, un, ue, ü, üe, üan, ün, uai, uan, uang)
    U,
    /// 鼻音/闭嘴 (n, m, ng 结尾或停顿)
    N,
    /// 闭嘴 (标点、空格等)
    Closed,
}

impl Phoneme {
    /// 转换为字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Phoneme::A => "A",
            Phoneme::E => "E",
            Phoneme::I => "I",
            Phoneme::O => "O",
            Phoneme::U => "U",
            Phoneme::N => "N",
            Phoneme::Closed => "closed",
        }
    }
}

/// 将拼音韵母映射到口型音素
fn final_to_phoneme(pinyin: &str) -> Phoneme {
    let pinyin_lower = pinyin.to_lowercase();
    
    // 提取韵母部分 (去掉声调数字)
    let finals: String = pinyin_lower
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect();
    
    // 根据韵母判断口型
    // 注意顺序：先匹配更特殊的情况
    
    // U 系列 (包括 ü, wu, yu 等)
    // "wu" 是 "乌" 的拼音，需要优先匹配
    if finals == "wu" || finals.starts_with("wu") {
        return Phoneme::U;
    }
    if finals.starts_with('u') || finals.starts_with('ü') || finals.contains("ue") || finals.contains("üe") {
        return Phoneme::U;
    }
    
    // A 系列
    if finals.contains('a') {
        return Phoneme::A;
    }
    
    // O 系列 (优先于其他，因为 o 的口型最明显)
    if finals.starts_with('o') || finals.ends_with("ong") {
        return Phoneme::O;
    }
    
    // E 系列
    if finals.contains('e') && !finals.contains("ie") && !finals.contains("ue") && !finals.contains("üe") {
        return Phoneme::E;
    }
    
    // I 系列
    if finals.contains('i') || finals.starts_with('y') {
        return Phoneme::I;
    }
    
    // 默认返回 A
    Phoneme::A
}

/// 判断是否是标点符号
fn is_punctuation(c: char) -> bool {
    // 常见中文标点
    matches!(c, 
        '，' | '。' | '！' | '？' | '、' | '；' | '：' |
        '（' | '）' | '【' | '】' | '《' | '》' | '·' | '…' | '—' |
        '"' | '\''
    )
}

/// 将单个字符转换为口型音素
fn char_to_phoneme(c: char) -> Phoneme {
    // 标点符号和空格 -> 闭嘴
    if c.is_whitespace() || c.is_ascii_punctuation() || is_punctuation(c) {
        return Phoneme::Closed;
    }
    
    // 尝试获取拼音
    if let Some(pinyin) = c.to_pinyin() {
        let py_str = pinyin.with_tone_num();
        return final_to_phoneme(py_str);
    }
    
    // 非汉字（英文字母等）
    let lower = c.to_ascii_lowercase();
    match lower {
        'a' => Phoneme::A,
        'e' => Phoneme::E,
        'i' | 'y' => Phoneme::I,
        'o' => Phoneme::O,
        'u' | 'w' => Phoneme::U,
        'm' | 'n' => Phoneme::N,
        _ => {
            // 辅音字母 - 随机分配元音以产生自然的说话效果
            // 使用字符编码的简单哈希
            let hash = (c as u32 * 7 + (c as u32 >> 3)) % 5;
            match hash {
                0 => Phoneme::A,
                1 => Phoneme::E,
                2 => Phoneme::I,
                3 => Phoneme::O,
                _ => Phoneme::U,
            }
        }
    }
}

/// 将文本转换为口型音素序列
///
/// # Arguments
/// * `text` - 输入文本（可以是中文、英文或混合）
///
/// # Returns
/// 口型音素字符串数组 ("A", "E", "I", "O", "U", "N", "closed")
pub fn text_to_phonemes(text: &str) -> Vec<String> {
    let mut phonemes = Vec::new();
    let mut last_phoneme: Option<Phoneme> = None;
    
    for c in text.chars() {
        let phoneme = char_to_phoneme(c);
        
        // 合并连续相同的音素（减少数据量）
        if Some(phoneme) != last_phoneme {
            phonemes.push(phoneme.as_str().to_string());
            last_phoneme = Some(phoneme);
        }
    }
    
    phonemes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_to_phonemes() {
        let phonemes = text_to_phonemes("你好");
        println!("你好 -> {:?}", phonemes);
        assert!(!phonemes.is_empty());
    }

    #[test]
    fn test_mixed_text() {
        let phonemes = text_to_phonemes("Hello 世界！");
        println!("Hello 世界！ -> {:?}", phonemes);
        assert!(phonemes.contains(&"closed".to_string())); // 标点
    }

    #[test]
    fn test_punctuation() {
        let phonemes = text_to_phonemes("。");
        assert_eq!(phonemes, vec!["closed"]);
    }

    #[test]
    fn test_vowels() {
        // 测试不同韵母的映射
        let test_cases = [
            ("啊", "A"),  // a
            ("哦", "O"),  // o
            ("呃", "E"),  // e
            ("衣", "I"),  // i (yi)
            ("乌", "U"),  // u (wu)
        ];
        
        for (char, expected) in test_cases {
            let phonemes = text_to_phonemes(char);
            println!("{} -> {:?}", char, phonemes);
            assert!(phonemes.contains(&expected.to_string()), 
                    "Expected {} for '{}', got {:?}", expected, char, phonemes);
        }
    }
}
