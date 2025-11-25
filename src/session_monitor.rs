// Claude Code会话监控模块 - 监控活跃的Claude Code进程和会话
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time;
use regex::Regex;
use anyhow::Result;
use crate::chinese_filter::ChineseFilter;

#[derive(Debug, Clone)]
pub struct ClaudeSession {
    pub pid: u32,
    pub command: String,
    pub start_time: SystemTime,
    pub working_dir: PathBuf,
    pub tmux_pane: Option<String>,
}

pub struct SessionMonitor {
    chinese_filter: ChineseFilter,
    active_sessions: HashMap<u32, ClaudeSession>,
    session_regex: Regex,
    last_scan_time: SystemTime,
}

impl SessionMonitor {
    pub fn new(chinese_filter: ChineseFilter) -> Result<Self> {
        // 匹配Claude Code相关进程的正则表达式
        let session_regex = Regex::new(r"(claude|claude-code|claude-cli)")?;

        Ok(Self {
            chinese_filter,
            active_sessions: HashMap::new(),
            session_regex,
            last_scan_time: SystemTime::now(),
        })
    }

    /// 启动会话监控
    pub async fn start_session_monitoring(&mut self) -> Result<()> {
        self.print_session_monitor_banner();

        // 初始扫描活跃会话
        self.discover_active_sessions().await?;

        // 监控Claude Code配置目录的变化
        self.monitor_claude_config_files().await?;

        // 主监控循环
        let mut interval = time::interval(Duration::from_secs(3));

        loop {
            interval.tick().await;

            // 更新活跃会话列表
            self.discover_active_sessions().await?;

            // 监控会话输入输出
            self.monitor_session_interactions().await?;

            // 检查Claude Code配置和历史文件
            self.check_claude_session_files().await?;

            // 监控Claude Code的实际日志和缓存文件
            self.monitor_claude_system_files().await?;
        }
    }

    /// 发现活跃的Claude Code会话
    async fn discover_active_sessions(&mut self) -> Result<()> {
        // 使用ps命令查找Claude Code进程
        let output = Command::new("ps")
            .args(&["aux"])
            .output()?;

        let ps_output = String::from_utf8_lossy(&output.stdout);
        let mut new_sessions = HashMap::new();
        let mut found_count = 0;

        for line in ps_output.lines() {
            if self.session_regex.is_match(line) && !line.contains("grep") {
                found_count += 1;
                println!("🔍 [DEBUG] 发现Claude进程 #{}: {}", found_count, line);

                if let Some(session) = self.parse_ps_line(line).await? {
                    // 检查是否是新会话
                    if !self.active_sessions.contains_key(&session.pid) {
                        println!("🎯 发现新的Claude Code会话: PID {} - {}",
                            session.pid, session.command);

                        // 尝试检测tmux pane
                        if let Some(tmux_pane) = self.detect_tmux_pane(session.pid).await? {
                            println!("  📺 tmux pane: {}", tmux_pane);
                        }
                    }
                    new_sessions.insert(session.pid, session);
                }
            }
        }

        // 显示扫描结果统计
        println!("📊 [DEBUG] 进程扫描结果: 找到 {} 个Claude进程，活跃会话 {} 个",
            found_count, new_sessions.len());

        // 检测结束的会话
        let ended_sessions: Vec<u32> = self.active_sessions.keys()
            .filter(|pid| !new_sessions.contains_key(pid))
            .copied()
            .collect();

        for pid in ended_sessions {
            if let Some(session) = self.active_sessions.remove(&pid) {
                println!("👋 Claude Code会话结束: PID {} - {}", pid, session.command);
            }
        }

        self.active_sessions = new_sessions;
        Ok(())
    }

    /// 解析ps命令行输出
    async fn parse_ps_line(&self, line: &str) -> Result<Option<ClaudeSession>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 11 {
            return Ok(None);
        }

