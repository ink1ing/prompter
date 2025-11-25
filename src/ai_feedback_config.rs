// AI反馈系统配置模块 - 用户友好的引导界面
use anyhow::{Result, anyhow};
use std::io::{Write, BufRead};
use std::fs;
use std::path::Path;
use std::process::Command;
use chrono::Local;

pub struct ConfigWizard;

impl ConfigWizard {
    /// 启动配置向导 - 一键式用户引导
    pub async fn start_setup_wizard() -> Result<()> {
        Self::print_welcome_banner();

        println!("🎯 让我们一步步配置您的专属AI反馈系统...\n");

        // 检查是否已有配置
        if Self::check_existing_config()? {
            println!("✅ 发现现有配置文件");
            Self::show_current_config()?;
            if !Self::confirm("是否重新配置？(y/N)")? {
                println!("👍 保持现有配置。您可以运行 --test-feedback 测试系统。");
                return Ok(());
            }
            println!();
        }

        // 配置Gemini API
        let gemini_key = Self::setup_gemini_api_interactive()?;

        // 配置Telegram Bot
        let (bot_token, chat_id) = Self::setup_telegram_bot_interactive().await?;

        // 保存配置
        Self::save_configuration(&gemini_key, &bot_token, &chat_id)?;

        // 测试配置
        println!("\n🧪 正在测试配置...");
        match Self::test_configuration(&gemini_key, &bot_token).await {
            Ok(_) => {
                Self::print_success_message();
                Self::print_next_steps();
            }
            Err(e) => {
                println!("⚠️ 配置测试失败: {}", e);
                println!("💡 请检查API密钥是否正确，或运行 --test-feedback 进行诊断");
            }
        }
        Ok(())
    }

