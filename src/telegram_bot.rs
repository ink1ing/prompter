// Telegram Bot通知模块 - 发送AI分析结果到用户
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use crate::feedback_interval::FeedbackInterval;

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

        let updates: HashMap<String, serde_json::Value> = response.json().await?;

        if let Some(result) = updates.get("result") {
            if let Some(updates_array) = result.as_array() {
                // 获取最新的消息
                if let Some(latest_update) = updates_array.last() {
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

        Err(anyhow!("无法获取Chat ID"))
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
        println!("   /status - 查看当前配置状态");
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
        // 读取所有历史提示词
        let data_file = std::path::Path::new("./data/prompts.md");

        if !data_file.exists() {
            return Ok("📭 <b>暂无数据</b>\n\n还没有收集到提示词，请继续使用Claude Code～".to_string());
        }

        let content = tokio::fs::read_to_string(data_file).await
            .map_err(|e| anyhow!("读取文件失败: {}", e))?;

        // 统计信息
        let lines: Vec<&str> = content.lines().collect();
        let total_prompts = lines.iter().filter(|l| l.starts_with("## 20")).count();

        if total_prompts == 0 {
            return Ok("📭 <b>暂无数据</b>\n\n数据文件存在但为空～".to_string());
        }

        // 提取所有提示词
        let mut prompts = Vec::new();
        let mut current_prompt = String::new();
        let mut in_prompt = false;

        for line in lines {
            if line.starts_with("## 20") {
                if !current_prompt.is_empty() {
                    prompts.push(current_prompt.clone());
                    current_prompt.clear();
                }
                in_prompt = true;
            } else if line == "---" {
                in_prompt = false;
            } else if in_prompt && !line.is_empty() {
                current_prompt.push_str(line);
                current_prompt.push(' ');
            }
        }
        if !current_prompt.is_empty() {
            prompts.push(current_prompt);
        }

        // 计算平均长度
        let total_chars: usize = prompts.iter().map(|p| p.len()).sum();
        let avg_length = if !prompts.is_empty() {
            total_chars / prompts.len()
        } else {
            0
        };

        // 找最长和最短的提示词
        let longest = prompts.iter().max_by_key(|p| p.len())
            .map(|p| if p.len() > 100 { format!("{}...", &p[..100]) } else { p.clone() })
            .unwrap_or_default();

        let shortest = prompts.iter().min_by_key(|p| p.len())
            .map(|p| p.clone())
            .unwrap_or_default();

        // 最近5个提示词
        let recent: Vec<String> = prompts.iter().rev().take(5)
            .map(|p| {
                if p.len() > 50 {
                    format!("• {}...", &p[..50])
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
            数据文件: ./data/prompts.md\n\
            \n\
            📏 <b>长度分析</b>\n\
            最长提示词: {}\n\
            最短提示词: {}\n\
            \n\
            🔄 <b>最近提示词</b> (最新5条)\n\
            {}\n\
            \n\
            💡 <i>提示: 使用 /based-on-number 设置自动分析频率</i>",
            total_prompts,
            avg_length,
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
            📈 <b>/status</b>\n\
            查看当前配置状态\n\
            \n\
            📋 <b>/report</b>\n\
            生成所有历史提示词总体报告\n\
            \n\
            ❓ <b>/help</b>\n\
            显示此帮助信息\n\
            \n\
            💡 <i>每次新设置会自动覆盖旧设置</i>"
            .to_string()
        )
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