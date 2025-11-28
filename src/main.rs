use anyhow::Result;
use chrono::{DateTime, Local};
use clap::Parser;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use regex::Regex;
use std::io::{Write};
use std::fs::OpenOptions;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use serde::Deserialize;

use chinese_filter::ChineseFilter;
use cloudflare_uploader::CloudflareConfig;
use auto_scheduler::{AutoScheduler, AutoConfig};
use shell_monitor::{ShellMonitor, ShellMonitorConfig};

// 导入新的模块
mod api_key_manager;
use api_key_manager::{ApiKeyManager, ApiKeyConfig};

#[derive(Debug, Deserialize)]
struct Config {
    app: AppConfig,
    filter: FilterConfig,
    cloudflare: CloudflareConfigToml,
    website: WebsiteConfig,
    storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    auto_upload_enabled: bool,
    upload_interval_hours: u64,
}

#[derive(Debug, Deserialize)]
struct FilterConfig {
    detect_chinese: bool,
    min_chinese_chars: usize,
    exclude_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CloudflareConfigToml {
    account_id: String,
    zone_id: String,
    api_token: String,
}

#[derive(Debug, Deserialize)]
struct WebsiteConfig {
    domain: String,
    upload_endpoint: String,
    github_repo: String,
}

#[derive(Debug, Deserialize)]
struct StorageConfig {
    data_dir: String,
    prompts_file: String,
    backup_dir: String,
    max_backups: usize,
}

mod simple_monitor;
mod benchmark;
mod chinese_filter;
mod cloudflare_uploader;
mod auto_scheduler;
mod shell_monitor;
mod session_monitor;
mod claude_history_monitor;
mod claude_jsonl_monitor;
mod ai_feedback_config;
mod gemini_analyzer;
mod perplexity_analyzer;
mod llm_selector;
mod telegram_bot;
mod feedback_interval;
mod global_process_monitor;

#[derive(Parser)]
#[command(name = "prompter")]
#[command(about = "Monitor Claude Code CLI prompts and save them to a file")]
struct Args {
    /// Path to save prompts (default: ./prompts.md)
    #[arg(short, long, default_value = "prompts.md")]
    output: PathBuf,

    /// Claude code command to run (default: claude)
    #[arg(short, long, default_value = "claude")]
    command: String,

    /// Use simple monitor mode (direct input capture)
    #[arg(short, long)]
    simple: bool,

    /// Run performance benchmark
    #[arg(long)]
    benchmark: bool,

    /// Enable auto-upload mode with hourly sync to Cloudflare
    #[arg(long)]
    auto: bool,

    /// Configuration file path
    #[arg(short = 'c', long, default_value = "config.toml")]
    config: PathBuf,

    /// Manual upload trigger
    #[arg(long)]
    upload: bool,

    /// Enable independent shell monitoring mode (no PTY wrapping)
    #[arg(long)]
    shell_monitor: bool,

    /// Enable Claude Code session monitoring mode (monitor active processes)
    #[arg(long)]
    session_monitor: bool,

    /// Enable Claude Code history monitoring mode (monitor ~/.claude/history.jsonl)
    #[arg(long)]
    history_monitor: bool,

    /// Enable Claude Code JSONL monitoring mode (monitor ~/.claude/projects/*.jsonl)
    #[arg(long)]
    jsonl_monitor: bool,

    /// Enable AI feedback system with automatic analysis and Telegram notifications
    #[arg(long)]
    auto_feedback: bool,

    /// Configure AI feedback system (setup wizard)
    #[arg(long)]
    setup_feedback: bool,

    /// Test AI feedback system configuration
    #[arg(long)]
    test_feedback: bool,

    /// Start Telegram bot command listener
    #[arg(long)]
    telegram_listen: bool,

    /// Enable global process monitoring mode (monitor all terminal Claude processes)
    #[arg(long)]
    global_monitor: bool,

    /// Show AI feedback system help and usage examples
    #[arg(long)]
    help_feedback: bool,
}

#[derive(Debug)]
struct PromptEntry {
    timestamp: DateTime<Local>,
    content: String,
}

impl PromptEntry {
    fn new(content: String) -> Self {
        Self {
            timestamp: Local::now(),
            content,
        }
    }

    fn to_markdown(&self) -> String {
        format!(
            "## {}\n\n```\n{}\n```\n\n",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.content
        )
    }
}

struct PrompterApp {
    output_file: PathBuf,
    claude_command: String,
    prompt_regex: Regex,
}

impl PrompterApp {
    fn new(output_file: PathBuf, claude_command: String) -> Result<Self> {
        // 正则表达式用于识别用户输入的提示词
        // 这里需要根据Claude Code的具体输出格式调整
        let prompt_regex = Regex::new(r"^[>\$]\s+(.+)$")?;

        Ok(Self {
            output_file,
            claude_command,
            prompt_regex,
        })
    }