        // 尝试解析PID
        if let Ok(pid) = parts[1].parse::<u32>() {
            let command = parts[10..].join(" ");

            // 获取进程工作目录
            let working_dir = self.get_process_working_dir(pid).await
                .unwrap_or_else(|_| PathBuf::from("/tmp"));

            let session = ClaudeSession {
                pid,
                command: command.clone(),
                start_time: SystemTime::now(),
                working_dir,
                tmux_pane: None,
            };

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// 获取进程工作目录
    async fn get_process_working_dir(&self, pid: u32) -> Result<PathBuf> {
        let cwd_path = format!("/proc/{}/cwd", pid);
        if Path::new(&cwd_path).exists() {
            Ok(fs::read_link(cwd_path)?)
        } else {
            // macOS使用lsof获取工作目录
            let output = Command::new("lsof")
                .args(&["-p", &pid.to_string(), "-d", "cwd"])
                .output()?;

            let lsof_output = String::from_utf8_lossy(&output.stdout);
            for line in lsof_output.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 8 {
                    return Ok(PathBuf::from(parts[8]));
                }
            }

            Ok(PathBuf::from("/tmp"))
        }
    }

    /// 检测tmux pane
    async fn detect_tmux_pane(&self, pid: u32) -> Result<Option<String>> {
        // 检查环境变量TMUX
        let output = Command::new("ps")
            .args(&["eww", &pid.to_string()])
            .output()?;

        let env_output = String::from_utf8_lossy(&output.stdout);
        if env_output.contains("TMUX=") {
            // 尝试获取tmux pane信息
            let tmux_output = Command::new("tmux")
                .args(&["list-panes", "-F", "#{pane_pid} #{pane_id}"])
                .output();

            if let Ok(output) = tmux_output {
                let panes = String::from_utf8_lossy(&output.stdout);
                for line in panes.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(pane_pid) = parts[0].parse::<u32>() {
                            if pane_pid == pid {
                                return Ok(Some(parts[1].to_string()));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// 监控会话交互
    async fn monitor_session_interactions(&mut self) -> Result<()> {
        // 收集需要处理的会话信息，避免借用冲突
        let sessions_to_process: Vec<(u32, Option<String>, PathBuf)> = self.active_sessions
            .iter()
            .map(|(pid, session)| (*pid, session.tmux_pane.clone(), session.working_dir.clone()))
            .collect();

        for (pid, tmux_pane, working_dir) in sessions_to_process {
            // 尝试捕获tmux pane输出
            if let Some(pane_id) = tmux_pane {
                self.capture_tmux_pane_content(&pane_id, pid).await?;
            }

            // 监控进程的文件描述符
            self.monitor_process_files_for_pid(pid, &working_dir).await?;
        }
        Ok(())
    }

    /// 捕获tmux pane内容
    async fn capture_tmux_pane_content(&mut self, pane_id: &str, pid: u32) -> Result<()> {
        let output = Command::new("tmux")
            .args(&["capture-pane", "-t", pane_id, "-p"])
            .output()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            self.analyze_captured_content(&content, pid).await?;
        }

        Ok(())
    }

    /// 监控进程文件
    async fn monitor_process_files_for_pid(&mut self, pid: u32, working_dir: &PathBuf) -> Result<()> {
        // 检查Claude Code的临时文件和会话文件
        let possible_paths = vec![
            format!("/tmp/claude-{}", pid),
            format!("/tmp/claude-session-{}", pid),
            working_dir.join(".claude").join("sessions").to_string_lossy().to_string(),
            working_dir.join(".claude").join("history.txt").to_string_lossy().to_string(),
        ];

        for path in possible_paths {
            if Path::new(&path).exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    self.analyze_captured_content(&content, pid).await?;
                }
            }
        }

        Ok(())
    }

    /// 分析捕获的内容
    async fn analyze_captured_content(&mut self, content: &str, pid: u32) -> Result<()> {
        // 寻找用户输入的中文提示词
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            // 增加调试输出
            if self.chinese_filter.contains_chinese(line) {
                println!("🔍 [DEBUG] 发现中文行 #{}: {}", i, line.chars().take(50).collect::<String>());

                // 检测可能的用户输入行
                if self.is_user_input_line(line) {
                    if let Some(chinese_content) = self.chinese_filter.filter_prompt(line) {
                        // 检查是否已经处理过这个内容（简化处理）
                        let _content_hash = format!("{}:{}", pid, chinese_content);

                        println!("🎯 捕获Claude Code会话中文输入 (PID: {}): {}",
                            pid,
                            chinese_content.chars().take(50).collect::<String>()
                        );

                        self.save_session_prompt(pid, &chinese_content).await?;
                    } else {
                        println!("🔍 [DEBUG] 中文行被过滤器排除: {}", line.chars().take(30).collect::<String>());
                    }
                } else {
                    println!("🔍 [DEBUG] 中文行不符合用户输入模式: {}", line.chars().take(30).collect::<String>());
                }
            }
        }

        Ok(())
    }

    /// 判断是否是用户输入行
    fn is_user_input_line(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // 过滤掉太短或空白行
        if trimmed.len() < 2 {
            return false;
        }

        // 过滤系统输出和Claude响应
        let system_indicators = [
            "Claude:", "Assistant:", "🤖", "✅", "❌", "⚠️", "💡",
            "[INFO]", "[DEBUG]", "[ERROR]", "[WARN]",
            "Loading", "Saving", "Found", "Error:",
            "usage:", "Usage:", "Options:",
            "http://", "https://", "ftp://",
            "exit", "quit", "/help", "/quit",
        ];

        for indicator in &system_indicators {
            if trimmed.contains(indicator) {
                return false;
            }
        }

        // Claude Code特定的输出模式
        let claude_output_patterns = [
            r"^\[.*\].*$",              // 时间戳格式输出
            r"^➤.*$",                   // Claude Code 提示符
            r"^›.*$",                   // Claude Code 箭头提示
            r"^.*\d{4}-\d{2}-\d{2}.*$", // 日期格式
            r"^.*\(\d+.*bytes?\).*$",   // 文件大小信息
        ];

        for pattern_str in &claude_output_patterns {
            if let Ok(regex) = Regex::new(pattern_str) {
                if regex.is_match(trimmed) {
                    return false;
                }
            }
        }

        // 检查是否包含中文且长度合理
        if self.chinese_filter.contains_chinese(trimmed) {
            // 更严格的中文检测：确保有足够的中文内容
            let chinese_count = self.chinese_filter.count_chinese_chars(trimmed);
            return chinese_count >= 2 && trimmed.len() >= 5;
        }

        false
    }

    /// 监控Claude Code配置文件
    async fn monitor_claude_config_files(&mut self) -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let claude_dirs = vec![
            PathBuf::from(format!("{}/.claude", home)),
            PathBuf::from(format!("{}/.config/claude", home)),
            PathBuf::from("./claude"),
        ];

        for dir in claude_dirs {
            if dir.exists() {
                println!("📁 监控Claude配置目录: {}", dir.display());
                // 这里可以添加文件监控逻辑
            }
        }

        Ok(())
    }

