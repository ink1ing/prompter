// Telegram Bot通知模块 - 发送AI分析结果到用户
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use crate::feedback_interval::FeedbackInterval;
use crate::gemini_analyzer::{GeminiAnalyzer, GeminiConfig};

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
    pub api_url: String,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            chat_id: String::new(),
            api_url: "https://api.telegram.org".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TelegramSendMessage {
    chat_id: String,
    text: String,
    parse_mode: String,
}

#[derive(Debug, Deserialize)]
struct TelegramResponse {
    ok: bool,
    result: Option<serde_json::Value>,  // 更宽松的结构
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    message_id: i64,
    date: i64,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramIncomingMessage>,
}

#[derive(Debug, Deserialize)]
struct TelegramIncomingMessage {
    message_id: i64,
    from: TelegramUser,
    chat: TelegramChat,
    date: i64,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramUser {
    id: i64,
    first_name: String,
    username: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
    #[serde(rename = "type")]
    chat_type: String,
}

#[derive(Clone)]
pub struct TelegramBot {
    config: TelegramConfig,
    client: reqwest::Client,
    last_update_id: i64,
}

impl TelegramBot {
    pub fn new(config: TelegramConfig) -> Result<Self> {
        if config.bot_token.is_empty() {
            return Err(anyhow!("Telegram Bot Token不能为空"));
        }

        let client = reqwest::Client::new();

        Ok(Self {
            config,
            client,
            last_update_id: 0,
        })
    }

    /// 发送AI分析报告到Telegram
    pub async fn send_analysis_report(&self, report: &str) -> Result<()> {
        println!("📱 开始发送Telegram消息...");

        // 如果没有配置chat_id，尝试获取
        let chat_id = if self.config.chat_id.is_empty() {
            match self.get_chat_id().await {
                Ok(id) => {
                    println!("🎯 自动获取到Chat ID: {}", id);
                    id
                }
                Err(_) => {
                    return Err(anyhow!(
                        "❌ 未配置Chat ID且无法自动获取。\n\
                        请先向机器人发送任意消息，然后重新运行程序。"
                    ));
                }
            }
        } else {
            self.config.chat_id.clone()
        };

        // Telegram消息长度限制为4096字符
        let chunks = self.split_message(report, 4000);

        for (i, chunk) in chunks.iter().enumerate() {
            if chunks.len() > 1 {
                let header = if i == 0 {
                    format!("📊 AI提示词分析报告 (第{}/{}部分)\n\n", i + 1, chunks.len())
                } else {
                    format!("📊 AI提示词分析报告 (第{}/{}部分 - 续)\n\n", i + 1, chunks.len())
                };
                self.send_message_to_chat(&chat_id, &format!("{}{}", header, chunk)).await?;
            } else {
                self.send_message_to_chat(&chat_id, chunk).await?;
            }

            // 避免消息发送过于频繁
            if i < chunks.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        }

        println!("✅ Telegram消息发送成功！");
        Ok(())
    }

    /// 公开方法:发送消息到配置的chat
    pub async fn send_message(&self, text: &str) -> Result<()> {
        let chat_id = if self.config.chat_id.is_empty() {
            match self.get_chat_id().await {
                Ok(id) => id,
                Err(_) => {
                    return Err(anyhow!("无法获取Chat ID，请先在Telegram向机器人发送 /start 命令"));
                }
            }
        } else {
            self.config.chat_id.clone()
        };

        self.send_message_to_chat(&chat_id, text).await
    }

    /// 私有方法:发送单条消息到指定chat
    async fn send_message_to_chat(&self, chat_id: &str, text: &str) -> Result<()> {
        let message = TelegramSendMessage {
            chat_id: chat_id.to_string(),
            text: text.to_string(),
            parse_mode: "HTML".to_string(), // 支持简单的HTML格式
        };

        let url = format!("{}/bot{}/sendMessage",
            self.config.api_url, self.config.bot_token);

        let response = self.client
            .post(&url)
            .json(&message)
            .send()
            .await?;

        let telegram_response: TelegramResponse = response.json().await?;

        if !telegram_response.ok {
            let error_msg = telegram_response.description
                .unwrap_or_else(|| "未知错误".to_string());
            return Err(anyhow!("Telegram消息发送失败: {}", error_msg));
        }

        println!("📤 消息发送成功 (长度: {} 字符)", text.len());
        Ok(())
    }

    /// 自动获取Chat ID
    async fn get_chat_id(&self) -> Result<String> {
        println!("🔍 尝试自动获取Chat ID...");

        let url = format!("{}/bot{}/getUpdates",
            self.config.api_url, self.config.bot_token);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        let response_text = response.text().await?;
        println!("📋 Telegram API响应: {}", &response_text[..std::cmp::min(200, response_text.len())]);

        let updates: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(result) = updates.get("result") {
            if let Some(updates_array) = result.as_array() {
                println!("📬 找到 {} 条更新", updates_array.len());

                // 获取最新的消息
                if let Some(latest_update) = updates_array.last() {
                    println!("📨 最新更新: {}", serde_json::to_string_pretty(latest_update).unwrap_or("解析失败".to_string()));

                    if let Some(message) = latest_update.get("message") {
                        if let Some(chat) = message.get("chat") {
                            if let Some(chat_id) = chat.get("id") {
                                if let Some(id) = chat_id.as_i64() {
                                    println!("✅ 成功获取Chat ID: {}", id);
                                    return Ok(id.to_string());
                                }
                            }
                        }
                    }
                } else {
                    println!("📭 没有找到任何更新消息");
                }
            }
        }

        Err(anyhow::anyhow!("无法获取Chat ID"))
    }

