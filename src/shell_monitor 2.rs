// Shell历史文件监控模块 - 独立终端监控方案
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use chrono::{DateTime, Local};
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
        })
    }

    /// 启动独立监控服务
    pub async fn start_independent_monitoring(&mut self) -> anyhow::Result<()> {
        println!("🚀 启动独立Shell监控服务...");
        println!("📂 监控的历史文件:");

        // 检查可用的历史文件
        let mut available_files = Vec::new();
        for path in &self.config.shell_history_paths {
            if path.exists() {
                println!("  ✅ {}", path.display());
                available_files.push(path.clone());
            } else {
                println!("  ❌ {} (不存在)", path.display());
            }
        }

        if available_files.is_empty() {
            anyhow::bail!("没有找到可监控的shell历史文件");
        }

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
        let mut watcher = watcher(tx, Duration::from_secs(1))?;

        // 监控所有历史文件
        for file_path in files {
            if let Some(parent) = file_path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
        }

        println!("✅ 文件监控已启动，等待变化...");

        // 主监控循环
        tokio::task::spawn_blocking(move || {
            loop {
                match rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(event) => {
                        match event {
                            DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                                // 检查是否是我们监控的历史文件
                                if files.iter().any(|f| f == &path) {
                                    println!("📝 检测到历史文件更新: {}", path.display());
                                    // 这里需要通过某种方式通知主线程处理新内容
                                    // 由于异步限制，这里简化处理
                                }
                            }
                            _ => {}
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
    async fn save_claude_command(&self, timestamp: DateTime<Local>, content: &str) -> anyhow::Result<()> {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir)?;

        let prompts_file = data_dir.join("shell_captured_prompts.md");
        let formatted_content = format!(
            "## {} (Shell监控)\n\n```\n{}\n```\n\n",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            content
        );

        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&prompts_file)?
            .write_all(formatted_content.as_bytes())?;

        Ok(())
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
}

use std::io::Write;