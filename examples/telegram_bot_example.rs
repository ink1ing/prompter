// Telegram Bot 命令处理示例
// 演示如何使用反馈间隔配置功能

use anyhow::Result;
use prompter::telegram_bot::{TelegramBot, TelegramConfig};
use prompter::feedback_interval::FeedbackInterval;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 Telegram Bot命令处理示例\n");

    // 读取配置
    let config_content = std::fs::read_to_string("config.toml")
        .expect("无法读取config.toml");

    let config: toml::Value = toml::from_str(&config_content)?;

    let bot_token = config
        .get("ai_feedback")
        .and_then(|af| af.get("telegram_bot_token"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let chat_id = config
        .get("ai_feedback")
        .and_then(|af| af.get("telegram_chat_id"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    if bot_token.is_empty() {
        println!("❌ 请先在config.toml中配置telegram_bot_token");
        println!("💡 运行 ./prompter --setup-feedback 进行配置");
        return Ok(());
    }

    let telegram_config = TelegramConfig {
        bot_token,
        chat_id,
        api_url: "https://api.telegram.org".to_string(),
    };

    println!("✅ Telegram配置加载成功\n");

    // 创建Bot实例
    let mut bot = TelegramBot::new(telegram_config)?;

    println!("📊 当前反馈间隔配置:");
    let interval = FeedbackInterval::load()?;
    println!("{}\n", interval.get_config_description());

    println!("🎧 启动命令监听器...");
    println!("💡 在Telegram中向机器人发送以下命令测试:");
    println!("   /based-on-time 12");
    println!("   /based-on-number 100");
    println!("   /status");
    println!("   /help\n");

    // 启动命令监听
    bot.start_command_listener().await?;

    Ok(())
}