    /// 检查Claude会话文件
    async fn check_claude_session_files(&mut self) -> Result<()> {
        // 检查常见的Claude会话文件位置
        let session_files = vec![
            "claude_history.txt",
            ".claude_session",
            "claude_prompts.log",
        ];

        for file in session_files {
            if Path::new(file).exists() {
                if let Ok(content) = fs::read_to_string(file) {
                    // 分析新的内容
                    let file_modified = fs::metadata(file)?.modified()?;
                    if file_modified > self.last_scan_time {
                        self.analyze_captured_content(&content, 0).await?;
                    }
                }
            }
        }

        self.last_scan_time = SystemTime::now();
        Ok(())
    }

    /// 保存会话提示词
    async fn save_session_prompt(&mut self, pid: u32, content: &str) -> Result<()> {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir)?;

        let prompts_file = data_dir.join("claude_session_prompts.md");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let formatted_content = format!(
            "## {} (会话监控 - PID: {})\n\n```\n{}\n```\n\n",
            chrono::DateTime::from_timestamp(timestamp as i64, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S"),
            pid,
            content
        );

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&prompts_file)?;

        use std::io::Write;
        file.write_all(formatted_content.as_bytes())?;

        // 显示HTTP状态码风格的成功消息
        let preview = if content.len() > 30 {
            format!("{}...", &content[..30])
        } else {
            content.to_string()
        };