    async fn run(&self) -> Result<()> {
        println!("🚀 Prompter started - monitoring Claude Code CLI");
        println!("📝 Saving prompts to: {}", self.output_file.display());

        // 创建伪终端
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // 启动Claude Code进程
        let cmd = CommandBuilder::new(&self.claude_command);
        let mut child = pty_pair.slave.spawn_command(cmd)?;

        // 获取PTY的读写端（暂未使用）
        let _reader = pty_pair.master.try_clone_reader()?;
        let _writer = pty_pair.master.take_writer()?;

        // 简化版本：直接监控stdin并启动Claude Code
        println!("⚠️  PTY模式功能正在开发中，建议使用 --simple 模式");
        println!("启动命令: {} --simple", &self.claude_command);

        // 主输入循环 - 简化版本
        let stdin = tokio::io::stdin();
        let mut stdin_reader = tokio::io::BufReader::new(stdin);
        let mut input_line = String::new();

        loop {
            print!("Your prompt: ");
            std::io::stdout().flush().unwrap();

            input_line.clear();
            match stdin_reader.read_line(&mut input_line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = input_line.trim();
                    if trimmed == "quit" || trimmed == "exit" {
                        break;
                    }

                    if !trimmed.is_empty() && !trimmed.starts_with('/') {
                        let entry = PromptEntry::new(trimmed.to_string());
                        self.save_prompt(&entry).await?;
                        println!("✅ 已保存提示词");
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }
        }

        // 等待子进程结束
        let _ = child.wait();

        println!("\n📋 Session ended. Prompts saved to: {}", self.output_file.display());

        Ok(())
    }

    async fn save_prompt(&self, entry: &PromptEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_file)?;

        writeln!(file, "{}", entry.to_markdown())?;

        println!("💾 Saved prompt: {}", entry.content);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.benchmark {
        // 运行性能测试
        benchmark::benchmark_direct_vs_pty()?;
        benchmark::test_io_throughput()?;
    } else if args.auto {
        // 启动自动化模式
        run_auto_mode(&args).await?;
    } else if args.upload {
        // 手动上传模式
        run_manual_upload(&args).await?;
    } else if args.shell_monitor {
        // 独立Shell监控模式
        run_shell_monitor_mode(&args).await?;
    } else if args.session_monitor {
        // Claude Code会话监控模式
        run_session_monitor_mode(&args).await?;
    } else if args.jsonl_monitor {
        // Claude Code JSONL监控模式
        run_jsonl_monitor_mode(&args).await?;
    } else if args.history_monitor {
        // Claude Code历史监控模式
        run_history_monitor_mode(&args).await?;
    } else if args.setup_feedback {
        // AI反馈系统配置向导
        run_setup_feedback_wizard(&args).await?;
    } else if args.test_feedback {
        // 测试AI反馈系统
        run_test_feedback(&args).await?;
    } else if args.telegram_listen {
        // 启动Telegram命令监听器
        run_telegram_listener(&args).await?;
    } else if args.global_monitor {
        // 启动全局进程监控模式
        run_global_process_monitor(&args).await?;
    } else if args.help_feedback {
        // 显示AI反馈系统帮助
        show_ai_feedback_help();
    } else if args.auto_feedback {
        // AI自动反馈模式
        run_auto_feedback_mode(&args).await?;
    } else if args.simple {
        // 使用简化版本
        simple_monitor::run_simple_monitor()?;
    } else {
        // 使用完整版本（PTY监控）
        let app = PrompterApp::new(args.output, args.command)?;
        app.run().await?;
    }

    Ok(())
}

/// 自动化模式 - 启动定时上传
async fn run_auto_mode(args: &Args) -> Result<()> {
    println!("🤖 启动自动化模式...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 验证配置
    if !validate_config(&config) {
        anyhow::bail!("配置文件验证失败，请检查 config.toml");
    }

    // 创建中文过滤器
    let chinese_filter = ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    // 创建Cloudflare配置
    let cloudflare_config = CloudflareConfig {
        account_id: config.cloudflare.account_id,
        zone_id: config.cloudflare.zone_id,
        api_token: config.cloudflare.api_token,
        domain: config.website.domain,
        upload_endpoint: config.website.upload_endpoint,
        github_repo: config.website.github_repo,
    };

    // 创建自动化配置
    let auto_config = AutoConfig {
        upload_interval_hours: config.app.upload_interval_hours,
        data_dir: config.storage.data_dir,
        prompts_file: config.storage.prompts_file,
        backup_dir: config.storage.backup_dir,
        max_backups: config.storage.max_backups,
    };

    // 创建并启动调度器
    let mut scheduler = AutoScheduler::new(auto_config, cloudflare_config, chinese_filter).await?;
    scheduler.start().await?;

    println!("🎯 按 Ctrl+C 停止自动化服务");

    // 等待中断信号
    tokio::signal::ctrl_c().await?;

    println!("⏹️  正在停止自动化服务...");
    scheduler.stop().await?;

    Ok(())
}

/// 手动上传模式
async fn run_manual_upload(args: &Args) -> Result<()> {
    println!("🚀 手动上传模式...");

    let config = load_config(&args.config)?;

    if !validate_config(&config) {
        anyhow::bail!("配置文件验证失败，请检查 config.toml");
    }

    let chinese_filter = ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    let cloudflare_config = CloudflareConfig {
        account_id: config.cloudflare.account_id,
        zone_id: config.cloudflare.zone_id,
        api_token: config.cloudflare.api_token,
        domain: config.website.domain,
        upload_endpoint: config.website.upload_endpoint,
        github_repo: config.website.github_repo,
    };

    let auto_config = AutoConfig {
        upload_interval_hours: config.app.upload_interval_hours,
        data_dir: config.storage.data_dir,
        prompts_file: config.storage.prompts_file,
        backup_dir: config.storage.backup_dir,
        max_backups: config.storage.max_backups,
    };

    let scheduler = AutoScheduler::new(auto_config, cloudflare_config, chinese_filter).await?;
    let count = scheduler.manual_upload().await?;

    if count > 0 {
        println!("✅ 手动上传完成: {}条中文提示词", count);
    } else {
        println!("ℹ️  没有新的中文提示词需要上传");
    }

    Ok(())
}

/// 加载配置文件
fn load_config(config_path: &PathBuf) -> Result<Config> {
    if !config_path.exists() {
        create_default_config(config_path)?;
        println!("📝 已创建默认配置文件: {}", config_path.display());
        println!("⚠️  请编辑配置文件后重新运行");
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// 创建默认配置文件
fn create_default_config(config_path: &PathBuf) -> Result<()> {
    let default_content = std::fs::read_to_string("config.toml")
        .unwrap_or_else(|_| include_str!("../config.toml").to_string());

    std::fs::write(config_path, default_content)?;
    Ok(())
}

/// 验证配置文件
fn validate_config(config: &Config) -> bool {
    // 检查必填字段
    if config.cloudflare.account_id.is_empty() ||
       config.cloudflare.zone_id.is_empty() ||
       config.cloudflare.api_token.is_empty() {
        println!("❌ Cloudflare配置不完整");
        return false;
    }

    if config.website.domain.is_empty() ||
       config.website.github_repo.is_empty() {
        println!("❌ 网站配置不完整");
        return false;
    }

    true
}

/// Shell监控模式 - 独立终端监控
async fn run_shell_monitor_mode(args: &Args) -> Result<()> {
    println!("🔍 启动独立Shell监控模式...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 创建中文过滤器
    let chinese_filter = ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    // 创建Shell监控配置
    let shell_config = ShellMonitorConfig::default();

    // 创建Shell监控器
    let mut shell_monitor = ShellMonitor::new(shell_config, chinese_filter)?;

    println!("🎯 独立Shell监控已启动，正在监控以下Claude命令:");
    println!("  - claude");
    println!("  - claude-code");
    println!("  - claude-cli");
    println!("📁 中文提示词将保存到: ./data/shell_captured_prompts.md");
    println!("⏰ 每5秒检查一次历史文件变化");
    println!("");
    println!("🔧 如需结合自动上传功能，请使用: --auto --shell-monitor");
    println!("⏹️  按 Ctrl+C 停止监控");

    // 启动监控服务
    shell_monitor.start_independent_monitoring().await?;

    // 等待中断信号
    tokio::signal::ctrl_c().await?;
    println!("⏹️  Shell监控服务已停止");

    Ok(())
}

/// Claude Code会话监控模式 - 监控活跃进程交互
async fn run_session_monitor_mode(args: &Args) -> Result<()> {
    println!("🎯 启动Claude Code会话监控模式...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 创建中文过滤器
    let chinese_filter = ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    // 创建会话监控器
    let mut session_monitor = session_monitor::SessionMonitor::new(chinese_filter)?;

    println!("🔍 会话监控已启动，正在监控Claude Code进程交互:");
    println!("  - 活跃Claude进程检测");
    println!("  - tmux会话监控");
    println!("  - 进程输入输出捕获");
    println!("📁 中文提示词将保存到: ./data/claude_session_prompts.md");
    println!();
    println!("🔧 如需结合自动上传功能，请使用: --auto --session-monitor");
    println!("⏹️  按 Ctrl+C 停止监控");
    println!();

    // 启动会话监控
    session_monitor.start_session_monitoring().await?;

    // 等待中断信号
    tokio::signal::ctrl_c().await?;
    println!("⏹️  会话监控服务已停止");

    Ok(())
}

/// Claude Code JSONL监控模式 - 监控~/.claude/projects/*.jsonl
async fn run_jsonl_monitor_mode(args: &Args) -> Result<()> {
    println!("📋 启动Claude Code JSONL监控模式...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 创建中文过滤器
    let chinese_filter = ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    // 创建JSONL监控器
    let mut jsonl_monitor = claude_jsonl_monitor::ClaudeJsonlMonitor::new(chinese_filter)?;

    println!("📝 JSONL监控已启动，正在监控Claude Code项目交互:");
    println!("  - ~/.claude/projects/*.jsonl 实时监控");
    println!("  - JSON消息解析和提取");
    println!("  - 用户输入中文提示词捕获");
    println!("📁 中文提示词将保存到: ./data/claude_session_prompts.md");
    println!();
    println!("🔧 如需结合自动上传功能，请使用: --auto --jsonl-monitor");
    println!("⏹️  按 Ctrl+C 停止监控");
    println!();

    // 启动JSONL监控
    jsonl_monitor.start_jsonl_monitoring().await?;

    // 等待中断信号
    tokio::signal::ctrl_c().await?;
    println!("⏹️  JSONL监控服务已停止");

    Ok(())
}

/// Claude Code历史监控模式 - 监控~/.claude/history.jsonl
async fn run_history_monitor_mode(args: &Args) -> Result<()> {
    println!("📚 启动Claude Code历史监控模式...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 创建中文过滤器
    let chinese_filter = ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    // 创建历史监控器
    let mut history_monitor = claude_history_monitor::ClaudeHistoryMonitor::new(chinese_filter)?;

    println!("📝 历史监控已启动，正在监控Claude Code交互记录:");
    println!("  - ~/.claude/history.jsonl 文件监控");
    println!("  - 实时JSON记录解析");
    println!("  - 用户输入提示词捕获");
    println!("📁 中文提示词将保存到: ./data/claude_history_prompts.md");
    println!();
    println!("🔧 如需结合自动上传功能，请使用: --auto --history-monitor");
    println!("⏹️  按 Ctrl+C 停止监控");
    println!();

    // 启动历史监控
    history_monitor.start_history_monitoring().await?;

    // 等待中断信号
    tokio::signal::ctrl_c().await?;
    println!("⏹️  历史监控服务已停止");

    Ok(())
}

/// AI反馈系统配置向导
async fn run_setup_feedback_wizard(_args: &Args) -> Result<()> {
    println!("🤖 启动AI反馈系统配置向导...");
    ai_feedback_config::ConfigWizard::start_setup_wizard().await?;
    Ok(())
}

/// 测试AI反馈系统配置
async fn run_test_feedback(args: &Args) -> Result<()> {
    println!("🧪 测试AI反馈系统配置...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 检查AI反馈配置
    let ai_config = load_ai_feedback_config(&config)?;

    println!("📋 当前配置:");
    ai_feedback_config::ConfigWizard::show_current_config()?;

    // 先测试Telegram Bot（更简单）
    println!("\n📱 测试Telegram Bot连接...");
    let telegram_config = telegram_bot::TelegramConfig {
        bot_token: ai_config.telegram_bot_token.clone(),
        chat_id: ai_config.telegram_chat_id.clone(),
        ..Default::default()
    };

    let bot = telegram_bot::TelegramBot::new(telegram_config)?;
    bot.test_connection().await?;

    // 发送测试消息
    if confirm_action("是否发送测试消息到Telegram？").is_ok() {
        bot.send_test_message().await?;
        println!("✅ Telegram测试消息发送成功！");
    }

    // 创建Gemini分析器
    println!("\n🤖 测试Gemini API连接...");
    let gemini_config = gemini_analyzer::GeminiConfig {
        api_key: ai_config.gemini_api_key.clone(),
        fast_model: ai_config.fast_model.clone(),
        max_retries: ai_config.max_retries,
        system_prompt: ai_config.system_prompt.clone(),
        ..Default::default()
    };

    let analyzer = gemini_analyzer::GeminiAnalyzer::new(gemini_config)?;
    match analyzer.test_connection().await {
        Ok(_) => println!("✅ Gemini API连接成功"),
        Err(e) => {
            println!("⚠️ Gemini API连接失败: {}", e);
            println!("💡 您可以继续使用，API问题不影响基本功能");
        }
    }

    println!("\n✅ 基础测试完成！系统已基本配置正确。");
    Ok(())
}

/// 启动Telegram命令监听器
async fn run_telegram_listener(args: &Args) -> Result<()> {
    println!("🎧 Telegram Bot 命令监听器");
    println!("{}", "=".repeat(50));
    println!();

    // 加载配置
    let config = load_config(&args.config)?;
    let mut ai_config = load_ai_feedback_config(&config)?;

    // 检测配置状态
    let has_gemini = !ai_config.gemini_api_key.is_empty()
        && ai_config.gemini_api_key != "YOUR_GEMINI_API_KEY"
        && ai_config.gemini_api_key != "YOUR_NEW_API_KEY_HERE";  // 新增检测
    let has_telegram = !ai_config.telegram_bot_token.is_empty()
        && ai_config.telegram_bot_token != "YOUR_TELEGRAM_BOT_TOKEN";

    println!("📊 配置检测:");
    println!("   Gemini API: {}", if has_gemini { "✅ 已配置" } else { "❌ 未配置" });
    println!("   Telegram Bot: {}", if has_telegram { "✅ 已配置" } else { "❌ 未配置" });
    println!();

    // 如果缺少配置，引导用户输入
    if !has_gemini || !has_telegram {
        println!("💡 检测到配置缺失，现在开始配置...\n");

        // 配置Gemini API
        if !has_gemini {
            println!("🤖 配置Gemini API");
            println!("{}", "-".repeat(50));
            println!("📖 Gemini API用于分析您的提示词并提供优化建议");
            println!("🔗 获取API密钥: https://aistudio.google.com/api-keys");
            println!();

            print!("请输入Gemini API Key: ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            ai_config.gemini_api_key = input.trim().to_string();

            if ai_config.gemini_api_key.is_empty() {
                println!("⚠️ 未输入API Key，将跳过Gemini配置");
            } else {
                println!("✅ Gemini API Key已保存\n");
            }
        }

        // 配置Telegram Bot
        if !has_telegram {
            println!("📱 配置Telegram Bot");
            println!("{}", "-".repeat(50));
            println!("🤖 用于接收命令和推送AI分析报告");
            println!("🔗 创建Bot: https://t.me/BotFather");
            println!("💡 步骤: 1) 向@BotFather发送 /newbot");
            println!("        2) 按提示设置名称和用户名");
            println!("        3) 复制得到的Token");
            println!();

            print!("请输入Telegram Bot Token: ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            ai_config.telegram_bot_token = input.trim().to_string();

            if ai_config.telegram_bot_token.is_empty() {
                anyhow::bail!("❌ Telegram Bot Token不能为空，无法启动监听器");
            }

            println!("✅ Telegram Bot Token已保存");
            println!();
            println!("📝 重要提示: 请先在Telegram中向机器人发送任意消息（如 /start）");
            println!("   这样系统才能自动获取您的Chat ID\n");
        }

        // 显示LLM选择界面
        println!("🤖 配置AI分析引擎");
        println!("{}", "-".repeat(50));
        println!("选择您希望使用的AI分析引擎和模式：");
        println!();

        let llm_selection = llm_selector::LlmSelector::show_selection_interface()?;

        // 保存LLM选择到配置文件
        llm_selector::LlmSelector::save_selection_to_config(&llm_selection, &args.config.to_string_lossy()).await?;

        // 保存其他配置到文件
        println!("💾 正在保存配置到 config.toml...");
        save_ai_config_to_file(&args.config, &ai_config)?;
        println!("✅ 配置已保存\n");
    }

    println!("📋 Telegram Bot配置:");
    if ai_config.telegram_bot_token.len() > 20 {
        println!("   Token: {}...{}",
            &ai_config.telegram_bot_token[..10],
            &ai_config.telegram_bot_token[ai_config.telegram_bot_token.len()-10..]);
    } else {
        println!("   Token: (已配置)");
    }
    if !ai_config.telegram_chat_id.is_empty() {
        println!("   Chat ID: {}", ai_config.telegram_chat_id);
    }
    println!();

    // 显示当前反馈间隔配置
    println!("📊 当前反馈间隔配置:");
    match feedback_interval::FeedbackInterval::load() {
        Ok(interval) => {
            println!("{}\n", interval.get_config_description());
        }
        Err(e) => {
            println!("   ⚠️ 加载配置失败: {}", e);
            println!("   💡 将使用默认配置（每24小时）\n");
        }
    }

    // 创建Telegram Bot
    let telegram_config = telegram_bot::TelegramConfig {
        bot_token: ai_config.telegram_bot_token,
        chat_id: ai_config.telegram_chat_id,
        ..Default::default()
    };

    let mut bot = telegram_bot::TelegramBot::new(telegram_config)?;

    // 自动初始化连接 - 非阻塞版本
    println!("🤖 正在自动初始化Telegram连接...");
    let telegram_available = match bot.auto_initialize_connection().await {
        Ok(_) => {
            println!("✅ Telegram连接初始化成功");
            true
        }
        Err(e) => {
            println!("⚠️ Telegram初始化失败: {}", e);
            println!("📊 将以历史监控模式运行，Telegram功能暂时不可用");
            false
        }
    };

    // 如果Telegram不可用，直接运行历史监控
    if !telegram_available {
        println!("🔄 切换到纯监控模式...");
        return run_history_monitor_with_status().await;
    }

    // 测试连接
    println!("🔧 测试Bot连接...");
    match bot.test_connection().await {
        Ok(_) => println!("✅ 连接成功\n"),
        Err(e) => {
            println!("❌ 连接失败: {}", e);
            println!("💡 请检查Bot Token是否正确");
            return Ok(());
        }
    }

    // 获取Bot信息
    if let Ok(bot_info) = bot.get_bot_info().await {
        println!("🤖 {}", bot_info);
    }
    println!();

    // 直接启动标准模式 - Gemini智能分析 + Telegram反馈
    println!("🤖 标准模式 - Gemini智能分析");
    println!("{}", "=".repeat(50));
    println!();
    println!("💡 功能说明:");
    println!("   • 监控Claude Code提示词");
    println!("   • Gemini AI深度分析和优化建议");
    println!("   • Telegram实时推送反馈报告");
    println!("   • 支持命令配置反馈间隔:");
    println!("     /based-on-time <小时>   - 时间间隔");
    println!("     /based-on-number <数量> - 数量间隔");
    println!("     /status                 - 查看状态");
    println!("     /help                   - 显示帮助");
    println!();
    println!("🎧 监听器已启动，等待命令...");
    println!("   按 Ctrl+C 停止");
    println!();

    // 发送启动通知
    if let Err(e) = bot.send_startup_notification().await {
        println!("⚠️ 发送启动通知失败: {}", e);
    }

    // 📖 读取历史提示词但不立即分析 - 让用户通过Telegram命令定制分析方式
    println!("📖 检查Claude历史提示词...");
    let history_file = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join(".claude/history.jsonl");

    let gemini_config = gemini_analyzer::GeminiConfig {
        api_key: ai_config.gemini_api_key.clone(),
        fast_model: ai_config.fast_model.clone(),
        max_retries: ai_config.max_retries,
        system_prompt: ai_config.system_prompt.clone(),
        ..Default::default()
    };

    let analyzer = gemini_analyzer::GeminiAnalyzer::new(gemini_config)?;

    // 读取历史提示词并存储计数，但不分析
    let historical_prompts_count = match analyzer.read_claude_history(&history_file).await {
        Ok(prompts) => {
            let count = prompts.len();
            println!("📊 已读取 {} 条历史提示词", count);
            println!("💡 使用以下Telegram命令开始分析:");
            println!("   📋 /report - 生成最近提示词的快速总体报告");
            println!("   📊 /report-per <数量> - 分段分析历史提示词");
            println!("   📈 /view-history - 查看历史提示词统计");
            println!();
            count
        }
        Err(e) => {
            println!("⚠️ 读取历史提示词失败: {}", e);
            0
        }
    };

    // 发送历史统计通知到Telegram
    let history_summary = if historical_prompts_count > 0 {
        format!(
            "📊 <b>系统启动完成</b>\n\n\
            📈 历史提示词统计: {} 条\n\
            📁 数据源: ~/.claude/history.jsonl\n\n\
            💡 <b>可用命令:</b>\n\
            📋 /report - 快速总体分析\n\
            📊 /report-per &lt;数量&gt; - 分段详细分析\n\
            📈 /view-history - 查看统计信息\n\
            ⚙️ /status - 系统状态\n\
            ❓ /help - 帮助信息",
            historical_prompts_count
        )
    } else {
        "📊 <b>系统启动完成</b>\n\n⚠️ 暂未发现历史提示词数据\n💡 开始使用Claude Code后，系统会自动监控新的提示词".to_string()
    };

    if let Err(e) = bot.send_message(&history_summary).await {
        println!("⚠️ 发送历史统计失败: {}", e);
    } else {
        println!("✅ 历史统计已发送到Telegram");
    }

    // 并发运行两个任务:
    // 1. Telegram命令监听
    // 2. Claude历史监控(实时显示200状态)

    let mut bot_clone = bot.clone();
    let command_listener = tokio::spawn(async move {
        if let Err(e) = bot_clone.start_command_listener().await {
            eprintln!("❌ 命令监听器错误: {}", e);
        }
    });

    let history_monitor = tokio::spawn(async move {
        if let Err(e) = run_history_monitor_with_status().await {
            eprintln!("❌ 历史监控错误: {}", e);
        }
    });

    // 等待任意一个任务结束
    tokio::select! {
        _ = command_listener => println!("📱 命令监听器已停止"),
        _ = history_monitor => println!("📊 历史监控已停止"),
    }

    Ok(())
}

/// 历史监控 - 带HTTP状态码风格输出，实时监控Claude历史文件
async fn run_history_monitor_with_status() -> Result<()> {
    use std::collections::HashSet;

    println!("📊 实时监控Claude Code提示词...");
    println!();

    // 监控Claude真实历史文件
    let claude_history_file = std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
        .join(".claude/history.jsonl");

    if !claude_history_file.exists() {
        println!("⚠️ Claude历史文件不存在: {}", claude_history_file.display());
        println!("💡 请先使用Claude Code进行一些交互");
        return Ok(());
    }

    println!("🔍 监控文件: {}", claude_history_file.display());
    println!("⏰ 检查间隔: 3秒");
    println!("🎯 检测语言: 中文提示词");
    println!();

    let mut seen_prompts = HashSet::new();
    let mut last_size = 0u64;

    // 先读取现有内容，避免重复显示历史提示词
    if let Ok(initial_content) = tokio::fs::read_to_string(&claude_history_file).await {
        let lines: Vec<&str> = initial_content.lines().collect();
        for line in lines {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(display) = json.get("display").and_then(|v| v.as_str()) {
                    let display_text = display.trim();
                    if display_text.len() > 10
                        && !display_text.starts_with('/')
                        && display_text.chars().any(|c| matches!(c, '\u{4e00}'..='\u{9fff}'))
                    {
                        seen_prompts.insert(display_text.to_string());
                    }
                }
            }
        }
        if let Ok(metadata) = tokio::fs::metadata(&claude_history_file).await {
            last_size = metadata.len();
        }
    }

    println!("✅ 监控系统已启动，等待新的Claude提示词...");
    println!();

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // 检查文件大小变化
        if let Ok(metadata) = tokio::fs::metadata(&claude_history_file).await {
            let current_size = metadata.len();

            if current_size > last_size {
                // 文件有新内容，读取并解析
                if let Ok(content) = tokio::fs::read_to_string(&claude_history_file).await {
                    let lines: Vec<&str> = content.lines().collect();

                    // 只检查新的行（从文件末尾开始）
                    let mut new_prompts = Vec::new();
                    for line in lines.iter().rev().take(50) {  // 检查最近50行
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(display) = json.get("display").and_then(|v| v.as_str()) {
                                let display_text = display.trim();

                                // 检查是否是新的中文提示词
                                if display_text.len() > 10
                                    && !display_text.starts_with('/')
                                    && !display_text.starts_with(":")
                                    && display_text.chars().any(|c| matches!(c, '\u{4e00}'..='\u{9fff}'))
                                    && !seen_prompts.contains(display_text)
                                {
                                    new_prompts.push(display_text.to_string());
                                    seen_prompts.insert(display_text.to_string());
                                }
                            }
                        }
                    }

                    // 显示新发现的提示词（按时间顺序）
                    for prompt in new_prompts.iter().rev() {
                        let timestamp = chrono::Local::now().format("%H:%M:%S");
                        let preview = if prompt.len() > 60 {
                            format!("{}...", &prompt[..60])
                        } else {
                            prompt.clone()
                        };

                        // HTTP状态码风格输出 - 绿色
                        println!("[\x1b[32m{}\x1b[0m] \x1b[32m200\x1b[0m ✅ 记录成功 | {}", timestamp, preview);

                        // 保存到本地文件
                        if let Err(e) = save_prompt_to_local(prompt).await {
                            let error_timestamp = chrono::Local::now().format("%H:%M:%S");
                            println!("[\x1b[31m{}\x1b[0m] \x1b[31m500\x1b[0m ❌ 保存失败 | {}", error_timestamp, e);
                        }
                    }
                }
                last_size = current_size;
            }
        }
    }
}

/// 保存提示词到本地文件
async fn save_prompt_to_local(prompt: &str) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    let data_dir = std::path::Path::new("./data");
    if !data_dir.exists() {
        tokio::fs::create_dir_all(data_dir).await?;
    }

    let file_path = data_dir.join("prompts.md");
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await?;

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!("\n## {}\n\n{}\n\n---\n", timestamp, prompt);

    file.write_all(entry.as_bytes()).await?;

    Ok(())
}

/// AI自动反馈模式 - 定期分析并推送
async fn run_auto_feedback_mode(args: &Args) -> Result<()> {
    println!("🤖 启动AI自动反馈模式...");

    // 加载配置
    let config = load_config(&args.config)?;
    let ai_config = load_ai_feedback_config(&config)?;

    if !ai_config.enabled {
        anyhow::bail!("❌ AI反馈系统未启用，请修改配置文件或运行 --setup-feedback");
    }

    // 创建服务实例
    let gemini_config = gemini_analyzer::GeminiConfig {
        api_key: ai_config.gemini_api_key,
        fast_model: ai_config.fast_model.clone(),
        max_retries: ai_config.max_retries,
        system_prompt: ai_config.system_prompt.clone(),
        ..Default::default()
    };

    let telegram_config = telegram_bot::TelegramConfig {
        bot_token: ai_config.telegram_bot_token,
        chat_id: ai_config.telegram_chat_id,
        ..Default::default()
    };

    let analyzer = gemini_analyzer::GeminiAnalyzer::new(gemini_config)?;
    let bot = telegram_bot::TelegramBot::new(telegram_config)?;

    // 发送启动通知
    bot.send_startup_notification().await?;

    println!("🎯 AI反馈系统已启动");
    println!("📊 监控文件: {}", ai_config.prompts_file_path);
    println!("⏰ 推送时间: {}", ai_config.daily_report_time);
    println!("📱 Telegram通知已配置");
    println!("🔄 每24小时自动分析一次");
    println!();

    // 立即执行一次分析（测试用途）
    println!("🔬 执行初始分析测试...");
    if let Err(e) = run_daily_analysis(&analyzer, &bot, &ai_config.prompts_file_path, ai_config.max_prompts_per_analysis).await {
        println!("⚠️ 初始分析失败: {}", e);
    }

    // 创建定时任务 - 简化版本，每小时检查一次
    println!("📅 设置定时任务: 每小时检查一次");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1小时

    println!("⏹️ 按 Ctrl+C 停止AI反馈服务");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let now = chrono::Local::now();
                let current_time = now.format("%H:%M").to_string();

                // 检查是否到了指定时间
                if current_time == ai_config.daily_report_time {
                    println!("⏰ 开始定时AI分析任务...");

                    match run_daily_analysis(&analyzer, &bot, &ai_config.prompts_file_path, ai_config.max_prompts_per_analysis).await {
                        Ok(_) => println!("✅ 定时分析任务完成"),
                        Err(e) => {
                            eprintln!("❌ 定时分析任务失败: {}", e);
                            let _ = bot.send_error_notification(&e.to_string()).await;
                        }
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n⏹️ AI反馈服务已停止");
                break;
            }
        }
    }

    Ok(())
}

/// 执行每日分析任务
async fn run_daily_analysis(
    analyzer: &gemini_analyzer::GeminiAnalyzer,
    bot: &telegram_bot::TelegramBot,
    file_path: &str,
    max_prompts: usize,
) -> Result<()> {
    use std::path::Path;

    let path = Path::new(file_path);
    if !path.exists() {
        println!("⚠️ 提示词文件不存在: {}", file_path);
        return Ok(());
    }

    // 分析提示词文件
    let mut analyses = analyzer.analyze_prompts_file(path).await?;

    // 限制分析数量
    if analyses.len() > max_prompts {
        analyses.truncate(max_prompts);
        println!("⚠️ 提示词数量超限，仅分析前{}条", max_prompts);
    }

    if analyses.is_empty() {
        let message = "📝 今日没有新的提示词需要分析";
        bot.send_analysis_report(message).await?;
        return Ok(());
    }

    // 生成报告
    let report = analyzer.generate_report(&analyses);

    // 发送到Telegram
    bot.send_analysis_report(&report).await?;

    println!("📊 今日分析完成: {}条提示词", analyses.len());
    Ok(())
}

/// 加载AI反馈配置
fn load_ai_feedback_config(_config: &Config) -> Result<AiFeedbackConfig> {
    // 尝试从配置文件中读取AI反馈配置
    let config_content = std::fs::read_to_string("config.toml")
        .unwrap_or_else(|_| "".to_string());

    if !config_content.contains("[ai_feedback]") {
        anyhow::bail!("❌ 未找到AI反馈配置，请运行: --setup-feedback");
    }

    // 解析TOML配置（简单解析）
    let mut ai_config = AiFeedbackConfig::default();

    for line in config_content.lines() {
        let line = line.trim();
        if let Some(key_value) = parse_config_line(line) {
            match key_value.0.as_str() {
                "enabled" => ai_config.enabled = key_value.1 == "true",
                // LLM配置
                "llm_provider" => ai_config.llm_provider = key_value.1,
                "llm_mode" => ai_config.llm_mode = key_value.1,
                // Gemini配置
                "gemini_api_key" => ai_config.gemini_api_key = key_value.1,
                "fast_model" => ai_config.fast_model = key_value.1,
                // Perplexity配置
                "perplexity_api_key" => ai_config.perplexity_api_key = key_value.1,
                "perplexity_model" => ai_config.perplexity_model = key_value.1,
                "perplexity_api_url" => ai_config.perplexity_api_url = key_value.1,
                // Telegram配置
                "telegram_bot_token" => ai_config.telegram_bot_token = key_value.1,
                "telegram_chat_id" => ai_config.telegram_chat_id = key_value.1,
                "daily_report_time" => ai_config.daily_report_time = key_value.1,
                "max_prompts_per_analysis" => {
                    if let Ok(num) = key_value.1.parse::<usize>() {
                        ai_config.max_prompts_per_analysis = num;
                    }
                }
                "max_retries" => {
                    if let Ok(num) = key_value.1.parse::<usize>() {
                        ai_config.max_retries = num;
                    }
                }
                "system_prompt" => {
                    // 处理系统提示词
                    ai_config.system_prompt = key_value.1;
                }
                _ => {}
            }
        }
    }

    // 如果配置中没有system_prompt，使用默认值
    if ai_config.system_prompt.is_empty() {
        ai_config.system_prompt = AiFeedbackConfig::default_system_prompt();
    }

    // 不再强制验证配置，让调用方决定是否需要交互式配置
    Ok(ai_config)
}

/// 保存AI反馈配置到文件
fn save_ai_config_to_file(config_path: &PathBuf, ai_config: &AiFeedbackConfig) -> Result<()> {
    // 读取现有配置
    let config_content = std::fs::read_to_string(config_path)
        .unwrap_or_else(|_| include_str!("../config.toml").to_string());

    // 更新配置内容
    let mut updated_lines = Vec::new();
    let mut in_ai_feedback_section = false;

    for line in config_content.lines() {
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
                updated_lines.push(format!("enabled = {}                    # 系统启用状态", ai_config.enabled));
            } else if trimmed_line.starts_with("gemini_api_key") {
                updated_lines.push(format!("gemini_api_key = \"{}\"                # Gemini API密钥", ai_config.gemini_api_key));
            } else if trimmed_line.starts_with("telegram_bot_token") {
                updated_lines.push(format!("telegram_bot_token = \"{}\"            # Telegram Bot Token", ai_config.telegram_bot_token));
            } else if trimmed_line.starts_with("telegram_chat_id") {
                updated_lines.push(format!("telegram_chat_id = \"{}\"              # Chat ID", ai_config.telegram_chat_id));
            } else {
                updated_lines.push(line.to_string());
            }
        } else {
            updated_lines.push(line.to_string());
        }
    }

    // 写入文件
    std::fs::write(config_path, updated_lines.join("\n"))?;

    Ok(())
}

/// 解析配置文件行
fn parse_config_line(line: &str) -> Option<(String, String)> {
    if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
        return None;
    }

    if let Some(eq_pos) = line.find('=') {
        let key = line[..eq_pos].trim();
        let mut value = line[eq_pos + 1..].trim();

        // 先处理行尾注释
        if let Some(comment_pos) = value.find('#') {
            value = value[..comment_pos].trim();
        }

        // 再移除引号
        let clean_value = if value.starts_with('"') && value.ends_with('"') && value.len() > 1 {
            value[1..value.len()-1].to_string()
        } else {
            value.to_string()
        };

        return Some((key.to_string(), clean_value));
    }

    None
}

/// 显示AI反馈系统帮助
fn show_ai_feedback_help() {
    println!("\n{}", "=".repeat(70));
    println!("🤖 AI反馈系统使用指南");
    println!("{}", "=".repeat(70));

    println!("\n🎯 功能概述:");
    println!("   • 自动分析Claude提示词，提供专业改进建议");
    println!("   • 支持Google Gemini和Perplexity AI双引擎");
    println!("   • 可配置快速模式和高级分析模式");
    println!("   • 每日通过Telegram推送分析报告");

    println!("\n🔧 配置命令:");
    println!("   ./target/release/prompter --setup-feedback");
    println!("   # 启动交互式配置向导，自动打开浏览器获取API密钥");

    println!("\n🧪 测试命令:");
    println!("   ./target/release/prompter --test-feedback");
    println!("   # 测试LLM API和Telegram Bot连接");

    println!("\n🚀 启动服务:");
    println!("   ./target/release/prompter --auto-feedback");
    println!("   # 启动每日AI分析服务");

    println!("\n📖 帮助信息:");
    println!("   ./target/release/prompter --help-feedback");
    println!("   # 显示这个帮助页面");

    println!("\n🔧 LLM配置 (config.toml):");
    println!("   [ai_feedback]");
    println!("   llm_provider = \"gemini\"              # LLM提供商: gemini 或 perplexity");
    println!("   llm_mode = \"fast\"                   # 模式: fast 或 thinking");
    println!("   gemini_api_key = \"your-api-key\"     # Gemini API密钥");
    println!("   perplexity_api_key = \"your-api-key\" # Perplexity API密钥");
    println!("   max_retries = 3                     # 最大重试次数");

    println!("\n📱 Telegram配置:");
    println!("   1. 向 @BotFather 发送 /newbot");
    println!("   2. 设置机器人名称");
    println!("   3. 获取Bot Token");
    println!("   4. 向新机器人发送任意消息");
    println!("   5. 系统会自动获取Chat ID");

    println!("\n🔑 API密钥获取:");
    println!("   Gemini: https://aistudio.google.com/api-keys");
    println!("   Telegram: https://web.telegram.org/k/#@BotFather");

    println!("\n💡 最佳实践:");
    println!("   • 使用动态思考 (thinking_budget = -1) 获得最佳效果");
    println!("   • 启用自动降级确保服务可用性");
    println!("   • 定期检查配额使用情况");
    println!("   • 配合历史监控模式获得最佳数据收集效果");

    println!("\n{}", "=".repeat(70));
    println!("🚀 准备好了吗？运行 --setup-feedback 开始配置！");
    println!("{}", "=".repeat(70));
}

/// 确认用户操作
fn confirm_action(prompt: &str) -> Result<()> {
    print!("{} (y/N): ", prompt);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" | "是" => Ok(()),
        _ => anyhow::bail!("用户取消操作"),
    }
}

/// AI反馈配置结构
#[derive(Debug, Clone)]
struct AiFeedbackConfig {
    enabled: bool,
    // LLM配置
    llm_provider: String,
    llm_mode: String,
    // Gemini配置
    gemini_api_key: String,
    fast_model: String,
    // Perplexity配置
    perplexity_api_key: String,
    perplexity_model: String,
    perplexity_api_url: String,
    // Telegram配置
    telegram_bot_token: String,
    telegram_chat_id: String,
    daily_report_time: String,
    max_prompts_per_analysis: usize,
    prompts_file_path: String,
    max_retries: usize,
    // 系统提示词 - 用于指导AI如何分析用户提示词
    system_prompt: String,
}

impl Default for AiFeedbackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            // LLM配置
            llm_provider: "gemini".to_string(),
            llm_mode: "fast".to_string(),
            // Gemini配置
            gemini_api_key: String::new(),
            fast_model: "gemini-2.5-flash".to_string(),
            // Perplexity配置
            perplexity_api_key: String::new(),
            perplexity_model: "sonar-reasoning".to_string(),
            perplexity_api_url: "https://api.perplexity.ai".to_string(),
            // Telegram配置
            telegram_bot_token: String::new(),
            telegram_chat_id: String::new(),
            daily_report_time: "09:00".to_string(),
            max_prompts_per_analysis: 50,
            prompts_file_path: "./data/claude_history_prompts.md".to_string(),
            max_retries: 3,
            system_prompt: Self::default_system_prompt(),
        }
    }
}

