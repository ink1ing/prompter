// 中文检测和过滤模块
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone)]
pub struct ChineseFilter {
    pub chinese_regex: Regex,
    min_chinese_chars: usize,
    exclude_patterns: Vec<Regex>,
}

impl ChineseFilter {
    pub fn new(min_chinese_chars: usize, exclude_commands: &[String]) -> anyhow::Result<Self> {
        // 中文字符正则表达式（包括中文标点）
        let chinese_regex = Regex::new(r"[\u4e00-\u9fff\u3400-\u4dbf\uff00-\uffef]")?;

        // 编译排除模式
        let mut exclude_patterns = Vec::new();
        for cmd in exclude_commands {
            let pattern = format!(r"^{}", regex::escape(cmd));
            exclude_patterns.push(Regex::new(&pattern)?);
        }

        Ok(Self {
            chinese_regex,
            min_chinese_chars,
            exclude_patterns,
        })
    }

    /// 检测文本是否包含足够的中文字符
    pub fn contains_sufficient_chinese(&self, text: &str) -> bool {
        let chinese_count = self.chinese_regex.find_iter(text).count();
        chinese_count >= self.min_chinese_chars
    }

    /// 检查是否应该排除此文本
    pub fn should_exclude(&self, text: &str) -> bool {
        let trimmed = text.trim();

        // 检查是否匹配排除模式
        for pattern in &self.exclude_patterns {
            if pattern.is_match(trimmed) {
                return true;
            }
        }

        // 排除空文本或只有空白字符的文本
        if trimmed.is_empty() {
            return true;
        }

        false
    }

    /// 过滤提示词，只保留包含中文的交互
    pub fn filter_prompt(&self, text: &str) -> Option<String> {
        if self.should_exclude(text) {
            return None;
        }

        if self.contains_sufficient_chinese(text) {
            Some(text.trim().to_string())
        } else {
            None
        }
    }

    /// 统计中文字符数量
    pub fn count_chinese_chars(&self, text: &str) -> usize {
        self.chinese_regex.find_iter(text).count()
    }

    /// 提取中文内容（用于分析）
    pub fn extract_chinese_content(&self, text: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for mat in self.chinese_regex.find_iter(text) {
            // 保留中文字符周围的上下文
            let start = mat.start().saturating_sub(10).max(last_end);
            let end = (mat.end() + 10).min(text.len());

            if start > last_end {
                result.push_str("...");
            }

            result.push_str(&text[start..end]);
            last_end = end;
        }

        if last_end < text.len() {
            result.push_str("...");
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_detection() {
        let filter = ChineseFilter::new(3, &["/".to_string(), "quit".to_string()]).unwrap();

        // 测试中文检测
        assert!(filter.contains_sufficient_chinese("帮我写一个函数"));
        assert!(filter.contains_sufficient_chinese("这是中文测试"));
        assert!(!filter.contains_sufficient_chinese("hello world"));
        assert!(!filter.contains_sufficient_chinese("你好")); // 只有2个中文字符

        // 测试过滤
        assert!(filter.filter_prompt("帮我优化代码").is_some());
        assert!(filter.filter_prompt("/help").is_none());
        assert!(filter.filter_prompt("quit").is_none());
        assert!(filter.filter_prompt("").is_none());
    }

    #[test]
    fn test_chinese_counting() {
        let filter = ChineseFilter::new(1, &[]).unwrap();

        assert_eq!(filter.count_chinese_chars("Hello 世界"), 2);
        assert_eq!(filter.count_chinese_chars("纯中文内容测试"), 6);
        assert_eq!(filter.count_chinese_chars("English only"), 0);
    }
}