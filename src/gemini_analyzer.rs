// Gemini API集成模块 - 自动分析用户提示词并提供反馈
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub api_url: String,
    pub thinking_model: String,        // 思考模型
    pub fast_model: String,           // 快速模型
    pub thinking_budget: i32,         // 思考预算
    pub auto_fallback: bool,          // 自动降级
    pub max_retries: usize,           // 最大重试次数
    pub system_prompt: String,        // 系统提示词
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            thinking_model: "gemini-2.5-pro".to_string(),       // 默认思考模型
            fast_model: "gemini-1.5-flash".to_string(),         // 默认快速模型
            thinking_budget: 1024,                              // 默认思考预算
            auto_fallback: true,                                // 默认启用自动降级
            max_retries: 3,                                     // 默认最大重试3次
            system_prompt: String::new(),                       // 默认空,由调用方提供
        }
    }
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "thinkingConfig", skip_serializing_if = "Option::is_none")]
    thinking_config: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: i32,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

#[derive(Debug)]
pub struct PromptAnalysis {
    pub timestamp: DateTime<Local>,
    pub original_prompt: String,
    pub gemini_feedback: String,
    pub suggestions_count: usize,
}

pub struct GeminiAnalyzer {
    config: GeminiConfig,
    client: reqwest::Client,
}

