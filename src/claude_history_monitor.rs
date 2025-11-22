// Claude Code历史文件监控模块 - 监控~/.claude/history.jsonl
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashSet;
use tokio::time::{interval, Duration};
use serde_json::Value;
use anyhow::Result;
use crate::chinese_filter::ChineseFilter;

pub struct ClaudeHistoryMonitor {
    chinese_filter: ChineseFilter,
    history_file_path: String,
    last_file_size: u64,
    processed_timestamps: HashSet<u64>,
    successful_records: u64,
}

impl ClaudeHistoryMonitor {
    pub fn new(chinese_filter: ChineseFilter) -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let history_file_path = format!("{}/.claude/history.jsonl", home);

        Ok(Self {
            chinese_filter,
            history_file_path,
            last_file_size: 0,
            processed_timestamps: HashSet::new(),
            successful_records: 0,
        })
    }

    /// 启动Claude历史文件监控
    pub async fn start_history_monitoring(&mut self) -> Result<()> {
        self.print_history_monitor_banner();

        // 检查历史文件是否存在
        if !Path::new(&self.history_file_path).exists() {
            anyhow::bail!("Claude历史文件不存在: {}", self.history_file_path);
        }

        // 显示监控状态
        self.print_monitoring_status();

        // 初始扫描现有历史
        self.scan_existing_history().await?;

        // 启动实时监控
        let mut file_check_interval = interval(Duration::from_secs(2));

        println!("🎯 监控已启动，按 Ctrl+C 停止...");
        println!();

        // 监控主循环，等待 Ctrl+C 信号
        tokio::select! {
            _ = async {
                loop {
                    file_check_interval.tick().await;
                    if let Err(e) = self.check_history_file_changes().await {
                        eprintln!("⚠️ 文件检查错误: {}", e);
                    }
                }
            } => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\n🛑 收到停止信号，正在关闭监控服务...");
                return Ok(());
            }
        }

        Ok(())
    }

    /// 扫描现有历史记录
    async fn scan_existing_history(&mut self) -> Result<()> {
        println!("🔍 扫描Claude历史记录...");

        match self.process_history_file(true).await {
            Ok(count) => {
                if count > 0 {
                    println!("📄 发现{}条包含中文的历史交互", count);
                } else {
                    println!("📝 没有发现中文交互记录");
                }
            }
            Err(e) => {
                println!("⚠️  历史文件处理错误: {}", e);
            }
        }

        // 更新文件大小基准
        if let Ok(metadata) = fs::metadata(&self.history_file_path) {
            self.last_file_size = metadata.len();
        }

        Ok(())
    }

    /// 检查历史文件变化
    async fn check_history_file_changes(&mut self) -> Result<()> {
        if let Ok(metadata) = fs::metadata(&self.history_file_path) {
            let current_size = metadata.len();

            // 文件大小发生变化，说明有新记录
            if current_size != self.last_file_size {
                println!("📝 检测到Claude历史文件更新: {} bytes → {} bytes",
                    self.last_file_size, current_size);

                match self.process_history_file(false).await {
                    Ok(new_count) => {
                        if new_count > 0 {
                            println!("📊 本次检查发现{}条新的中文交互", new_count);
                        }
                    }
                    Err(e) => {
                        println!("⚠️  处理新记录失败: {}", e);
                    }
                }

                self.last_file_size = current_size;
            }
        }

        Ok(())
    }

    /// 处理历史文件
    async fn process_history_file(&mut self, is_initial_scan: bool) -> Result<usize> {
        let content = fs::read_to_string(&self.history_file_path)?;
        let mut new_records = 0;

        // 按行解析JSONL格式
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(line) {
                Ok(json) => {
                    if let Some(record_count) = self.process_json_record(&json, is_initial_scan).await? {
                        new_records += record_count;
                    }
                }
                Err(e) => {
                    // 忽略JSON解析错误，继续处理其他行
                    if is_initial_scan {
                        println!("⚠️  跳过无效JSON行: {}", e);
                    }
                }
            }
        }

        Ok(new_records)
    }

    /// 处理单个JSON记录
    async fn process_json_record(&mut self, json: &Value, is_initial_scan: bool) -> Result<Option<usize>> {
        // 提取关键字段
        let display = json.get("display")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let timestamp = json.get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let project = json.get("project")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let session_id = json.get("sessionId")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // 避免重复处理
        if self.processed_timestamps.contains(&timestamp) {
            return Ok(None);
        }

        self.processed_timestamps.insert(timestamp);

        // 检查是否包含中文内容
        if let Some(chinese_content) = self.chinese_filter.filter_prompt(display) {
            let datetime = SystemTime::UNIX_EPOCH + Duration::from_millis(timestamp);
            self.save_claude_interaction(datetime, &chinese_content, project, session_id).await?;

            if !is_initial_scan {
                println!("🎯 捕获Claude Code中文交互: {}",
                    self.safe_truncate(&chinese_content, 50));
            }

            return Ok(Some(1));
        }

        Ok(None)
    }

    /// 保存Claude交互到本地文件
    async fn save_claude_interaction(
        &mut self,
        timestamp: SystemTime,
        content: &str,
        project: &str,
        session_id: &str
    ) -> Result<()> {
        let data_dir = Path::new("./data");
        fs::create_dir_all(data_dir)?;

        let prompts_file = data_dir.join("claude_history_prompts.md");
        let timestamp_secs = timestamp.duration_since(UNIX_EPOCH)?.as_secs();

        let formatted_content = format!(
            "## {} (Claude历史监控)\n\n**项目**: {}\n**会话ID**: {}\n\n```\n{}\n```\n\n",
            chrono::DateTime::from_timestamp(timestamp_secs as i64, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S"),
            project,
            session_id,
            content
        );

        use std::io::Write;
        match fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&prompts_file)
        {
            Ok(mut file) => {
                file.write_all(formatted_content.as_bytes())?;

                // 成功保存，显示HTTP状态码风格响应
                self.successful_records += 1;
                let preview = self.safe_truncate(content, 30);

                println!("📥 [{}] 200 OK - Claude历史交互已保存 | Records: {}",
                    chrono::Local::now().format("%H:%M:%S%.3f"),
                    self.successful_records
                );
                println!("   └─ 📄 Content: {}", preview);
                println!("   └─ 🏗️  Project: {}", project);

                Ok(())
            }
            Err(e) => {
                println!("❌ [{}] 500 ERROR - 文件写入失败: {}",
                    chrono::Local::now().format("%H:%M:%S%.3f"),
                    e
                );
                Err(e.into())
            }
        }
    }

    /// 打印历史监控横幅
    fn print_history_monitor_banner(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📚 CLAUDE HISTORY MONITOR v1.0");
        println!("   Claude Code历史记录监控 - 直接捕获用户交互");
        println!("{}", "=".repeat(60));
        println!("⏰ 启动时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("🔧 检查间隔: 2秒");
        println!("{}", "-".repeat(60));
    }

    /// 显示监控状态
    fn print_monitoring_status(&self) {
        println!("📊 监控状态总览");
        println!("{}", "-".repeat(60));

        // 显示监控文件
        if let Ok(metadata) = fs::metadata(&self.history_file_path) {
            println!("📂 Claude历史文件:");
            println!("   ✅ {} ({} bytes)",
                self.history_file_path,
                metadata.len()
            );
        }

        println!();
        println!("🎯 监控目标:");
        println!("   📝 用户输入的中文提示词");
        println!("   💬 Claude Code会话交互");
        println!("   🏗️  项目上下文信息");

        println!("{}", "-".repeat(60));
        println!("🚀 历史监控服务已就绪，监控Claude交互...");
        println!();
    }

    /// 获取监控统计信息
    pub fn get_history_stats(&self) -> String {
        format!(
            "📊 Claude历史监控统计\n\
            ==================\n\
            成功记录数: {}\n\
            已处理时间戳: {}\n\
            监控文件: {}\n",
            self.successful_records,
            self.processed_timestamps.len(),
            self.history_file_path
        )
    }

    /// 安全截取字符串，避免UTF-8字符边界问题
    fn safe_truncate(&self, text: &str, max_chars: usize) -> String {
        if text.chars().count() <= max_chars {
            text.to_string()
        } else {
            format!("{}...", text.chars().take(max_chars).collect::<String>())
        }
    }
}