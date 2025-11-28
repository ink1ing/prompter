use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

/// Perplexity API配置
#[derive(Debug, Clone)]
pub struct PerplexityConfig {
    pub api_key: String,
    pub model: String,
    pub api_url: String,
    pub max_retries: usize,
    pub timeout_seconds: u64,
}

impl Default for PerplexityConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "sonar-reasoning".to_string(),
            api_url: "https://api.perplexity.ai".to_string(),
            max_retries: 3,
            timeout_seconds: 60,
        }
    }
}

/// Perplexity API消息结构
#[derive(Debug, Serialize, Deserialize)]
struct PerplexityMessage {
    role: String,
    content: String,
}

/// Perplexity API请求结构
#[derive(Debug, Serialize)]
struct PerplexityRequest {
    model: String,
    messages: Vec<PerplexityMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: bool,
}

/// Perplexity API响应结构
#[derive(Debug, Deserialize)]
struct PerplexityResponse {
    choices: Vec<PerplexityChoice>,
    usage: Option<PerplexityUsage>,
}

#[derive(Debug, Deserialize)]
struct PerplexityChoice {
    message: PerplexityMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PerplexityUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Perplexity分析器
pub struct PerplexityAnalyzer {
    config: PerplexityConfig,
    client: Client,
}

impl PerplexityAnalyzer {
    pub fn new(config: PerplexityConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(anyhow!("Perplexity API密钥不能为空"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self { config, client })
    }

    /// 测试API连接
    pub async fn test_connection(&self) -> Result<()> {
        println!("🔧 测试Perplexity API连接...");

        let test_prompt = "Hello, this is a connection test.";
        match self.analyze_single_prompt(test_prompt).await {
            Ok(_) => {
                println!("✅ Perplexity API连接成功");
                Ok(())
            }
            Err(e) => {
                println!("❌ Perplexity API连接失败: {}", e);
                Err(e)
            }
        }
    }

    /// 分析单个提示词
    pub async fn analyze_single_prompt(&self, prompt: &str) -> Result<String> {
        let system_prompt = "你是一名专业的 LLM 提示词审阅分析员。请对用户的提示词进行简要分析，指出其优点和可改进之处。回复要简洁专业，控制在200字以内。";

        self.send_request(system_prompt, prompt).await
    }

    /// 生成总体分析报告
    pub async fn generate_overall_review(&self, prompts: &[String]) -> Result<String> {
        let system_prompt = r#"你是一名专业的 LLM 提示词审阅分析员。请基于今天抓取到的所有"我与 Claude Code 的提示词交互记录"，输出一份可在 1 分钟内阅读完毕的日报式总结。按照以下要求生成内容：

1. **关键不足与改进建议（最重要）**
   * 用简洁、结构化的方式指出用户提示词在表达、目标定义、约束条件、输入结构、示例、可执行性、可复用性等方面的不足。
   * 每条给出明确、可操作的改写建议。
   * 参考 OpenAI/Anthropic 官方文档与研究中的提示词最佳实践（如：清晰指令、层级化结构、角色设定、输入输出示例、约束条件、逐步推理等）。

2. **亮点与优秀交互案例**
   * 选出当天 1–3 个做得特别好的提示词或交互，并说明为何有效（与最佳实践的对齐点）。

3. **总体模式洞察**
   * 简要概括用户近期提示词风格的趋势：典型误区、优势、可优化方向。

4. **风格要求**
   * 简洁精炼、无废话；整体阅读时间不超过2分钟。
   * 按条列形式组织，不要冗长解释。
   * 聚焦任务性、结构化、可执行。
   * 不用markdown语法，直接输出即可。

输出格式如下（严格遵守）：

【不足与改进】
* …
* …

【亮点案例】
* …

【总体洞察】
* …"#;

        // 选择最近的5-10个提示词进行分析
        let recent_prompts: Vec<String> = prompts.iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        let combined_prompts = format!(
            "以下是用户今天的提示词记录：\n\n{}",
            recent_prompts.iter()
                .enumerate()
                .map(|(i, p)| format!("{}. {}", i+1, p))
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        self.send_request(system_prompt, &combined_prompts).await
    }

    /// 发送API请求
    async fn send_request(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                println!("⚠️ Perplexity API重试第 {} 次...", attempt);
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt as u32))).await;
            }

