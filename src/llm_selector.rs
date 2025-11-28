use anyhow::Result;
use std::io::{self, Write};

/// LLM提供商选择
#[derive(Debug, Clone)]
pub enum LlmProvider {
    Gemini,
    Perplexity,
}

impl LlmProvider {
    pub fn as_str(&self) -> &str {
        match self {
            LlmProvider::Gemini => "gemini",
            LlmProvider::Perplexity => "perplexity",
        }
    }
}

/// LLM模式选择
#[derive(Debug, Clone)]
pub enum LlmMode {
    Fast,
    Thinking,
}

impl LlmMode {
    pub fn as_str(&self) -> &str {
        match self {
            LlmMode::Fast => "fast",
            LlmMode::Thinking => "thinking",
        }
    }
}

/// LLM配置选择结果
#[derive(Debug, Clone)]
pub struct LlmSelection {
    pub provider: LlmProvider,
    pub mode: LlmMode,
}

/// LLM选择界面
pub struct LlmSelector;

impl LlmSelector {
    /// 显示并处理LLM选择界面
    pub fn show_selection_interface() -> Result<LlmSelection> {
        Self::print_header();

        let provider = Self::select_provider()?;
        let mode = Self::select_mode(&provider)?;

        Self::print_confirmation(&provider, &mode);

        Ok(LlmSelection { provider, mode })
    }

    /// 打印头部信息
    fn print_header() {
        println!("\n{}", "=".repeat(70));
        println!("🤖 Prompter AI反馈系统 - LLM配置");
        println!("{}", "=".repeat(70));
        println!();
        println!("请选择您要使用的LLM提供商和模式：");
        println!();
    }

    /// 选择LLM提供商
    fn select_provider() -> Result<LlmProvider> {
        println!("📋 步骤 1: 选择LLM提供商");
        println!("{}", "-".repeat(40));
        println!();
        println!("1. 🧠 Gemini (Google)");
        println!("   • 优势: 强大的推理能力，支持思考模式");
        println!("   • 适用: 深度分析、复杂推理");
        println!("   • 模型: gemini-2.0-flash-thinking、gemini-2.5-flash");
        println!();
        println!("2. ⚡ Perplexity");
        println!("   • 优势: 实时搜索能力，推理模式强");
        println!("   • 适用: 实时信息、快速分析");
        println!("   • 模型: sonar-reasoning、sonar");
        println!();

        loop {
            print!("请选择提供商 (1-2) [1]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" | "" => {
                    println!("✅ 已选择: Gemini");
                    return Ok(LlmProvider::Gemini);
                }
                "2" => {
                    println!("✅ 已选择: Perplexity");
                    return Ok(LlmProvider::Perplexity);
                }
                _ => {
                    println!("❌ 无效选择，请输入 1 或 2");
                }
            }
        }
    }

