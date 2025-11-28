use anyhow::Result;
use psutil::process::Process;
#[cfg(target_os = "macos")]
use psutil::process::os::macos::ProcessExt;
#[cfg(target_os = "linux")]
use psutil::process::os::unix::ProcessExt;
use std::collections::HashSet;
use std::time::Duration;
use chrono::Local;
use crate::chinese_filter::ChineseFilter;

/// 全局进程监控器 - 监控所有终端中的Claude相关进程
pub struct GlobalProcessMonitor {
    chinese_filter: ChineseFilter,
    known_processes: HashSet<u32>,
    output_file: String,
}

impl GlobalProcessMonitor {
    pub fn new(chinese_filter: ChineseFilter) -> Result<Self> {
        Ok(Self {
            chinese_filter,
            known_processes: HashSet::new(),
            output_file: "./data/global_claude_prompts.md".to_string(),
        })
    }

    /// 启动全局监控
    pub async fn start_global_monitoring(&mut self) -> Result<()> {
        println!("🌍 全局Claude进程监控已启动");
        println!("📊 监控目标: 所有终端中的Claude相关进程");
        println!("📁 保存位置: {}", self.output_file);
        println!("⏰ 扫描间隔: 3秒");
        println!();

        // 确保输出目录存在
        if let Some(parent) = std::path::Path::new(&self.output_file).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut scan_interval = tokio::time::interval(Duration::from_secs(3));

        loop {
            scan_interval.tick().await;

            match self.scan_claude_processes().await {
                Ok(count) => {
                    if count > 0 {
                        println!("[{}] 🔍 扫描完成: 发现 {} 个Claude进程",
                            Local::now().format("%H:%M:%S"), count);
                    }
                }
                Err(e) => {
                    eprintln!("[{}] ❌ 扫描错误: {}",
                        Local::now().format("%H:%M:%S"), e);
                }
            }
        }
    }

    /// 扫描Claude相关进程
    async fn scan_claude_processes(&mut self) -> Result<usize> {
        let mut claude_processes = Vec::new();
        let mut current_pids = HashSet::new();

        // 获取所有进程
        let processes = psutil::process::processes()?;

        for process_result in processes {
            if let Ok(process) = process_result {
                let pid = process.pid();
                current_pids.insert(pid);

                // 检查是否是Claude相关进程
                if let Ok(name) = process.name() {
                    if self.is_claude_related_process(&name) {
                        // 检查是否是新进程
                        if !self.known_processes.contains(&pid) {
                            println!("[{}] 🎯 发现新Claude进程: {} (PID: {})",
                                Local::now().format("%H:%M:%S"), name, pid);
                            self.known_processes.insert(pid);
                        }

                        claude_processes.push(process);
                    }
                }
            }
        }

        // 清理已结束的进程
        self.known_processes.retain(|pid| current_pids.contains(pid));

        let process_count = claude_processes.len();

        // 监控每个Claude进程的输入输出
        for process in &claude_processes {
            if let Err(e) = self.monitor_process_io(process).await {
                // 静默处理错误，避免输出过多日志
                if e.to_string().contains("Permission denied") {
                    // 权限问题很常见，不需要输出
                } else {
                    eprintln!("进程监控错误: {}", e);
                }
            }
        }

        Ok(process_count)
    }

    /// 检查是否是Claude相关进程
    fn is_claude_related_process(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("claude") ||
        name_lower == "node" || // Claude Code通常运行在Node环境
        name_lower.contains("claude-code") ||
        name_lower.contains("claude_code") ||
        name_lower.contains("anthropic")
    }

    /// 监控进程的输入输出
    async fn monitor_process_io(&self, process: &Process) -> Result<()> {
        // 获取进程的命令行参数
        if let Ok(cmdline_vec) = process.cmdline() {
            if let Some(full_command) = cmdline_vec {
                // 检查是否包含中文内容
                if self.chinese_filter.contains_sufficient_chinese(&full_command) {
                    let chinese_content = self.chinese_filter.extract_chinese_content(&full_command);

                    if chinese_content.len() >= 10 { // 至少10个字符
                        let timestamp = Local::now().format("%H:%M:%S");
                        let preview = if chinese_content.len() > 50 {
                            format!("{}...", &chinese_content[..50])
                        } else {
                            chinese_content.clone()
                        };

                        println!("[{}] 📥 捕获Claude进程中文输入: {}", timestamp, preview);

                        // 保存到文件
                        if let Err(e) = self.save_chinese_input(&chinese_content, process).await {
                            eprintln!("保存失败: {}", e);
                        }
                    }
                }
            }
        }

        // 尝试获取进程的环境变量（如果可能的话）
        #[cfg(target_os = "macos")]
        if let Ok(environ) = process.environ() {
            for (key, value) in environ {
                if key.to_lowercase().contains("claude") || key.to_lowercase().contains("input") {
                    if self.chinese_filter.contains_sufficient_chinese(&value) {
                        let chinese_content = self.chinese_filter.extract_chinese_content(&value);
                        if chinese_content.len() >= 10 {
                            println!("[{}] 📥 从环境变量捕获中文: {}",
                                Local::now().format("%H:%M:%S"),
                                &chinese_content[..std::cmp::min(50, chinese_content.len())]);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 保存中文输入到文件
    async fn save_chinese_input(&self, content: &str, process: &Process) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let timestamp = Local::now();
        let process_info = format!("PID: {}, 名称: {}",
            process.pid(),
            process.name().unwrap_or_else(|_| "unknown".to_string())
        );

        let entry = format!(
            "## {} (全局进程监控)\n\n**进程信息**: {}\n\n```\n{}\n```\n\n---\n\n",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            process_info,
            content
        );

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_file)
            .await?;

        file.write_all(entry.as_bytes()).await?;
        file.sync_all().await?;

        println!("[{}] 💾 全局进程中文输入已保存",
            Local::now().format("%H:%M:%S"));

        Ok(())
    }

    /// 监控系统剪贴板（macOS特定）
    #[cfg(target_os = "macos")]
    async fn monitor_clipboard(&self) -> Result<()> {
        // 使用pbpaste命令获取剪贴板内容
        if let Ok(output) = tokio::process::Command::new("pbpaste").output().await {
            if let Ok(content) = String::from_utf8(output.stdout) {
                if self.chinese_filter.contains_sufficient_chinese(&content) {
                    let chinese_content = self.chinese_filter.extract_chinese_content(&content);
                    if chinese_content.len() >= 10 {
                        println!("[{}] 📋 从剪贴板检测到中文: {}",
                            Local::now().format("%H:%M:%S"),
                            &chinese_content[..std::cmp::min(30, chinese_content.len())]);
                    }
                }
            }
        }
        Ok(())
    }

    /// 获取所有活跃的终端窗口
    #[cfg(target_os = "macos")]
    async fn get_active_terminal_windows(&self) -> Result<Vec<String>> {
        let mut windows = Vec::new();

        // 使用AppleScript获取Terminal.app的窗口信息
        let script = r#"
        tell application "Terminal"
            repeat with w in windows
                set window_info to (contents of w as string)
                if window_info contains "claude" then
                    return window_info
                end if
            end repeat
        end tell
        "#;

        if let Ok(output) = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .await
        {
            if let Ok(content) = String::from_utf8(output.stdout) {
                if !content.trim().is_empty() {
                    windows.push(content);
                }
            }
        }

        Ok(windows)
    }
}

/// 安全截取字符串，避免UTF-8边界问题
fn safe_truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}