impl GeminiAnalyzer {
    pub fn new(config: GeminiConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow!("Gemini API密钥不能为空"));
        }

        let client = reqwest::Client::new();

        Ok(Self {
            config,
            client,
        })
    }

    /// 分析提示词文件并返回反馈
    pub async fn analyze_prompts_file(&self, file_path: &Path) -> Result<Vec<PromptAnalysis>> {
        println!("🤖 开始分析提示词文件: {}", file_path.display());

        if !file_path.exists() {
            return Err(anyhow!("提示词文件不存在: {}", file_path.display()));
        }

        let content = fs::read_to_string(file_path)?;
        let prompts = self.extract_prompts_from_markdown(&content)?;

        if prompts.is_empty() {
            println!("📝 文件中没有发现新的提示词");
            return Ok(Vec::new());
        }

        println!("📊 发现 {} 条提示词，开始AI分析...", prompts.len());

        let mut analyses = Vec::new();
        for (i, prompt) in prompts.iter().enumerate() {
            println!("🧠 分析第 {}/{} 条提示词...", i + 1, prompts.len());

            match self.analyze_single_prompt(prompt).await {
                Ok(analysis) => {
                    println!("✅ 分析完成: {}",
                        prompt.chars().take(30).collect::<String>());
                    analyses.push(analysis);
                }
                Err(e) => {
                    println!("⚠️ 分析失败: {} - 错误: {}",
                        prompt.chars().take(30).collect::<String>(), e);
                    // 继续处理其他提示词
                }
            }

            // 避免API调用过于频繁
            if i < prompts.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        println!("🎯 AI分析完成！成功分析 {}/{} 条提示词", analyses.len(), prompts.len());
        Ok(analyses)
    }

    /// 分析单个提示词（支持思考模型和自动降级）
    async fn analyze_single_prompt(&self, prompt: &str) -> Result<PromptAnalysis> {
        // 使用系统提示词 + 用户提示词内容
        let analysis_prompt = if !self.config.system_prompt.is_empty() {
            format!("{}\n\n# 用户提示词内容\n\n\"{}\"", self.config.system_prompt, prompt)
        } else {
            // 如果没有系统提示词,使用默认格式
            format!(
                "请分析以下Claude Code用户提示词，提供专业的改进建议：

提示词内容：
\"{}\"

请从以下几个方面提供分析和改进建议：
1. 清晰度：提示词是否表达清晰、目标明确
2. 结构性：是否有良好的逻辑结构和层次
3. 完整性：是否包含足够的上下文和要求
4. 效率性：是否可以更简洁地表达同样的需求
5. 具体改进建议：给出1-3条具体的改写建议

请用中文回复，格式清晰，重点突出。",
                prompt
            )
        };

        // 先尝试使用思考模型
        match self.try_thinking_model(&analysis_prompt).await {
            Ok(analysis) => {
                println!("🧠 使用思考模型分析成功");
                Ok(analysis)
            }
            Err(e) => {
                println!("⚠️ 思考模型失败: {}", e);

                if self.config.auto_fallback {
                    println!("🔄 自动切换到快速模型...");
                    self.try_fast_model(&analysis_prompt).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// 尝试使用思考模型
    async fn try_thinking_model(&self, prompt: &str) -> Result<PromptAnalysis> {
        let mut retries = 0;

        while retries <= self.config.max_retries {
            match self.call_gemini_api(&self.config.thinking_model, prompt, Some(self.config.thinking_budget)).await {
                Ok(feedback) => {
                    let suggestions_count = feedback.matches("建议").count() +
                                          feedback.matches("改进").count() +
                                          feedback.matches("优化").count();

                    return Ok(PromptAnalysis {
                        timestamp: Local::now(),
                        original_prompt: prompt.to_string(),
                        gemini_feedback: feedback,
                        suggestions_count: suggestions_count.max(1),
                    });
                }
                Err(e) => {
                    retries += 1;

                    // 检查是否是配额错误
                    let error_msg = e.to_string();
                    if error_msg.contains("quota") || error_msg.contains("rate") || error_msg.contains("429") {
                        println!("⚠️ 思考模型配额不足，第{}次重试失败", retries);
                        if retries > self.config.max_retries {
                            return Err(anyhow::anyhow!("思考模型配额耗尽"));
                        }
                        // 等待后重试
                        tokio::time::sleep(tokio::time::Duration::from_secs(2u64.pow(retries as u32))).await;
                        continue;
                    } else {
                        // 非配额错误，直接返回
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("思考模型重试次数已耗尽"))
    }

    /// 尝试使用快速模型
    async fn try_fast_model(&self, prompt: &str) -> Result<PromptAnalysis> {
        let feedback = self.call_gemini_api(&self.config.fast_model, prompt, Some(0)).await?; // 快速模型关闭思考

        let suggestions_count = feedback.matches("建议").count() +
                              feedback.matches("改进").count() +
                              feedback.matches("优化").count();

        println!("⚡ 快速模型分析完成");

        Ok(PromptAnalysis {
            timestamp: Local::now(),
            original_prompt: prompt.to_string(),
            gemini_feedback: feedback,
            suggestions_count: suggestions_count.max(1),
        })
    }

    /// 调用Gemini API的通用方法
    async fn call_gemini_api(&self, model: &str, prompt: &str, thinking_budget: Option<i32>) -> Result<String> {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: thinking_budget.map(|budget| GenerationConfig {
                thinking_config: Some(ThinkingConfig {
                    thinking_budget: budget,
                }),
            }),
        };

        let url = format!("{}/models/{}:generateContent?key={}",
            self.config.api_url, model, self.config.api_key);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API调用失败 ({}): {}", model, error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;

        let feedback = gemini_response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .unwrap_or_else(|| "无法获取AI反馈".to_string());

        Ok(feedback)
    }

    /// 从Markdown文件中提取提示词
    fn extract_prompts_from_markdown(&self, content: &str) -> Result<Vec<String>> {
        let mut prompts = Vec::new();
        let mut in_code_block = false;
        let mut current_prompt = String::new();

        for line in content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // 代码块结束
                    if !current_prompt.trim().is_empty() {
                        prompts.push(current_prompt.trim().to_string());
                    }
                    current_prompt.clear();
                    in_code_block = false;
                } else {
                    // 代码块开始
                    in_code_block = true;
                }
            } else if in_code_block {
                // 在代码块内，收集提示词内容
                if !current_prompt.is_empty() {
                    current_prompt.push('\n');
                }
                current_prompt.push_str(line);
            }
        }

        // 过滤掉太短的内容
        let filtered_prompts: Vec<String> = prompts
            .into_iter()
            .filter(|p| p.chars().count() > 10) // 至少10个字符
            .filter(|p| self.contains_chinese(p)) // 包含中文
            .collect();

        Ok(filtered_prompts)
    }

    /// 检查字符串是否包含中文
    fn contains_chinese(&self, text: &str) -> bool {
        text.chars().any(|c| {
            matches!(c, '\u{4e00}'..='\u{9fff}' | '\u{3400}'..='\u{4dbf}' | '\u{20000}'..='\u{2a6df}')
        })
    }

    /// 生成总体点评报告(用于启动时分析历史提示词)
    pub async fn generate_overall_review(&self, prompts: &[String]) -> Result<String> {
        if prompts.is_empty() {
            return Ok("📭 暂无历史提示词数据".to_string());
        }

        println!("📊 正在分析 {} 条历史提示词...", prompts.len());

        // 将所有提示词合并成一个分析请求
        let combined_prompt = format!(
            "请对以下 {} 条Claude Code用户历史提示词进行总体分析和点评：

{}

请从以下角度提供总体分析：
1. **整体质量评价**: 这些提示词的总体质量如何(优秀/良好/一般)
2. **常见优点**: 这些提示词中体现出的优秀实践(如果有)
3. **常见问题**: 这些提示词中普遍存在的问题(如果有)
4. **改进方向**: 3-5条具体的、可操作的整体改进建议
5. **使用习惯分析**: 从提示词看出用户的使用习惯和偏好

请用中文回复,格式清晰,重点突出。总字数控制在500-800字之间。",
            prompts.len(),
            prompts.iter()
                .enumerate()
                .map(|(i, p)| format!("{}. {}", i + 1, p))
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        // 优先使用思考模型进行深度分析
        let review = match self.try_thinking_model(&combined_prompt).await {
            Ok(analysis) => {
                println!("✅ 总体点评分析完成(思考模型)");
                analysis.gemini_feedback
            }
            Err(e) => {
                println!("⚠️ 思考模型失败: {}", e);
                if self.config.auto_fallback {
                    println!("🔄 切换到快速模型...");
                    match self.try_fast_model(&combined_prompt).await {
                        Ok(analysis) => {
                            println!("✅ 总体点评分析完成(快速模型)");
                            analysis.gemini_feedback
                        }
                        Err(e2) => {
                            return Err(anyhow!("总体分析失败: {}", e2));
                        }
                    }
                } else {
                    return Err(e);
                }
            }
        };

        Ok(review)
    }

    /// 从本地文件读取历史提示词(运行软件前的历史数据)
    pub async fn read_historical_prompts(&self, file_path: &Path) -> Result<Vec<String>> {
        if !file_path.exists() {
            println!("📝 历史提示词文件不存在: {}", file_path.display());
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(file_path).await
            .map_err(|e| anyhow!("读取历史文件失败: {}", e))?;

        // 从Markdown格式提取提示词
        let mut prompts = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // 查找时间戳行 (## 2024-11-24 21:30:15)
            if line.starts_with("##") && line.contains("20") {
                // 跳过空行
                i += 1;
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // 收集提示词内容(直到遇到分隔线或下一个时间戳)
                let mut prompt_content = String::new();
                while i < lines.len() {
                    let content_line = lines[i].trim();

                    if content_line == "---" || content_line.starts_with("##") {
                        break;
                    }

                    if !content_line.is_empty() {
                        if !prompt_content.is_empty() {
                            prompt_content.push(' ');
                        }
                        prompt_content.push_str(content_line);
                    }

                    i += 1;
                }

                // 添加非空的提示词
                if !prompt_content.is_empty() && prompt_content.len() >= 10 {
                    prompts.push(prompt_content);
                }
            } else {
                i += 1;
            }
        }

        println!("📖 读取到 {} 条历史提示词", prompts.len());
        Ok(prompts)
    }

    /// 生成分析报告
    pub fn generate_report(&self, analyses: &[PromptAnalysis]) -> String {
        if analyses.is_empty() {
            return "📝 本次没有新的提示词需要分析".to_string();
        }

        let mut report = String::new();
        report.push_str("🤖 AI提示词分析报告\n");
        report.push_str(&"=".repeat(50));
        report.push('\n');
        report.push_str(&format!("📊 分析时间: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&format!("📈 提示词数量: {}\n", analyses.len()));
        report.push_str(&format!("🎯 建议总数: {}\n",
            analyses.iter().map(|a| a.suggestions_count).sum::<usize>()));
        report.push_str(&"-".repeat(50));
        report.push('\n');

        for (i, analysis) in analyses.iter().enumerate() {
            report.push_str(&format!("\n## 📝 提示词 #{}\n", i + 1));
            report.push_str(&format!("**原始内容**: {}\n",
                if analysis.original_prompt.len() > 100 {
                    format!("{}...", analysis.original_prompt.chars().take(100).collect::<String>())
                } else {
                    analysis.original_prompt.clone()
                }
            ));
            report.push_str(&format!("**分析时间**: {}\n",
                analysis.timestamp.format("%H:%M:%S")));
            report.push_str(&format!("**建议数量**: {}\n\n", analysis.suggestions_count));

            report.push_str("### 🧠 AI分析与建议：\n");
            report.push_str(&analysis.gemini_feedback);
            report.push_str("\n\n");
            report.push_str(&"-".repeat(30));
            report.push('\n');
        }

        report.push_str("\n\n🎉 分析完成！希望这些建议对您有帮助。\n");
        report.push_str("💡 定期优化提示词可以显著提升AI交互效果。\n");

        report
    }

    /// 测试API连接
    pub async fn test_connection(&self) -> Result<()> {
        println!("🔧 测试Gemini API连接...");

        let test_prompt = "你好，请回复一个简短的测试消息。";

        // 先测试思考模型
        match self.call_gemini_api(&self.config.thinking_model, test_prompt, Some(self.config.thinking_budget)).await {
            Ok(_) => {
                println!("✅ 思考模型 ({}) 连接成功！", self.config.thinking_model);
                return Ok(());
            }
            Err(e) => {
                println!("⚠️ 思考模型连接失败: {}", e);

                if self.config.auto_fallback {
                    println!("🔄 尝试快速模型...");

                    // 测试快速模型
                    match self.call_gemini_api(&self.config.fast_model, test_prompt, Some(0)).await {
                        Ok(_) => {
                            println!("✅ 快速模型 ({}) 连接成功！", self.config.fast_model);
                            return Ok(());
                        }
                        Err(e2) => {
                            return Err(anyhow::anyhow!(
                                "思考模型和快速模型都连接失败。\n思考模型错误: {}\n快速模型错误: {}",
                                e, e2
                            ));
                        }
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prompts_from_markdown() {
        let analyzer = GeminiAnalyzer::new(GeminiConfig::default()).unwrap();
        let content = r#"
## 2024-01-01 10:00:00 (Claude历史监控)

```
帮我写一个Rust函数来处理JSON数据
```

## 2024-01-01 10:05:00 (Claude历史监控)

```
请解释一下这段代码的作用
```
"#;

        let prompts = analyzer.extract_prompts_from_markdown(content).unwrap();
        assert_eq!(prompts.len(), 2);
        assert!(prompts[0].contains("Rust函数"));
        assert!(prompts[1].contains("解释"));
    }

    #[test]
    fn test_contains_chinese() {
        let analyzer = GeminiAnalyzer::new(GeminiConfig::default()).unwrap();
        assert!(analyzer.contains_chinese("帮我写代码"));
        assert!(analyzer.contains_chinese("Hello 世界"));
        assert!(!analyzer.contains_chinese("Hello World"));
    }
}