            match self.make_api_call(system_prompt, user_prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    println!("⚠️ Perplexity API调用失败: {}", last_error.as_ref().unwrap());
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// 实际的API调用
    async fn make_api_call(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let request_body = PerplexityRequest {
            model: self.config.model.clone(),
            messages: vec![
                PerplexityMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                PerplexityMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            max_tokens: Some(2000),
            temperature: Some(0.7),
            stream: false,
        };

        let url = format!("{}/chat/completions", self.config.api_url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Perplexity API错误 {}: {}",
                status,
                error_text
            ));
        }

        let perplexity_response: PerplexityResponse = response.json().await?;

        if let Some(choice) = perplexity_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("Perplexity API返回空响应"))
        }
    }

    /// 分析提示词文件
    pub async fn analyze_prompts_file(&self, file_path: &std::path::Path) -> Result<Vec<String>> {
        if !file_path.exists() {
            return Err(anyhow!("文件不存在: {:?}", file_path));
        }

        let content = tokio::fs::read_to_string(file_path).await?;
        let prompts = self.extract_prompts_from_content(&content);

        println!("📊 从文件中提取到 {} 条提示词", prompts.len());

        let mut analyses = Vec::new();

        for (i, prompt) in prompts.iter().enumerate() {
            if i >= 5 { // 限制分析数量避免API配额过度消耗
                break;
            }

            println!("🔍 分析第 {} 条提示词...", i + 1);

            match self.analyze_single_prompt(prompt).await {
                Ok(analysis) => {
                    analyses.push(analysis);
                    println!("✅ 第 {} 条分析完成", i + 1);
                }
                Err(e) => {
                    println!("⚠️ 第 {} 条分析失败: {}", i + 1, e);
                }
            }

            // 添加间隔避免API限制
            if i < prompts.len() - 1 {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }

        Ok(analyses)
    }

    /// 从文件内容中提取提示词
    fn extract_prompts_from_content(&self, content: &str) -> Vec<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut prompts = Vec::new();
        let mut current_prompt = String::new();
        let mut in_code_block = false;

        for line in lines {
            // 检测时间戳行，表示新的提示词条目开始
            if line.starts_with("## 20") {
                // 保存上一个提示词
                if !current_prompt.trim().is_empty() {
                    prompts.push(current_prompt.trim().to_string());
                    current_prompt.clear();
                }
                in_code_block = false;
                continue;
            }

            // 检测代码块标记
            if line.trim() == "```" {
                in_code_block = !in_code_block;
                continue;
            }

            // 如果在代码块中，这就是提示词内容
            if in_code_block && !line.trim().is_empty() {
                if !current_prompt.is_empty() {
                    current_prompt.push(' ');
                }
                current_prompt.push_str(line.trim());
                continue;
            }
        }

        // 添加最后一个提示词
        if !current_prompt.trim().is_empty() {
            prompts.push(current_prompt.trim().to_string());
        }

        prompts
    }

    /// 生成分析报告
    pub fn generate_report(&self, analyses: &[String]) -> String {
        format!(
            "📊 Perplexity分析报告\n\n\
            🕒 生成时间: {}\n\
            🔍 分析数量: {}条\n\
            🤖 使用模型: {}\n\n\
            {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            analyses.len(),
            self.config.model,
            analyses.iter()
                .enumerate()
                .map(|(i, analysis)| format!("【分析 {}】\n{}", i + 1, analysis))
                .collect::<Vec<_>>()
                .join("\n\n")
        )
    }
}