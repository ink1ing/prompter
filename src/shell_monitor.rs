// Shell历史文件监控模块 - 独立终端监控方案
use notify::{Watcher, RecursiveMode, recommended_watcher, Event, EventKind};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use std::process::Command;
use chrono::{DateTime, Local, TimeZone};
use regex::Regex;
use crate::chinese_filter::ChineseFilter;

#[derive(Debug, Clone)]
pub struct ShellMonitorConfig {
    pub shell_history_paths: Vec<PathBuf>,
    pub claude_patterns: Vec<String>,
    pub check_interval_seconds: u64,
    pub enable_realtime_watch: bool,
}

impl Default for ShellMonitorConfig {
    fn default() -> Self {
        Self {
            shell_history_paths: Self::get_default_history_paths(),
            claude_patterns: vec![
                r"claude\s+".to_string(),
                r"claude-code\s+".to_string(),
                r"claude-cli\s+".to_string(),
            ],
            check_interval_seconds: 5,
            enable_realtime_watch: true,
        }
    }
}

impl ShellMonitorConfig {
    /// 获取默认的shell历史文件路径
    fn get_default_history_paths() -> Vec<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        vec![
            PathBuf::from(format!("{}/.bash_history", home)),
            PathBuf::from(format!("{}/.zsh_history", home)),
            PathBuf::from(format!("{}/.history", home)),
            PathBuf::from(format!("{}/.local/share/fish/fish_history", home)),
        ]
    }
}

pub struct ShellMonitor {
    config: ShellMonitorConfig,
    chinese_filter: ChineseFilter,
    processed_commands: HashSet<String>,
    claude_regex: Regex,
    last_check_time: DateTime<Local>,
    active_terminals: HashSet<String>,
    successful_records: u64,
}

impl ShellMonitor {
    pub fn new(config: ShellMonitorConfig, chinese_filter: ChineseFilter) -> anyhow::Result<Self> {
        // 编译Claude命令匹配正则表达式
        let pattern = format!("({})", config.claude_patterns.join("|"));
        let claude_regex = Regex::new(&pattern)?;

        Ok(Self {
            config,
            chinese_filter,
            processed_commands: HashSet::new(),
            claude_regex,
            last_check_time: Local::now(),
            active_terminals: HashSet::new(),
            successful_records: 0,
        })
    }

    /// 启动独立监控服务
    pub async fn start_independent_monitoring(&mut self) -> anyhow::Result<()> {
        self.print_startup_banner();

        // 检查可用的历史文件
        let mut available_files = Vec::new();
        for path in &self.config.shell_history_paths {
            if path.exists() {
                available_files.push(path.clone());
            }
        }

        if available_files.is_empty() {
            anyhow::bail!("没有找到可监控的shell历史文件");
        }

        // 检测活跃终端
        self.detect_active_terminals().await?;

        // 显示监控状态
        self.print_monitoring_status(&available_files);

        // 初始扫描现有历史
        self.scan_existing_history(&available_files).await?;

        if self.config.enable_realtime_watch {
            // 启动实时文件监控
            self.start_file_watcher(&available_files).await?;
        } else {
            // 使用定时扫描模式
            self.start_polling_mode(&available_files).await?;
        }

        Ok(())
    }

    /// 扫描现有历史记录
    async fn scan_existing_history(&mut self, files: &[PathBuf]) -> anyhow::Result<()> {
        println!("🔍 扫描现有历史记录...");

        for file_path in files {
            match self.process_history_file(file_path, true).await {
                Ok(count) => {
                    if count > 0 {
                        println!("  📄 {}: 找到{}条Claude命令", file_path.display(), count);
                    }
                }
                Err(e) => {
                    println!("  ⚠️  处理{}失败: {}", file_path.display(), e);
                }
            }
        }

        self.last_check_time = Local::now();
        Ok(())
    }

