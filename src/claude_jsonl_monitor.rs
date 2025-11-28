// Claude Code JSONL监控模块 - 基于项目文件的真实监控方案
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time;
use anyhow::Result;
use crate::chinese_filter::ChineseFilter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ClaudeMessage {
    #[serde(rename = "type")]
    message_type: String,
    message: Option<MessageContent>,
    timestamp: Option<String>,
    uuid: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    role: String,
    content: String,
}

pub struct ClaudeJsonlMonitor {
    chinese_filter: ChineseFilter,
    claude_projects_dir: PathBuf,
    processed_messages: HashMap<String, SystemTime>,
    last_scan_time: SystemTime,
}

impl ClaudeJsonlMonitor {
    pub fn new(chinese_filter: ChineseFilter) -> Result<Self> {
        // Claude项目目录
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let claude_projects_dir = PathBuf::from(format!("{}/.claude/projects", home));

        Ok(Self {
            chinese_filter,
            claude_projects_dir,
            processed_messages: HashMap::new(),
            last_scan_time: SystemTime::now(),
        })
    }

    /// 启动JSONL监控
    pub async fn start_jsonl_monitoring(&mut self) -> Result<()> {
        self.print_monitor_banner();

        // 检查Claude项目目录
        if !self.claude_projects_dir.exists() {
            println!("❌ Claude项目目录不存在: {}", self.claude_projects_dir.display());
            println!("💡 请确保Claude Code已经运行过并创建了项目文件");
            return Ok(());
        }

        self.scan_existing_projects().await?;

        // 主监控循环
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            // 监控所有项目的JSONL文件
            self.monitor_all_project_files().await?;

            // 清理过期的消息记录
            self.cleanup_processed_messages().await;
        }
    }

    /// 扫描现有项目
    async fn scan_existing_projects(&mut self) -> Result<()> {
        println!("🔍 扫描Claude项目目录...");

        if let Ok(entries) = fs::read_dir(&self.claude_projects_dir) {
            let mut project_count = 0;
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    project_count += 1;
                    let project_name = entry.file_name().to_string_lossy().to_string();

                    // 检查项目目录中的JSONL文件
                    let jsonl_count = self.count_jsonl_files(&entry.path()).await?;

                    if project_count <= 5 {  // 只显示前5个项目
                        println!("  📂 {} ({} 个JSONL文件)", project_name, jsonl_count);
                    }
                }
            }

            if project_count > 5 {
                println!("  ... 还有 {} 个项目", project_count - 5);
            }

            println!("📊 总共 {} 个项目目录", project_count);
        }

        Ok(())
    }

    /// 统计JSONL文件数量
    async fn count_jsonl_files(&self, project_dir: &PathBuf) -> Result<usize> {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "jsonl" {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    /// 监控所有项目文件
    async fn monitor_all_project_files(&mut self) -> Result<()> {
        if let Ok(entries) = fs::read_dir(&self.claude_projects_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    self.scan_project_directory(&entry.path()).await?;
                }
            }
        }
        Ok(())
    }

    /// 扫描单个项目目录
    async fn scan_project_directory(&mut self, project_dir: &PathBuf) -> Result<()> {
        if let Ok(entries) = fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "jsonl" {
                        self.process_jsonl_file(&path).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 处理单个JSONL文件
    async fn process_jsonl_file(&mut self, jsonl_path: &PathBuf) -> Result<()> {
        // 检查文件修改时间
        if let Ok(metadata) = fs::metadata(jsonl_path) {
            if let Ok(modified) = metadata.modified() {
                // 只处理最近5分钟内修改的文件
                let five_minutes_ago = SystemTime::now()
                    .checked_sub(Duration::from_secs(300))
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                if modified < five_minutes_ago {
                    return Ok(());
                }
            }
        }

        // 读取JSONL文件内容
        match tokio::fs::read_to_string(jsonl_path).await {
            Ok(content) => {
                let mut new_messages = 0;
                for (line_num, line) in content.lines().enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<ClaudeMessage>(line) {
                        Ok(message) => {
                            if self.should_process_message(&message) {
                                if let Some(user_content) = self.extract_user_content(&message) {
                                    if let Some(chinese_content) = self.chinese_filter.filter_prompt(&user_content) {
                                        // 记录这个消息已处理
                                        if let Some(uuid) = &message.uuid {
                                            self.processed_messages.insert(uuid.clone(), SystemTime::now());
                                        }

                                        self.save_jsonl_prompt(&chinese_content, &message, jsonl_path).await?;
                                        new_messages += 1;

                                        println!("🎯 捕获Claude Code中文输入: {}",
                                            chinese_content.chars().take(50).collect::<String>()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // 详细的JSON解析错误调试
                            if line.contains("\"type\":\"user\"") {
                                println!("🔍 [DEBUG] JSON解析失败 (行 {}): {}", line_num + 1, e);
                                println!("🔍 [DEBUG] 原始行内容: {}",
                                    line.chars().take(100).collect::<String>());
                            }
                        }
                    }
                }

                if new_messages > 0 {
                    println!("📝 [{}] 处理了 {} 条新消息",
                        jsonl_path.file_name().unwrap_or_default().to_string_lossy(),
                        new_messages);
                }
            }
            Err(e) => {
                println!("⚠️ 无法读取JSONL文件 {}: {}", jsonl_path.display(), e);
            }
        }

        Ok(())
    }

    /// 判断是否应该处理这个消息
    fn should_process_message(&self, message: &ClaudeMessage) -> bool {
        // 只处理用户类型的消息
        if message.message_type != "user" {
            return false;
        }

        // 检查是否已经处理过
        if let Some(uuid) = &message.uuid {
            if self.processed_messages.contains_key(uuid) {
                return false;
            }
        }

        true
    }

    /// 提取用户输入内容
    fn extract_user_content(&self, message: &ClaudeMessage) -> Option<String> {
        if let Some(msg_content) = &message.message {
            if msg_content.role == "user" {
                return Some(msg_content.content.clone());
            }
        }
        None
    }

    /// 保存JSONL提示词
    async fn save_jsonl_prompt(&mut self, content: &str, message: &ClaudeMessage, source_file: &PathBuf) -> Result<()> {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir)?;

        let prompts_file = data_dir.join("claude_session_prompts.md");

        // 使用消息的时间戳，或当前时间
        let timestamp_str = message.timestamp.as_ref()
            .map(|ts| ts.clone())
            .unwrap_or_else(|| {
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
            });

        // 清理时间戳格式
        let clean_timestamp = timestamp_str
            .replace("T", " ")
            .replace("Z", "")
            .split('.')
            .next()
            .unwrap_or(&timestamp_str)
            .to_string();

        let source_project = source_file.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| "unknown".into());

        let formatted_content = format!(
            "## {} (Claude Code JSONL监控)\n\n**项目**: {}\n**会话ID**: {}\n\n```\n{}\n```\n\n",
            clean_timestamp,
            source_project,
            message.session_id.as_ref().unwrap_or(&"unknown".to_string()),
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
            let safe_truncated = Self::safe_truncate(content, 30);
            format!("{}...", safe_truncated)
        } else {
            content.to_string()
        };

        println!("📥 [{}] 200 OK - Claude JSONL提示词已保存",
            chrono::Local::now().format("%H:%M:%S%.3f")
        );
        println!("   └─ 📄 Content: {}", preview);
        println!("   └─ 📁 Project: {}", source_project);

        Ok(())
    }

    /// 清理过期的处理记录
    async fn cleanup_processed_messages(&mut self) {
        let one_hour_ago = SystemTime::now()
            .checked_sub(Duration::from_secs(3600))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let before_count = self.processed_messages.len();
        self.processed_messages.retain(|_, timestamp| *timestamp > one_hour_ago);
        let after_count = self.processed_messages.len();

        if before_count > after_count {
            println!("🧹 清理了 {} 个过期消息记录", before_count - after_count);
        }
    }

    /// 打印监控横幅
    fn print_monitor_banner(&self) {
        println!("\n{}", "=".repeat(60));
        println!("🎯 CLAUDE CODE JSONL MONITOR v2.0");
        println!("   真实JSONL监控模式 - 基于项目文件的精准监控");
        println!("{}", "=".repeat(60));
        println!("⏰ 启动时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("🔧 扫描间隔: 1秒 (实时响应)");
        println!("📁 监控目录: {}", self.claude_projects_dir.display());
        println!("{}", "-".repeat(60));
    }

    /// 获取监控统计信息
    pub fn get_monitor_stats(&self) -> String {
        let last_scan_datetime: chrono::DateTime<chrono::Local> = self.last_scan_time.into();
        format!(
            "📊 Claude JSONL监控统计\n\
            ==================\n\
            已处理消息: {}\n\
            上次扫描: {}\n\
            监控目录: {}\n",
            self.processed_messages.len(),
            last_scan_datetime.format("%Y-%m-%d %H:%M:%S"),
            self.claude_projects_dir.display()
        )
    }

    /// 安全截取字符串，避免UTF-8字符边界问题
    fn safe_truncate(s: &str, max_chars: usize) -> &str {
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
}