    /// 交互式设置Gemini API
    fn setup_gemini_api_interactive() -> Result<String> {
        println!("{}", "=".repeat(70));
        println!("🤖 步骤 1: 配置 Google Gemini API");
        println!("{}", "=".repeat(70));

        println!("📖 Gemini API 是Google的生成式AI服务，用于智能分析您的Claude提示词");
        println!("💡 我们将使用它来提供专业的改进建议和优化方案\n");

        // 自动打开API密钥获取页面
        println!("🔑 即将为您打开Google AI Studio获取API密钥...");
        if Self::confirm("按 Enter 继续")? {
            Self::open_browser("https://aistudio.google.com/api-keys")?;
        }

        println!("\n📋 获取API密钥步骤:");
        println!("   1. 点击 'Create API Key' 按钮");
        println!("   2. 选择一个Google Cloud项目（或创建新项目）");
        println!("   3. 复制生成的API密钥");
        println!("   4. 粘贴到下方输入框\n");

        loop {
            print!("🔐 请输入您的Gemini API密钥: ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let api_key = input.trim();

            if api_key.is_empty() {
                println!("❌ API密钥不能为空，请重新输入");
                continue;
            }

            if !api_key.starts_with("AIza") {
                println!("⚠️ API密钥格式可能不正确（通常以'AIza'开头）");
                if !Self::confirm("确定要使用这个密钥吗？(y/N)")? {
                    continue;
                }
            }

            println!("✅ API密钥已保存");
            return Ok(api_key.to_string());
        }
    }

    /// 交互式设置Telegram Bot
    async fn setup_telegram_bot_interactive() -> Result<(String, String)> {
        println!("\n{}", "=".repeat(70));
        println!("📱 步骤 2: 配置 Telegram 机器人");
        println!("{}", "=".repeat(70));

        println!("🤖 Telegram Bot 用于接收AI分析报告和系统通知");
        println!("💬 您将每天收到个性化的提示词优化建议\n");

        // 自动打开Telegram Bot创建页面
        println!("🔧 即将为您打开Telegram BotFather创建机器人...");
        if Self::confirm("按 Enter 继续")? {
            Self::open_browser("https://web.telegram.org/k/#@BotFather")?;
        }

        println!("\n📋 创建Telegram Bot步骤:");
        println!("   1. 在Telegram中找到 @BotFather");
        println!("   2. 发送 /newbot 命令");
        println!("   3. 为您的机器人起个名字（如: MyAI助手）");
        println!("   4. 设置用户名（如: my_prompter_bot）");
        println!("   5. 复制获得的Bot Token");
        println!("   6. 向新创建的机器人发送任意消息（如: /start）");
        println!("   7. 粘贴Bot Token到下方输入框\n");

        let bot_token = loop {
            print!("🤖 请输入您的Telegram Bot Token: ");
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let token = input.trim();

            if token.is_empty() {
                println!("❌ Bot Token不能为空，请重新输入");
                continue;
            }

            if !token.contains(':') || token.len() < 35 {
                println!("⚠️ Bot Token格式可能不正确（格式: 数字:字母数字）");
                if !Self::confirm("确定要使用这个Token吗？(y/N)")? {
                    continue;
                }
            }

            println!("✅ Bot Token已保存");
            break token.to_string();
        };

        // 自动获取Chat ID
        println!("\n🔍 正在自动获取您的Chat ID...");
        println!("💡 请确保您已经向机器人发送了至少一条消息");

        let chat_id = match Self::get_chat_id_from_telegram(&bot_token).await {
            Ok(id) => {
                println!("✅ 成功获取到Chat ID: {}", id);
                id
            }
            Err(_) => {
                println!("⚠️ 无法自动获取Chat ID");
                println!("📝 请手动输入，或确保您已向机器人发送消息后重试");

                print!("💬 请输入您的Chat ID（可选，留空将尝试自动获取）: ");
                std::io::stdout().flush()?;

                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                input.trim().to_string()
            }
        };

        Ok((bot_token, chat_id))
    }

    /// 自动打开浏览器
    fn open_browser(url: &str) -> Result<()> {
        println!("🌐 正在打开浏览器: {}", url);

        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(url).spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(url).spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd").args(["/c", "start", url]).spawn()?;
        }

        // 给浏览器一些时间启动
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    /// 从Telegram API获取Chat ID
    async fn get_chat_id_from_telegram(bot_token: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/getUpdates", bot_token);

        let response = client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(result) = data.get("result") {
            if let Some(updates) = result.as_array() {
                if let Some(latest_update) = updates.last() {
                    if let Some(message) = latest_update.get("message") {
                        if let Some(chat) = message.get("chat") {
                            if let Some(chat_id) = chat.get("id") {
                                if let Some(id) = chat_id.as_i64() {
                                    return Ok(id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow!("无法获取Chat ID，请确保已向机器人发送消息"))
    }

    /// 保存配置到文件
    fn save_configuration(gemini_key: &str, bot_token: &str, chat_id: &str) -> Result<()> {
        println!("\n💾 正在保存配置...");

        // 读取现有配置
        let config_content = fs::read_to_string("config.toml")
            .unwrap_or_else(|_| include_str!("../config.toml").to_string());

        // 更新配置内容
        let updated_content = Self::update_config_content(&config_content, gemini_key, bot_token, chat_id)?;

        // 写入文件
        fs::write("config.toml", updated_content)?;

        println!("✅ 配置已保存到 config.toml");
        Ok(())
    }

    /// 更新配置文件内容
    fn update_config_content(content: &str, gemini_key: &str, bot_token: &str, chat_id: &str) -> Result<String> {
        let mut updated_lines = Vec::new();
        let mut in_ai_feedback_section = false;

        for line in content.lines() {
            let trimmed_line = line.trim();

            // 检查是否进入AI反馈配置段
            if trimmed_line == "[ai_feedback]" {
                in_ai_feedback_section = true;
                updated_lines.push(line.to_string());
                continue;
            }

            // 检查是否离开AI反馈配置段
            if trimmed_line.starts_with('[') && trimmed_line != "[ai_feedback]" && in_ai_feedback_section {
                in_ai_feedback_section = false;
            }

            if in_ai_feedback_section {
                if trimmed_line.starts_with("enabled") {
                    updated_lines.push("enabled = true                    # 系统启用状态".to_string());
                } else if trimmed_line.starts_with("gemini_api_key") {
                    updated_lines.push(format!("gemini_api_key = \"{}\"                # 您的Gemini API密钥", gemini_key));
                } else if trimmed_line.starts_with("telegram_bot_token") {
                    updated_lines.push(format!("telegram_bot_token = \"{}\"            # 您的Telegram Bot Token", bot_token));
                } else if trimmed_line.starts_with("telegram_chat_id") {
                    updated_lines.push(format!("telegram_chat_id = \"{}\"              # 自动获取的Chat ID", chat_id));
                } else {
                    updated_lines.push(line.to_string());
                }
            } else {
                updated_lines.push(line.to_string());
            }
        }

        Ok(updated_lines.join("\n"))
    }

    /// 测试配置
    async fn test_configuration(gemini_key: &str, bot_token: &str) -> Result<()> {
        println!("🔧 测试Gemini API连接...");

        // 简单的Gemini API测试
        let client = reqwest::Client::new();
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", gemini_key);

        let test_request = serde_json::json!({
            "contents": [{
                "parts": [{"text": "Hello, this is a test."}]
            }]
        });

        let response = client.post(&url)
            .json(&test_request)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Gemini API连接成功");
        } else {
            return Err(anyhow!("Gemini API测试失败: {}", response.status()));
        }

        println!("📱 测试Telegram Bot连接...");

        // 简单的Telegram Bot测试
        let telegram_url = format!("https://api.telegram.org/bot{}/getMe", bot_token);
        let telegram_response = client.get(&telegram_url).send().await?;

        if telegram_response.status().is_success() {
            println!("✅ Telegram Bot连接成功");
        } else {
            return Err(anyhow!("Telegram Bot测试失败: {}", telegram_response.status()));
        }

        Ok(())
    }

    /// 检查是否已有配置
    fn check_existing_config() -> Result<bool> {
        Ok(Path::new("config.toml").exists())
    }

    /// 显示当前配置状态
    pub fn show_current_config() -> Result<()> {
        println!("📋 当前AI反馈系统配置:");
        println!("--------------------------------------------------");

        if let Ok(content) = fs::read_to_string("config.toml") {
            let mut has_gemini = false;
            let mut has_telegram = false;
            let mut enabled = false;

            for line in content.lines() {
                let line = line.trim();
                if let Some((key, value)) = Self::parse_config_line(line) {
                    match key.as_str() {
                        "enabled" => enabled = value == "true",
                        "gemini_api_key" => has_gemini = !value.is_empty() && value != "\"\"",
                        "telegram_bot_token" => has_telegram = !value.is_empty() && value != "\"\"",
                        _ => {}
                    }
                }
            }

            println!("   🎯 AI反馈系统: {}", if enabled { "✅ 已启用" } else { "❌ 未启用" });
            println!("   🔑 Gemini API: {}", if has_gemini { "✅ 已配置" } else { "❌ 未配置" });
            println!("   📱 Telegram Bot: {}", if has_telegram { "✅ 已配置" } else { "❌ 未配置" });
        } else {
            println!("   ❌ 未找到配置文件");
        }
        Ok(())
    }

    /// 解析配置文件行
    fn parse_config_line(line: &str) -> Option<(String, String)> {
        if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
            return None;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            // 移除引号和注释
            let clean_value = if value.starts_with('\"') && value.contains('\"') {
                if let Some(end_quote) = value[1..].find('\"') {
                    value[1..end_quote + 1].to_string()
                } else {
                    value.to_string()
                }
            } else {
                // 处理行尾注释
                if let Some(comment_pos) = value.find('#') {
                    value[..comment_pos].trim().to_string()
                } else {
                    value.to_string()
                }
            };

            return Some((key.to_string(), clean_value));
        }

        None
    }

    /// 用户确认提示
    fn confirm(prompt: &str) -> Result<bool> {
        print!("{} ", prompt);
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" | "是" | "" => Ok(true), // 空输入默认为确认
            _ => Ok(false),
        }
    }

    /// 打印欢迎横幅
    fn print_welcome_banner() {
        println!("\n{}", "=".repeat(70));
        println!("🚀 欢迎使用 Prompter AI反馈系统配置向导");
        println!("   智能提示词分析 + 自动优化建议 + 每日Telegram推送");
        println!("{}", "=".repeat(70));
        println!();
        println!("✨ 系统特色:");
        println!("   🧠 使用Google Gemini思考模型深度分析您的提示词");
        println!("   ⚡ 配额不足时自动切换到快速模型，确保服务稳定");
        println!("   📱 每日通过Telegram推送个性化优化建议");
        println!("   🔄 支持多种监控模式：历史监控、会话监控、Shell监控");
        println!();
    }

    /// 打印成功消息
    fn print_success_message() {
        println!("\n🎉 配置完成！");
        println!("{}", "=".repeat(50));
        println!("✅ 您的AI反馈系统已成功配置并测试通过");
        println!("🎯 系统将智能分析您的Claude提示词并提供专业建议");
    }

    /// 打印后续步骤
    fn print_next_steps() {
        println!("\n📋 接下来您可以:");
        println!("   1. 🚀 启动AI分析服务:  ./prompter --auto-feedback");
        println!("   2. 📊 启动历史监控:    ./prompter --history-monitor");
        println!("   3. 🔍 启动会话监控:    ./prompter --session-monitor");
        println!("   4. 🧪 再次测试配置:    ./prompter --test-feedback");
        println!("   5. 📖 查看帮助信息:    ./prompter --help-feedback");
        println!();
        println!("💡 推荐先运行历史监控收集数据，再启动AI分析服务获得最佳效果！");
        println!("{}", "=".repeat(70));
    }
}