    /// 启动文件监控模式
    async fn start_file_watcher(&mut self, files: &[PathBuf]) -> anyhow::Result<()> {
        println!("👁️  启动实时文件监控模式...");

        let (tx, rx) = channel();
        let mut watcher = recommended_watcher(tx)?;

        // 监控所有历史文件
        for file_path in files {
            if let Some(parent) = file_path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
        }

        println!("✅ 文件监控已启动，等待变化...");

        // 克隆文件路径以避免生命周期问题
        let files_owned = files.to_vec();

        // 主监控循环
        tokio::task::spawn_blocking(move || {
            loop {
                match rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(event_result) => {
                        match event_result {
                            Ok(event) => {
                                match event.kind {
                                    EventKind::Modify(_) | EventKind::Create(_) => {
                                        for path in &event.paths {
                                            if files_owned.iter().any(|f| f == path) {
                                                println!("📝 检测到历史文件更新: {}", path.display());
                                                // 这里需要通过某种方式通知主线程处理新内容
                                                // 由于异步限制，这里简化处理
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                println!("文件监控错误: {}", e);
                            }
                        }
                    }
                    Err(_) => {
                        // 超时，继续循环
                    }
                }
            }
        });

        // 为了保持兼容性，我们还是使用定时检查作为备用
        self.start_polling_mode(files).await
    }

    /// 启动定时扫描模式
    async fn start_polling_mode(&mut self, files: &[PathBuf]) -> anyhow::Result<()> {
        println!("⏰ 启动定时扫描模式 (间隔: {}秒)", self.config.check_interval_seconds);

        let mut interval = tokio::time::interval(
            Duration::from_secs(self.config.check_interval_seconds)
        );

        loop {
            interval.tick().await;

            let mut total_new_commands = 0;
            for file_path in files {
                match self.process_history_file(file_path, false).await {
                    Ok(count) => {
                        total_new_commands += count;
                    }
                    Err(e) => {
                        println!("⚠️  处理{}失败: {}", file_path.display(), e);
                    }
                }
            }

            if total_new_commands > 0 {
                println!("📊 本轮检查发现{}条新的Claude命令", total_new_commands);
            }

            self.last_check_time = Local::now();
        }
    }

    /// 处理单个历史文件
    async fn process_history_file(&mut self, file_path: &Path, is_initial_scan: bool) -> anyhow::Result<usize> {
        let content = fs::read_to_string(file_path)?;
        let mut new_commands = 0;

        // 根据shell类型解析历史格式
        let commands = if file_path.to_string_lossy().contains("zsh_history") {
            self.parse_zsh_history(&content)?
        } else if file_path.to_string_lossy().contains("fish_history") {
            self.parse_fish_history(&content)?
        } else {
            self.parse_bash_history(&content)?
        };

        for (timestamp, command) in commands {
            // 检查是否是Claude相关命令
            if self.claude_regex.is_match(&command) {
                let command_hash = format!("{}:{}", timestamp.timestamp(), command);

                // 避免重复处理
                if !self.processed_commands.contains(&command_hash) {
                    self.processed_commands.insert(command_hash);

                    // 只处理指定时间之后的命令（非初始扫描时）
                    if is_initial_scan || timestamp > self.last_check_time {
                        if let Some(chinese_content) = self.chinese_filter.filter_prompt(&command) {
                            self.save_claude_command(timestamp, &chinese_content).await?;
                            new_commands += 1;

                            if !is_initial_scan {
                                println!("🎯 捕获到中文Claude命令: {}",
                                    chinese_content.chars().take(50).collect::<String>());
                            }
                        }
                    }
                }
            }
        }

        Ok(new_commands)
    }

    /// 解析bash历史格式
    fn parse_bash_history(&self, content: &str) -> anyhow::Result<Vec<(DateTime<Local>, String)>> {
        let mut commands = Vec::new();
        let mut current_time = Local::now();

        for line in content.lines() {
            if line.starts_with('#') {
                // Bash时间戳格式: #1234567890
                if let Ok(timestamp) = line[1..].parse::<i64>() {
                    current_time = DateTime::from_timestamp(timestamp, 0)
                        .map(|dt| dt.with_timezone(&Local))
                        .unwrap_or_else(|| Local::now());
                }
            } else if !line.trim().is_empty() {
                commands.push((current_time, line.to_string()));
            }
        }

        Ok(commands)
    }

    /// 解析zsh历史格式
    fn parse_zsh_history(&self, content: &str) -> anyhow::Result<Vec<(DateTime<Local>, String)>> {
        let mut commands = Vec::new();
        let zsh_regex = Regex::new(r"^: (\d+):\d+;(.*)$")?;

        for line in content.lines() {
            if let Some(captures) = zsh_regex.captures(line) {
                if let (Some(timestamp_str), Some(command)) = (captures.get(1), captures.get(2)) {
                    if let Ok(timestamp) = timestamp_str.as_str().parse::<i64>() {
                        let datetime = DateTime::from_timestamp(timestamp, 0)
                            .map(|dt| dt.with_timezone(&Local))
                            .unwrap_or_else(|| Local::now());
                        commands.push((datetime, command.as_str().to_string()));
                    }
                }
            }
        }

        Ok(commands)
    }

    /// 解析fish历史格式
    fn parse_fish_history(&self, content: &str) -> anyhow::Result<Vec<(DateTime<Local>, String)>> {
        let mut commands = Vec::new();
        let mut current_command = String::new();
        let mut current_time = Local::now();

        for line in content.lines() {
            if line.starts_with("- cmd: ") {
                if !current_command.is_empty() {
                    commands.push((current_time, current_command.clone()));
                }
                current_command = line[7..].to_string();
            } else if line.starts_with("  when: ") {
                if let Ok(timestamp) = line[8..].parse::<i64>() {
                    current_time = DateTime::from_timestamp(timestamp, 0)
                        .map(|dt| dt.with_timezone(&Local))
                        .unwrap_or_else(|| Local::now());
                }
            }
        }

        if !current_command.is_empty() {
            commands.push((current_time, current_command));
        }

        Ok(commands)
    }

    /// 保存Claude命令到本地文件
    async fn save_claude_command(&mut self, timestamp: DateTime<Local>, content: &str) -> anyhow::Result<()> {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir)?;

        let prompts_file = data_dir.join("shell_captured_prompts.md");
        let formatted_content = format!(
            "## {} (Shell监控)\n\n```\n{}\n```\n\n",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            content
        );

        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&prompts_file)?
            .write_all(formatted_content.as_bytes()) {
            Ok(_) => {
                // 成功保存，显示200状态码
                let preview = if content.len() > 30 {
                    format!("{}...", &content[..30])
                } else {
                    content.to_string()
                };
                self.print_status_response(true, "中文提示词已保存", Some(&preview));
                Ok(())
            }
            Err(e) => {
                // 保存失败，显示错误状态码
                self.print_status_response(false, "文件写入失败", Some(&e.to_string()));
                Err(e.into())
            }
        }
    }

    /// 获取监控统计信息
    pub async fn get_stats(&self) -> String {
        format!(
            "📊 Shell监控统计\n\
            ==================\n\
            已处理命令数: {}\n\
            上次检查: {}\n\
            监控文件数: {}\n\
            检查间隔: {}秒\n",
            self.processed_commands.len(),
            self.last_check_time.format("%Y-%m-%d %H:%M:%S"),
            self.config.shell_history_paths.len(),
            self.config.check_interval_seconds
        )
    }

    /// 打印启动横幅
    fn print_startup_banner(&self) {
        println!("\n{}", "=".repeat(60));
        println!("🤖 PROMPTER SHELL MONITOR v1.0");
        println!("   独立终端监控模式 - Claude提示词智能收集");
        println!("{}", "=".repeat(60));
        println!("⏰ 启动时间: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("🔧 监控间隔: {}秒", self.config.check_interval_seconds);
        println!("{}", "-".repeat(60));
    }

    /// 检测活跃终端
    async fn detect_active_terminals(&mut self) -> anyhow::Result<()> {
        println!("🔍 正在检测活跃终端...");

        // macOS/Linux: 使用ps命令检测终端进程
        let output = Command::new("ps")
            .args(&["aux"])
            .output()?;

        let ps_output = String::from_utf8_lossy(&output.stdout);
        let mut terminals = HashSet::new();

        // 检测常见终端应用
        let terminal_patterns = vec![
            "Terminal.app", "iTerm.app", "Alacritty", "kitty",
            "gnome-terminal", "konsole", "xterm", "tmux", "screen"
        ];

        for line in ps_output.lines() {
            for pattern in &terminal_patterns {
                if line.contains(pattern) && !line.contains("grep") {
                    terminals.insert(pattern.to_string());
                }
            }
        }

        // 检测TTY会话
        if let Ok(tty_output) = Command::new("who").output() {
            let tty_sessions = String::from_utf8_lossy(&tty_output.stdout);
            let tty_count = tty_sessions.lines().count();

            if tty_count > 0 {
                terminals.insert(format!("TTY会话({} sessions)", tty_count));
            }
        }

        self.active_terminals = terminals;
        Ok(())
    }

    /// 显示监控状态
    fn print_monitoring_status(&self, available_files: &[PathBuf]) {
        println!("📊 监控状态总览");
        println!("{}", "-".repeat(60));

        // 显示活跃终端
        println!("🖥️  活跃终端 ({} 个):", self.active_terminals.len());
        if self.active_terminals.is_empty() {
            println!("   ⚠️  未检测到明确的终端进程");
        } else {
            for terminal in &self.active_terminals {
                println!("   ✅ {}", terminal);
            }
        }

        println!();

        // 显示监控文件
        println!("📂 监控文件 ({} 个):", available_files.len());
        for file in available_files {
            let file_size = fs::metadata(file)
                .map(|m| m.len())
                .unwrap_or(0);
            println!("   ✅ {} ({} bytes)", file.display(), file_size);
        }

        println!();
        println!("🎯 目标命令模式:");
        for pattern in &self.config.claude_patterns {
            println!("   📝 {}", pattern.trim_end_matches("\\s+"));
        }

        println!("{}", "-".repeat(60));
        println!("🚀 监控服务已就绪，等待Claude命令...");
        println!();
    }

    /// 打印HTTP状态码风格的响应
    fn print_status_response(&mut self, success: bool, message: &str, details: Option<&str>) {
        let timestamp = Local::now().format("%H:%M:%S%.3f");

        if success {
            self.successful_records += 1;
            println!("📥 [{}] 200 OK - {} | Records: {}",
                timestamp, message, self.successful_records);
            if let Some(detail) = details {
                println!("   └─ 📄 Content: {}", detail);
            }
        } else {
            println!("❌ [{}] 400 ERROR - {}", timestamp, message);
            if let Some(detail) = details {
                println!("   └─ 🔍 Details: {}", detail);
            }
        }
    }

    /// 显示实时监控界面
    fn print_monitoring_dashboard(&self) {
        // 清屏并显示实时状态（可选）
        print!("\x1B[2J\x1B[H"); // ANSI清屏命令

        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║                    🤖 PROMPTER MONITOR                   ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ 活跃终端: {:2} │ 成功记录: {:4} │ 监控文件: {:2}        ║",
            self.active_terminals.len(),
            self.successful_records,
            self.config.shell_history_paths.len()
        );
        println!("║ 最后检查: {}                       ║",
            self.last_check_time.format("%H:%M:%S")
        );
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
    }
}

use std::io::Write;