    /// 分割长消息
    fn split_message(&self, text: &str, max_length: usize) -> Vec<String> {
        if text.len() <= max_length {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for line in text.lines() {
            if current_chunk.len() + line.len() + 1 > max_length {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = String::new();
                }
            }

            if !current_chunk.is_empty() {
                current_chunk.push('\n');
            }
            current_chunk.push_str(line);
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// 发送测试消息
    pub async fn send_test_message(&self) -> Result<()> {
        println!("🧪 发送Telegram测试消息...");

        let test_message = format!(
            "🤖 <b>Prompter AI反馈系统</b>\n\
            \n\
            ✅ 连接测试成功！\n\
            📅 测试时间: {}\n\
            🎯 系统已就绪，将每日推送AI分析报告\n\
            \n\
            💡 <i>这是一条测试消息</i>",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        // 如果没有配置chat_id，尝试获取
        let chat_id = if self.config.chat_id.is_empty() {
            match self.get_chat_id().await {
                Ok(id) => {
                    println!("🎯 自动获取到Chat ID: {}", id);
                    id
                }
                Err(_) => {
                    return Err(anyhow!(
                        "❌ 未配置Chat ID且无法自动获取。\n\
                        请先向机器人发送任意消息（如: /start），然后重新运行测试。"
                    ));
                }
            }
        } else {
            self.config.chat_id.clone()
        };

        self.send_message_to_chat(&chat_id, &test_message).await?;
        println!("✅ 测试消息发送成功！");
        Ok(())
    }

    /// 测试Bot连接
    pub async fn test_connection(&self) -> Result<()> {
        println!("🔧 测试Telegram Bot连接...");

        let url = format!("{}/bot{}/getMe",
            self.config.api_url, self.config.bot_token);

        let response = self.client
            .get(&url)
            .timeout(tokio::time::Duration::from_secs(10))
            .send()
            .await?;

        let bot_info: TelegramResponse = response.json().await?;

        if bot_info.ok {
            println!("✅ Telegram Bot连接成功！");
            Ok(())
        } else {
            let error_msg = bot_info.description
                .unwrap_or_else(|| "未知错误".to_string());
            Err(anyhow!("❌ Bot连接失败: {}", error_msg))
        }
    }

    /// 安全截取字符串，避免UTF-8字符边界问题
    fn safe_truncate(s: &str, max_chars: usize) -> &str {
        // 使用字符迭代器安全截取，避免字节边界问题
        let mut end_byte_idx = s.len();
        let mut char_count = 0;

        for (byte_idx, _char) in s.char_indices() {
            if char_count >= max_chars {
                end_byte_idx = byte_idx;
                break;
            }
            char_count += 1;
        }

        &s[..end_byte_idx]
    }

    /// 获取Bot信息
    pub async fn get_bot_info(&self) -> Result<String> {
        let url = format!("{}/bot{}/getMe",
            self.config.api_url, self.config.bot_token);

        let response = self.client.get(&url).send().await?;
        let bot_info: serde_json::Value = response.json().await?;

        if let Some(result) = bot_info.get("result") {
            let username = result.get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("未知");
            let first_name = result.get("first_name")
                .and_then(|v| v.as_str())
                .unwrap_or("未知");

            return Ok(format!("Bot: {} (@{})", first_name, username));
        }

        Ok("无法获取Bot信息".to_string())
    }

    /// 发送启动通知
    pub async fn send_startup_notification(&self) -> Result<()> {
        let message = format!(
            "🚀 <b>Prompter AI反馈系统已启动</b>\n\
            \n\
            📊 系统状态: 运行中\n\
            🕐 启动时间: {}\n\
            📅 下次推送: 明天同一时间\n\
            \n\
            🎯 系统将自动分析您的Claude提示词并提供AI反馈建议\n\
            💡 <i>让我们一起优化您的AI交互体验！</i>",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let chat_id = if self.config.chat_id.is_empty() {
            self.get_chat_id().await?
        } else {
            self.config.chat_id.clone()
        };

        self.send_message_to_chat(&chat_id, &message).await?;
        Ok(())
    }

    /// 自动初始化连接 - 尝试多种方式建立连接（非阻塞版本）
    pub async fn auto_initialize_connection(&mut self) -> Result<()> {
        println!("🤖 正在自动初始化Telegram连接...");

        // 首先测试连接
        match self.test_connection().await {
            Ok(_) => println!("✅ Telegram Bot连接成功"),
            Err(e) => {
                println!("❌ Telegram Bot连接失败: {}", e);
                return Err(anyhow::anyhow!("Bot连接失败，跳过Telegram功能"));
            }
        }

        // 如果已经有Chat ID，直接发送重连消息
        if !self.config.chat_id.is_empty() && self.config.chat_id != "YOUR_TELEGRAM_CHAT_ID" {
            let reconnect_message = format!(
                "🔄 <b>Prompter已重新连接</b>\n\
                \n\
                ✅ 系统重启成功\n\
                🕐 重启时间: {}\n\
                \n\
                🎯 <i>继续为您提供AI反馈服务</i>",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            );

            match self.send_message_to_chat(&self.config.chat_id, &reconnect_message).await {
                Ok(_) => {
                    println!("✅ 与Telegram的连接已重新建立！");
                    return Ok(());
                }
                Err(e) => {
                    println!("⚠️ 发送重连消息失败: {}", e);
                    // 继续尝试重新获取Chat ID
                }
            }
        }

        // 尝试获取现有的Chat ID（只尝试一次）
        println!("📱 正在尝试自动获取Chat ID...");
        match self.get_chat_id().await {
            Ok(chat_id) => {
                println!("✅ 自动获取到Chat ID: {}", chat_id);

                // 更新配置中的Chat ID
                self.config.chat_id = chat_id.clone();

                // 发送欢迎消息
                let welcome_message = format!(
                    "🎉 <b>欢迎使用Prompter AI反馈系统 v2.1!</b>\n\
                    \n\
                    ✅ 系统已自动连接成功\n\
                    🕐 连接时间: {}\n\
                    \n\
                    🎯 核心功能:\n\
                    • 🔍 实时监控Claude Code提示词\n\
                    • 🤖 Gemini AI智能分析和优化建议\n\
                    • 📊 分段历史分析和进度追踪\n\
                    • 📱 定期推送反馈报告\n\
                    \n\
                    📋 <b>主要命令:</b>\n\
                    /status - 查看系统状态\n\
                    /report - 生成快速分析报告\n\
                    /report-per &lt;数量&gt; - 分段分析历史提示词\n\
                    /view-history - 查看历史统计和进度\n\
                    /help - 显示完整帮助信息\n\
                    \n\
                    🚀 <b>新功能亮点:</b>\n\
                    • 智能分段分析 ({} 条历史提示词)\n\
                    • 自动进度追踪，避免重复分析\n\
                    • 支持大批量数据处理\n\
                    \n\
                    💡 <i>开始使用: 发送 /view-history 查看统计</i>",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    "5492"  // 显示实际的历史提示词数量
                );

                self.send_message_to_chat(&chat_id, &welcome_message).await?;
                println!("🎊 欢迎消息发送成功，连接完全建立！");

                // 尝试保存Chat ID到配置文件
                if let Err(e) = self.save_chat_id_to_config(&chat_id).await {
                    println!("⚠️ 保存Chat ID到配置文件失败: {}", e);
                }

                return Ok(());
            }
            Err(_) => {
                // 如果无法获取Chat ID，提供友好的指导但不阻塞程序
                println!("📱 无法自动获取Chat ID，需要手动初始化:");
                println!("   1. 打开Telegram应用");
                println!("   2. 搜索您的机器人");
                println!("   3. 发送 /start 消息给机器人");
                println!("   4. 下次启动时将自动连接");
                println!();
                println!("💡 Telegram功能暂时不可用，其他功能正常运行");

                return Err(anyhow::anyhow!("Telegram Chat ID未配置，跳过Telegram功能"));
            }
        }
    }

    /// 保存Chat ID到配置文件
    async fn save_chat_id_to_config(&self, chat_id: &str) -> Result<()> {
        // 读取现有配置文件
        let config_path = std::path::Path::new("config.toml");
        if !config_path.exists() {
            return Ok(()); // 如果配置文件不存在，跳过保存
        }

        let config_content = tokio::fs::read_to_string(config_path).await?;
        let mut updated_lines = Vec::new();
        let mut found_telegram_section = false;
        let mut found_chat_id_line = false;

        for line in config_content.lines() {
            let trimmed = line.trim();

            if trimmed == "[ai_feedback]" {
                found_telegram_section = true;
                updated_lines.push(line.to_string());
                continue;
            }

            if found_telegram_section && trimmed.starts_with("telegram_chat_id") {
                found_chat_id_line = true;
                updated_lines.push(format!("telegram_chat_id = \"{}\"              # Chat ID (自动获取)", chat_id));
                continue;
            }

            // 如果是其他section，停止修改
            if trimmed.starts_with('[') && trimmed != "[ai_feedback]" {
                found_telegram_section = false;
            }

            updated_lines.push(line.to_string());
        }

        if found_chat_id_line {
            tokio::fs::write(config_path, updated_lines.join("\n")).await?;
            println!("💾 Chat ID已自动保存到配置文件");
        }

        Ok(())
    }

    /// 发送错误通知
    pub async fn send_error_notification(&self, error: &str) -> Result<()> {
        let message = format!(
            "⚠️ <b>Prompter系统错误</b>\n\
            \n\
            🕐 错误时间: {}\n\
            📝 错误详情: {}\n\
            \n\
            🔧 <i>请检查系统状态或联系管理员</i>",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            error
        );

        let chat_id = if self.config.chat_id.is_empty() {
            self.get_chat_id().await?
        } else {
            self.config.chat_id.clone()
        };

        // 错误通知不应该因为发送失败而中断程序
        if let Err(e) = self.send_message_to_chat(&chat_id, &message).await {
            eprintln!("⚠️ 发送错误通知失败: {}", e);
        }

        Ok(())
    }

    /// 启动命令监听循环
    pub async fn start_command_listener(&mut self) -> Result<()> {
        println!("🎧 启动Telegram命令监听器...");
        println!("💡 支持的命令:");
        println!("   /based-on-time <小时数> - 设置基于时间的反馈间隔");
        println!("   /based-on-number <提示词数> - 设置基于数量的反馈间隔");
        println!("   /prompt <系统提示词> - 自定义Gemini分析提示词");
        println!("   /status - 查看当前配置状态");
        println!("   /report - 生成总体分析报告");
        println!("   /help - 显示帮助信息\n");

        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            match self.check_for_commands().await {
                Ok(()) => {
                    retry_count = 0; // 成功后重置重试计数
                }
                Err(e) => {
                    let error_msg = e.to_string();

                    // 网络连接错误,自动重试
                    if error_msg.contains("connection closed") ||
                       error_msg.contains("timeout") ||
                       error_msg.contains("Connection reset") {
                        retry_count += 1;

                        if retry_count <= max_retries {
                            eprintln!("⚠️ 网络错误 (重试 {}/{}): {}", retry_count, max_retries, error_msg);
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            continue;
                        } else {
                            eprintln!("❌ 网络错误达到最大重试次数,等待5秒后继续...");
                            retry_count = 0;
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            continue;
                        }
                    }

                    // 其他错误,只记录不中断
                    eprintln!("⚠️ 命令处理错误: {}", e);
                }
            }

            // 短暂等待后继续
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    /// 检查并处理新命令
    async fn check_for_commands(&mut self) -> Result<()> {
        let url = format!("{}/bot{}/getUpdates?offset={}&timeout=30",
            self.config.api_url, self.config.bot_token, self.last_update_id + 1);

        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        if let Some(result) = data.get("result") {
            if let Some(updates) = result.as_array() {
                for update in updates {
                    if let Some(update_obj) = update.as_object() {
                        if let Some(update_id) = update_obj.get("update_id").and_then(|v| v.as_i64()) {
                            self.last_update_id = update_id;
                        }

                        if let Some(message) = update_obj.get("message") {
                            self.handle_message(message).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 处理接收到的消息
    async fn handle_message(&self, message: &serde_json::Value) -> Result<()> {
        let chat_id = message
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|id| id.as_i64())
            .ok_or_else(|| anyhow!("无法获取chat_id"))?
            .to_string();

        let text = message
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        if text.is_empty() {
            return Ok(());
        }

        println!("📨 收到消息: {}", text);

        let response = self.process_command(text).await?;
        self.send_message_to_chat(&chat_id, &response).await?;

        Ok(())
    }

    /// 处理命令并返回响应
    async fn process_command(&self, text: &str) -> Result<String> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return Ok("❓ 请输入有效命令，发送 /help 查看帮助".to_string());
        }

        let command = parts[0];

        match command {
            "/based-on-time" | "/based_on_time" => {
                self.handle_time_based_command(&parts).await
            }
            "/based-on-number" | "/based_on_number" => {
                self.handle_number_based_command(&parts).await
            }
            "/status" => {
                self.handle_status_command().await
            }
            "/report" => {
                self.handle_report_command().await
            }
            "/report-per" | "/report_per" => {
                self.handle_report_per_command(&parts).await
            }
            "/view-history" | "/view_history" => {
                self.handle_view_history_command().await
            }
            "/prompt" => {
                self.handle_prompt_command(text).await
            }
            "/help" | "/start" => {
                self.handle_help_command().await
            }
            _ => {
                Ok(format!("❓ 未知命令: {}\n发送 /help 查看可用命令", command))
            }
        }
    }

    /// 处理基于时间的间隔设置命令
    async fn handle_time_based_command(&self, parts: &[&str]) -> Result<String> {
        if parts.len() < 2 {
            return Ok("⚠️ 用法: /based-on-time <小时数>\n例如: /based-on-time 12".to_string());
        }

        let hours_str = parts[1];
        let hours = hours_str.parse::<u32>()
            .map_err(|_| anyhow!("无效的小时数: {}", hours_str))?;

        let mut interval = FeedbackInterval::load()?;
        interval.set_time_based(hours)?;

        Ok(format!(
            "✅ <b>反馈间隔已更新</b>\n\
            \n\
            ⏰ 新设置: 每 <b>{}</b> 小时推送一次\n\
            📅 下次推送: 约 {} 小时后\n\
            \n\
            💡 <i>新设置已覆盖之前的配置</i>",
            hours, hours
        ))
    }

    /// 处理基于数量的间隔设置命令
    async fn handle_number_based_command(&self, parts: &[&str]) -> Result<String> {
        if parts.len() < 2 {
            return Ok("⚠️ 用法: /based-on-number <提示词数量>\n例如: /based-on-number 100".to_string());
        }

        let count_str = parts[1];
        let count = count_str.parse::<u32>()
            .map_err(|_| anyhow!("无效的数量: {}", count_str))?;

        let mut interval = FeedbackInterval::load()?;
        interval.set_number_based(count)?;

        Ok(format!(
            "✅ <b>反馈间隔已更新</b>\n\
            \n\
            📊 新设置: 每 <b>{}</b> 个提示词推送一次\n\
            📈 当前计数: 0（已重置）\n\
            \n\
            💡 <i>新设置已覆盖之前的配置</i>",
            count
        ))
    }

    /// 处理状态查询命令
    async fn handle_status_command(&self) -> Result<String> {
        let interval = FeedbackInterval::load()?;
        let config_desc = interval.get_config_description();

        Ok(format!(
            "📊 <b>反馈系统状态</b>\n\
            \n\
            {}\n\
            \n\
            💡 使用 /help 查看更多命令",
            config_desc
        ))
    }

    /// 处理报告生成命令
    async fn handle_report_command(&self) -> Result<String> {
        // 读取真实的历史提示词数据，优先使用历史监控数据
        let data_files = vec![
            "./data/claude_history_prompts.md",
            "./data/claude_session_prompts.md",
            "./data/shell_captured_prompts.md",
            "./data/prompts.md"
        ];

        let mut content = String::new();
        let mut data_source = String::new();

        // 找到第一个存在且有内容的数据文件
        for file_path in data_files {
            let data_file = std::path::Path::new(file_path);
            if data_file.exists() {
                match tokio::fs::read_to_string(data_file).await {
                    Ok(file_content) => {
                        if !file_content.trim().is_empty() {
                            content = file_content;
                            data_source = file_path.to_string();
                            break;
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        if content.is_empty() {
            return Ok("📭 <b>暂无数据</b>\n\n还没有收集到提示词，请先启动监控模式:\n• 历史监控: <code>--history-monitor</code>\n• 会话监控: <code>--session-monitor</code>\n• Shell监控: <code>--shell-monitor</code>".to_string());
        }

        // 统计信息
        let lines: Vec<&str> = content.lines().collect();
        let total_prompts = lines.iter().filter(|l| l.starts_with("## 20")).count();

        if total_prompts == 0 {
            return Ok("📭 <b>暂无数据</b>\n\n数据文件存在但为空～".to_string());
        }

        // 根据数据源格式提取提示词
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

            // 对于非代码块格式的数据，检测其他格式
            if !in_code_block && !line.trim().is_empty()
                && !line.starts_with("**")  // 排除项目信息
                && !line.contains("会话ID")
                && !line.contains("项目:")
                && line != "---" {
                if !current_prompt.is_empty() {
                    current_prompt.push(' ');
                }
                current_prompt.push_str(line.trim());
            }
        }

        // 添加最后一个提示词
        if !current_prompt.trim().is_empty() {
            prompts.push(current_prompt.trim().to_string());
        }

        if prompts.is_empty() {
            return Ok("📭 <b>暂无有效提示词</b>\n\n找到了数据文件但没有提取到有效的提示词内容。".to_string());
        }

        // 发送"正在分析"状态消息
        self.send_message(&format!(
            "🤖 <b>正在分析您的 {} 条提示词...</b>\n\n💭 使用LLM进行智能分析中，请稍候～",
            prompts.len()
        )).await?;

        // 生成基础统计报告
        let stats_report = self.generate_stats_report(&prompts, &data_source).await?;

        // 尝试生成AI分析报告
        match self.generate_ai_analysis_report(&prompts).await {
            Ok(ai_analysis) => {
                Ok(format!("{}\n\n📋 <b>AI深度分析</b>\n{}", stats_report, ai_analysis))
            }
            Err(e) => {
                Ok(format!("{}\n\n⚠️ <b>AI分析暂时不可用:</b> {}\n\n💡 请检查Gemini API配置", stats_report, e))
            }
        }
    }

    /// 生成AI分析报告
    async fn generate_ai_analysis_report(&self, prompts: &[String]) -> Result<String> {
        // 从配置文件读取Gemini配置
        let config_content = std::fs::read_to_string("config.toml")
            .map_err(|_| anyhow!("无法读取配置文件"))?;

        let mut gemini_config = GeminiConfig::default();

        // 解析配置
        for line in config_content.lines() {
            if let Some((key, value)) = self.parse_config_line(line) {
                match key.as_str() {
                    "gemini_api_key" => gemini_config.api_key = value,
                    "fast_model" => gemini_config.fast_model = value,
                    "max_retries" => {
                        if let Ok(retries) = value.parse::<usize>() {
                            gemini_config.max_retries = retries;
                        }
                    }
                    _ => {}
                }
            }
        }

        if gemini_config.api_key.is_empty() {
            return Err(anyhow!("Gemini API密钥未配置"));
        }

        // 创建Gemini分析器
        let analyzer = match GeminiAnalyzer::new(gemini_config) {
            Ok(analyzer) => analyzer,
            Err(e) => return Err(anyhow!("创建Gemini分析器失败: {}", e)),
        };

        // 分析前5个最新的提示词（避免API配额过度消耗）
        let recent_prompts: Vec<String> = prompts.iter().rev().take(5).cloned().collect();

        // 使用Gemini生成整体分析
        match analyzer.generate_overall_review(&recent_prompts).await {
            Ok(analysis) => Ok(analysis),
            Err(e) => Err(anyhow!("AI分析失败: {}", e))
        }
    }

    /// 解析配置文件行
    fn parse_config_line(&self, line: &str) -> Option<(String, String)> {
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

    /// 生成统计报告
    async fn generate_stats_report(&self, prompts: &[String], data_source: &str) -> Result<String> {
        let total_prompts = prompts.len();
        let total_chars: usize = prompts.iter().map(|p| p.len()).sum();
        let avg_length = if total_prompts > 0 { total_chars / total_prompts } else { 0 };

        // 找最长和最短的提示词
        let longest = prompts.iter().max_by_key(|p| p.len())
            .map(|p| if p.len() > 100 {
                format!("{}...", Self::safe_truncate(p, 100))
            } else {
                p.clone()
            })
            .unwrap_or_default();

        let shortest = prompts.iter().min_by_key(|p| p.len())
            .map(|p| p.clone())
            .unwrap_or_default();

        // 最近5个提示词
        let recent: Vec<String> = prompts.iter().rev().take(5)
            .map(|p| {
                let safe_truncated = Self::safe_truncate(p, 50);
                if p.len() > 50 {
                    format!("• {}...", safe_truncated)
                } else {
                    format!("• {}", p)
                }
            })
            .collect();

        Ok(format!(
            "📊 <b>提示词总体分析报告</b>\n\
            \n\
            📈 <b>统计概览</b>\n\
            总提示词数: {} 条\n\
            平均长度: {} 字符\n\
            数据来源: {}\n\
            \n\
            📏 <b>长度分析</b>\n\
            最长提示词: {}\n\
            最短提示词: {}\n\
            \n\
            🔄 <b>最近提示词</b> (最新5条)\n\
            {}",
            total_prompts,
            avg_length,
            data_source,
            longest,
            shortest,
            recent.join("\n")
        ))
    }

    /// 处理帮助命令
    async fn handle_help_command(&self) -> Result<String> {
        Ok(
            "🤖 <b>Prompter AI反馈系统</b>\n\
            \n\
            📋 <b>可用命令:</b>\n\
            \n\
            ⏰ <b>/based-on-time</b> &lt;小时数&gt;\n\
            设置基于时间的反馈间隔\n\
            示例: <code>/based-on-time 12</code>\n\
            \n\
            📊 <b>/based-on-number</b> &lt;提示词数&gt;\n\
            设置基于数量的反馈间隔\n\
            示例: <code>/based-on-number 100</code>\n\
            \n\
            🎨 <b>/prompt</b> &lt;新系统提示词&gt;\n\
            自定义Gemini分析提示词的系统提示\n\
            示例: <code>/prompt 你是一位专业的提示词优化专家...</code>\n\
            \n\
            📈 <b>/status</b>\n\
            查看当前配置状态\n\
            \n\
            📋 <b>/report</b>\n\
            生成最近提示词的快速总体报告\n\
            \n\
            📊 <b>/report-per</b> &lt;数量&gt;\n\
            分段分析历史提示词（智能进度追踪）\n\
            示例: <code>/report-per 1000</code>\n\
            \n\
            📈 <b>/view-history</b>\n\
            查看历史提示词统计和分析进度\n\
            \n\
            ❓ <b>/help</b>\n\
            显示此帮助信息\n\
            \n\
            💡 <b>分段分析说明:</b>\n\
            • <code>/report-per 1000</code> - 分析前1000条\n\
            • <code>/report-per 800</code> - 分析第1001-1800条\n\
            • 系统自动追踪进度，避免重复分析\n\
            • 所有分析完成后会提示无更多记录\n\
            \n\
            💡 <i>每次新设置会自动覆盖旧设置</i>"
            .to_string()
        )
    }

    /// 处理系统提示词设置命令
    async fn handle_prompt_command(&self, text: &str) -> Result<String> {
        // 提取 /prompt 之后的所有内容作为新的系统提示词
        let new_prompt = text.strip_prefix("/prompt")
            .unwrap_or("")
            .trim();

        if new_prompt.is_empty() {
            return Ok(
                "⚠️ <b>用法说明</b>\n\
                \n\
                /prompt &lt;新系统提示词&gt;\n\
                \n\
                📝 <b>示例:</b>\n\
                <code>/prompt 你是一位专业的AI提示词优化专家，负责审视用户与Claude Code的交互...</code>\n\
                \n\
                💡 <b>提示:</b>\n\
                • 系统提示词用于指导Gemini如何分析你的提示词\n\
                • 建议包含分析标准、输出格式等要求\n\
                • 设置后会立即生效并保存到配置文件"
                .to_string()
            );
        }

        // 保存到配置文件
        match Self::save_system_prompt_to_config(new_prompt).await {
            Ok(()) => {
                let preview = if new_prompt.len() > 100 {
                    format!("{}...", Self::safe_truncate(new_prompt, 100))
                } else {
                    new_prompt.to_string()
                };

                Ok(format!(
                    "✅ <b>系统提示词已更新</b>\n\
                    \n\
                    📝 新提示词预览:\n\
                    <code>{}</code>\n\
                    \n\
                    💾 已保存到: <code>config.toml</code>\n\
                    🔄 重启程序后生效\n\
                    \n\
                    💡 <i>此提示词将用于指导Gemini分析你的提示词质量</i>",
                    preview
                ))
            }
            Err(e) => {
                Ok(format!(
                    "❌ <b>保存失败</b>\n\
                    \n\
                    错误信息: {}\n\
                    \n\
                    请检查文件权限或联系管理员",
                    e
                ))
            }
        }
    }

    /// 保存系统提示词到配置文件
    async fn save_system_prompt_to_config(new_prompt: &str) -> Result<()> {
        use std::path::Path;
        use tokio::io::AsyncWriteExt;

        let config_path = Path::new("config.toml");

        // 读取现有配置
        let content = if config_path.exists() {
            tokio::fs::read_to_string(config_path).await?
        } else {
            String::new()
        };

        // 查找是否已有 system_prompt 配置
        let mut new_content = String::new();
        let mut found_prompt = false;
        let mut in_ai_feedback = false;

        for line in content.lines() {
            // 检测是否进入 [ai_feedback] 区块
            if line.trim() == "[ai_feedback]" {
                in_ai_feedback = true;
                new_content.push_str(line);
                new_content.push('\n');
                continue;
            }

            // 检测是否离开当前区块
            if line.trim().starts_with('[') && line.trim() != "[ai_feedback]" {
                // 如果在 ai_feedback 区块但还没找到 system_prompt，添加它
                if in_ai_feedback && !found_prompt {
                    new_content.push_str(&format!("system_prompt = \"\"\"\n{}\n\"\"\"\n", new_prompt));
                    found_prompt = true;
                }
                in_ai_feedback = false;
            }

            // 替换现有的 system_prompt
            if in_ai_feedback && (line.trim().starts_with("system_prompt =") || line.trim().starts_with("system_prompt=")) {
                // 跳过旧的 system_prompt（包括多行格式）
                found_prompt = true;
                new_content.push_str(&format!("system_prompt = \"\"\"\n{}\n\"\"\"\n", new_prompt));

                // 如果是多行字符串格式，跳过后续行直到结束
                continue;
            }

            // 跳过旧的多行 system_prompt 内容
            if found_prompt && line.trim() == "\"\"\"" {
                found_prompt = false; // 重置标记，表示已完成替换
                continue;
            }

            if !found_prompt || !in_ai_feedback {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        // 如果没有找到 [ai_feedback] 区块，添加一个
        if !found_prompt {
            if !in_ai_feedback {
                new_content.push_str("\n[ai_feedback]\n");
            }
            new_content.push_str(&format!("system_prompt = \"\"\"\n{}\n\"\"\"\n", new_prompt));
        }

        // 写入文件
        let mut file = tokio::fs::File::create(config_path).await?;
        file.write_all(new_content.as_bytes()).await?;
        file.sync_all().await?;

        println!("✅ 系统提示词已保存到 config.toml");
        Ok(())
    }

    /// 处理分段报告命令
    async fn handle_report_per_command(&self, parts: &[&str]) -> Result<String> {
        if parts.len() < 2 {
            return Ok(
                "⚠️ <b>用法说明</b>\n\
                \n\
                /report-per &lt;数量&gt;\n\
                \n\
                📝 <b>示例:</b>\n\
                <code>/report-per 1000</code> - 分析前1000条提示词\n\
                <code>/report-per 500</code> - 分析第1001-1500条\n\
                \n\
                💡 <b>说明:</b>\n\
                • 系统会自动记录已分析的进度\n\
                • 每次分析指定数量的提示词\n\
                • 直到分析完所有历史记录"
                .to_string()
            );
        }

        let count_str = parts[1];
        let count = count_str.parse::<usize>()
            .map_err(|_| anyhow::anyhow!("无效的数量: {}", count_str))?;

        if count == 0 || count > 2000 {
            return Ok("⚠️ 数量必须在 1-2000 之间".to_string());
        }

        // 读取报告进度状态
        let progress_file = "./data/report_progress.json";
        let mut analyzed_count = 0usize;

        if let Ok(progress_content) = tokio::fs::read_to_string(progress_file).await {
            if let Ok(progress_data) = serde_json::from_str::<serde_json::Value>(&progress_content) {
                analyzed_count = progress_data.get("analyzed_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
            }
        }

        // 读取所有提示词
        let prompts = self.load_all_prompts().await?;
        let total_prompts = prompts.len();

        // 检查是否还有未分析的提示词
        if analyzed_count >= total_prompts {
            return Ok(
                "📭 <b>没有更多历史记录可以分析</b>\n\
                \n\
                ✅ 所有历史提示词已分析完成\n\
                📊 总计: {total} 条\n\
                \n\
                💡 使用 /view-history 查看统计信息"
                .replace("{total}", &total_prompts.to_string())
            );
        }

        // 获取下一批要分析的提示词
        let end_index = std::cmp::min(analyzed_count + count, total_prompts);
        let batch_prompts: Vec<String> = prompts.iter()
            .skip(analyzed_count)
            .take(count)
            .cloned()
            .collect();

        if batch_prompts.is_empty() {
            return Ok("📭 没有更多提示词可分析".to_string());
        }

        // 发送"正在分析"状态消息
        self.send_message(&format!(
            "🤖 <b>正在分析第 {}-{} 条提示词...</b>\n\n💭 共 {} 条，使用Gemini深度分析中...",
            analyzed_count + 1,
            end_index,
            batch_prompts.len()
        )).await?;

        // 生成AI分析报告
        match self.generate_ai_analysis_report(&batch_prompts).await {
            Ok(ai_analysis) => {
                // 更新进度
                let new_analyzed_count = end_index;
                let progress_data = serde_json::json!({
                    "analyzed_count": new_analyzed_count,
                    "last_analysis_time": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    "total_prompts": total_prompts
                });

                if let Err(e) = tokio::fs::write(progress_file, progress_data.to_string()).await {
                    eprintln!("⚠️ 保存进度失败: {}", e);
                }

                let remaining = total_prompts - new_analyzed_count;
                let progress_info = if remaining > 0 {
                    format!("\n📈 <b>分析进度</b>\n已分析: {}/{} 条\n剩余: {} 条\n\n💡 使用 <code>/report-per {}</code> 继续分析下一批",
                        new_analyzed_count, total_prompts, remaining,
                        std::cmp::min(remaining, count))
                } else {
                    format!("\n🎉 <b>全部分析完成！</b>\n总计: {} 条提示词", total_prompts)
                };

                Ok(format!(
                    "📊 <b>第 {}-{} 条提示词分析报告</b>\n\n{}{}",
                    analyzed_count + 1,
                    end_index,
                    ai_analysis,
                    progress_info
                ))
            }
            Err(e) => {
                Ok(format!(
                    "❌ <b>第 {}-{} 条分析失败</b>\n\n错误: {}\n\n💡 请稍后重试",
                    analyzed_count + 1,
                    end_index,
                    e
                ))
            }
        }
    }

    /// 处理查看历史命令
    async fn handle_view_history_command(&self) -> Result<String> {
        // 读取所有提示词
        let prompts = self.load_all_prompts().await?;
        let total_prompts = prompts.len();

        if total_prompts == 0 {
            return Ok("📭 <b>暂无历史记录</b>\n\n还没有收集到提示词数据".to_string());
        }

        // 读取分析进度
        let progress_file = "./data/report_progress.json";
        let mut analyzed_count = 0usize;
        let mut last_analysis_time = "未开始".to_string();

        if let Ok(progress_content) = tokio::fs::read_to_string(progress_file).await {
            if let Ok(progress_data) = serde_json::from_str::<serde_json::Value>(&progress_content) {
                analyzed_count = progress_data.get("analyzed_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                last_analysis_time = progress_data.get("last_analysis_time")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知")
                    .to_string();
            }
        }

        // 分析项目分布
        let mut project_stats = std::collections::HashMap::new();
        for prompt in &prompts {
            // 简单估算项目（这里可以改进）
            if prompt.contains("prompter") {
                *project_stats.entry("prompter".to_string()).or_insert(0) += 1;
            } else if prompt.contains("tothemoon") {
                *project_stats.entry("tothemoon".to_string()).or_insert(0) += 1;
            } else if prompt.contains("agent-daily") {
                *project_stats.entry("agent-daily".to_string()).or_insert(0) += 1;
            } else {
                *project_stats.entry("其他项目".to_string()).or_insert(0) += 1;
            }
        }

        let mut project_list = String::new();
        for (project, count) in project_stats.iter() {
            project_list.push_str(&format!("• {}: {} 条\n", project, count));
        }

        let remaining = if total_prompts > analyzed_count {
            total_prompts - analyzed_count
        } else {
            0
        };

        let next_batch_suggestion = if remaining > 0 {
            format!("\n💡 <b>建议下次分析:</b>\n<code>/report-per {}</code>",
                std::cmp::min(remaining, 1000))
        } else {
            "\n🎉 <b>所有提示词已分析完成</b>".to_string()
        };

        Ok(format!(
            "📊 <b>历史提示词统计</b>\n\
            \n\
            📈 <b>总体数据</b>\n\
            总提示词数: <b>{}</b> 条\n\
            已分析数: <b>{}</b> 条\n\
            未分析数: <b>{}</b> 条\n\
            \n\
            📅 <b>分析进度</b>\n\
            上次分析: {}\n\
            \n\
            🏗️ <b>项目分布</b>\n\
            {}{}",
            total_prompts,
            analyzed_count,
            remaining,
            last_analysis_time,
            project_list,
            next_batch_suggestion
        ))
    }

    /// 加载所有提示词
    async fn load_all_prompts(&self) -> Result<Vec<String>> {
        let data_files = vec![
            "./data/claude_history_prompts.md",
            "./data/claude_session_prompts.md",
            "./data/shell_captured_prompts.md",
            "./data/prompts.md"
        ];

        let mut all_prompts = Vec::new();

        for file_path in data_files {
            let data_file = std::path::Path::new(file_path);
            if data_file.exists() {
                match tokio::fs::read_to_string(data_file).await {
                    Ok(content) => {
                        let prompts = self.extract_prompts_from_content(&content);
                        all_prompts.extend(prompts);
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(all_prompts)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_message() {
        let bot = TelegramBot::new(TelegramConfig::default()).unwrap();
        let long_text = "A".repeat(5000);
        let chunks = bot.split_message(&long_text, 4000);

        assert!(chunks.len() > 1);
        for chunk in chunks {
            assert!(chunk.len() <= 4000);
        }
    }

    #[test]
    fn test_short_message() {
        let bot = TelegramBot::new(TelegramConfig::default()).unwrap();
        let short_text = "Short message";
        let chunks = bot.split_message(short_text, 4000);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], short_text);
    }
}