impl AiFeedbackConfig {
    /// 默认的系统提示词 - LLM提示词审阅分析员
    fn default_system_prompt() -> String {
        r#"你是一名专业的 LLM 提示词审阅分析员。请基于今天抓取到的所有"我与 Claude Code 的提示词交互记录"，输出一份可在 1 分钟内阅读完毕的日报式总结。按照以下要求生成内容：

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
* …"#.to_string()
    }
}

/// 全局进程监控模式 - 监控所有终端中的Claude相关进程
async fn run_global_process_monitor(args: &Args) -> Result<()> {
    println!("🌍 启动全局进程监控模式...");

    // 加载配置
    let config = load_config(&args.config)?;

    // 创建中文过滤器
    let chinese_filter = chinese_filter::ChineseFilter::new(
        config.filter.min_chinese_chars,
        &config.filter.exclude_commands,
    )?;

    // 创建全局进程监控器
    let mut global_monitor = global_process_monitor::GlobalProcessMonitor::new(chinese_filter)?;

    println!("🌐 全局进程监控已启动，正在监控所有终端的Claude进程:");
    println!("  - 所有Claude相关进程检测");
    println!("  - 跨终端命令行参数捕获");
    println!("  - 环境变量中文内容监控");
    #[cfg(target_os = "macos")]
    println!("  - macOS剪贴板监控");
    println!("📁 中文输入将保存到: ./data/global_claude_prompts.md");
    println!("⏰ 扫描间隔: 3秒");
    println!();
    println!("🔧 如需结合自动上传功能，请使用: --auto --global-monitor");
    println!("⏹️ 按 Ctrl+C 停止监控");
    println!();

    // 启动全局监控
    global_monitor.start_global_monitoring().await?;

    // 等待中断信号
    tokio::signal::ctrl_c().await?;
    println!("⏹️ 全局进程监控服务已停止");

    Ok(())
}