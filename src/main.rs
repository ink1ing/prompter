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