        println!("📥 [{}] 200 OK - Claude会话提示词已保存 | PID: {}",
            chrono::Local::now().format("%H:%M:%S%.3f"),
            pid
        );
        println!("   └─ 📄 Content: {}", preview);

        Ok(())
    }

    /// 打印会话监控横幅
    fn print_session_monitor_banner(&self) {
        println!("\n{}", "=".repeat(60));
        println!("🎯 CLAUDE CODE SESSION MONITOR v1.0");
        println!("   实时会话监控模式 - 监控活跃Claude进程交互");
        println!("{}", "=".repeat(60));
        println!("⏰ 启动时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("🔧 扫描间隔: 3秒");
        println!("{}", "-".repeat(60));
    }

    /// 监控Claude Code的实际日志和缓存文件
    async fn monitor_claude_system_files(&mut self) -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

        // Claude Code常见的系统文件位置
        let system_file_paths = vec![
            // Claude Code配置和日志目录
            format!("{}/.claude/logs", home),
            format!("{}/.claude/cache", home),
            format!("{}/.claude/history.jsonl", home),
            format!("{}/.claude/sessions", home),

            // 可能的临时文件
            "/tmp/claude-input.txt".to_string(),
            "/tmp/claude-session.txt".to_string(),

            // 应用特定缓存
            format!("{}/.config/claude", home),
            format!("{}/Library/Caches/claude", home),
            format!("{}/Library/Application Support/claude", home),

            // 当前工作目录中可能的Claude文件
            ".claude".to_string(),
            "claude.log".to_string(),
            "claude-session.log".to_string(),
        ];

        for path_str in system_file_paths {
            let path = Path::new(&path_str);

            // 检查是否为目录
            if path.is_dir() {
                // 扫描目录中的文件
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                self.check_file_for_recent_changes(&entry.path()).await?;
                            }
                        }
                    }
                }
            } else if path.is_file() {
                // 直接检查文件
                self.check_file_for_recent_changes(path).await?;
            }
        }

        Ok(())
    }

    /// 检查文件是否有最近更改
    async fn check_file_for_recent_changes(&mut self, file_path: &Path) -> Result<()> {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                // 只检查最近10秒内修改的文件
                let ten_seconds_ago = SystemTime::now()
                    .checked_sub(Duration::from_secs(10))
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                if modified > ten_seconds_ago {
                    // 读取文件内容并分析
                    if let Ok(content) = tokio::fs::read_to_string(file_path).await {
                        // 只分析文本文件，跳过二进制文件
                        if content.is_ascii() || content.chars().all(|c| !c.is_control() || c.is_whitespace()) {
                            println!("📄 检测到文件更新: {}", file_path.display());
                            self.analyze_captured_content(&content, 0).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取会话统计信息
    pub fn get_session_stats(&self) -> String {
        let last_scan_datetime: chrono::DateTime<chrono::Local> = self.last_scan_time.into();
        format!(
            "📊 Claude会话监控统计\n\
            ==================\n\
            活跃会话数: {}\n\
            上次扫描: {}\n",
            self.active_sessions.len(),
            last_scan_datetime.format("%Y-%m-%d %H:%M:%S")
        )
    }
}