    /// 选择LLM模式
    fn select_mode(provider: &LlmProvider) -> Result<LlmMode> {
        println!();
        println!("📋 步骤 2: 选择运行模式");
        println!("{}", "-".repeat(40));
        println!();

        match provider {
            LlmProvider::Gemini => {
                println!("Gemini 可用模式:");
                println!();
                println!("1. ⚡ 快速模式");
                println!("   • 模型: gemini-2.5-flash");
                println!("   • 特点: 响应快速，成本较低");
                println!("   • 适用: 日常分析、快速反馈");
                println!();
                println!("2. 🧠 思考模式");
                println!("   • 模型: gemini-2.0-flash-thinking");
                println!("   • 特点: 深度思考，质量更高");
                println!("   • 适用: 复杂分析、深度洞察");
                println!("   • 注意: 可能有配额限制，系统会自动降级");
            }
            LlmProvider::Perplexity => {
                println!("Perplexity 可用模式:");
                println!();
                println!("1. ⚡ 快速模式");
                println!("   • 模型: sonar");
                println!("   • 特点: 快速搜索和分析");
                println!("   • 适用: 实时信息查询");
                println!();
                println!("2. 🔍 推理模式");
                println!("   • 模型: sonar-reasoning");
                println!("   • 特点: 深度推理和分析");
                println!("   • 适用: 复杂问题解决");
            }
        }

        println!();

        loop {
            print!("请选择模式 (1-2) [2]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => {
                    println!("✅ 已选择: 快速模式");
                    return Ok(LlmMode::Fast);
                }
                "2" | "" => {
                    match provider {
                        LlmProvider::Gemini => println!("✅ 已选择: 思考模式"),
                        LlmProvider::Perplexity => println!("✅ 已选择: 推理模式"),
                    }
                    return Ok(LlmMode::Thinking);
                }
                _ => {
                    println!("❌ 无效选择，请输入 1 或 2");
                }
            }
        }
    }

    /// 打印确认信息
    fn print_confirmation(provider: &LlmProvider, mode: &LlmMode) {
        println!();
        println!("🎯 配置确认");
        println!("{}", "-".repeat(40));

        let (provider_name, model_name) = match (provider, mode) {
            (LlmProvider::Gemini, LlmMode::Fast) => ("Gemini", "gemini-2.5-flash"),
            (LlmProvider::Gemini, LlmMode::Thinking) => ("Gemini", "gemini-2.0-flash-thinking"),
            (LlmProvider::Perplexity, LlmMode::Fast) => ("Perplexity", "sonar"),
            (LlmProvider::Perplexity, LlmMode::Thinking) => ("Perplexity", "sonar-reasoning"),
        };

        println!("📍 LLM提供商: {}", provider_name);
        println!("🚀 运行模式: {}", match mode {
            LlmMode::Fast => "快速模式",
            LlmMode::Thinking => match provider {
                LlmProvider::Gemini => "思考模式",
                LlmProvider::Perplexity => "推理模式",
            }
        });
        println!("🤖 使用模型: {}", model_name);
        println!();
    }

    /// 检查API密钥配置
    pub fn check_api_keys(selection: &LlmSelection) -> Result<()> {
        // 这里可以添加API密钥检查逻辑
        match selection.provider {
            LlmProvider::Gemini => {
                println!("🔑 检查Gemini API密钥...");
                // TODO: 实际的API密钥检查
            }
            LlmProvider::Perplexity => {
                println!("🔑 检查Perplexity API密钥...");
                // TODO: 实际的API密钥检查
            }
        }
        Ok(())
    }

    /// 保存选择到配置文件
    pub async fn save_selection_to_config(selection: &LlmSelection, config_path: &str) -> Result<()> {
        use tokio::fs;

        println!("💾 保存配置到 {}...", config_path);

        // 读取现有配置
        let content = if std::path::Path::new(config_path).exists() {
            fs::read_to_string(config_path).await?
        } else {
            String::new()
        };

        // 更新配置内容
        let mut updated_lines = Vec::new();
        let mut in_ai_feedback_section = false;
        let mut found_provider = false;
        let mut found_mode = false;

        for line in content.lines() {
            let trimmed_line = line.trim();

            // 检测是否进入AI反馈配置段
            if trimmed_line == "[ai_feedback]" {
                in_ai_feedback_section = true;
                updated_lines.push(line.to_string());
                continue;
            }

            // 检测是否离开AI反馈配置段
            if trimmed_line.starts_with('[') && trimmed_line != "[ai_feedback]" && in_ai_feedback_section {
                in_ai_feedback_section = false;
            }

            if in_ai_feedback_section {
                if trimmed_line.starts_with("llm_provider") {
                    found_provider = true;
                    updated_lines.push(format!("llm_provider = \"{}\"            # LLM提供商选择", selection.provider.as_str()));
                } else if trimmed_line.starts_with("llm_mode") {
                    found_mode = true;
                    updated_lines.push(format!("llm_mode = \"{}\"                  # 模式选择", selection.mode.as_str()));
                } else {
                    updated_lines.push(line.to_string());
                }
            } else {
                updated_lines.push(line.to_string());
            }
        }

        // 如果没有找到相关配置行，添加它们
        if !found_provider || !found_mode {
            // 需要在ai_feedback段中添加缺失的配置
            // 这里简化处理，实际可能需要更复杂的逻辑
        }

        // 写入文件
        fs::write(config_path, updated_lines.join("\n")).await?;

        println!("✅ 配置已保存");
        Ok(())
    }
}