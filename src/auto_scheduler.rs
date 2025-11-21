// 自动化调度器 - 定时任务和自动上传
use tokio_cron_scheduler::{JobScheduler, Job};
use chrono::{Local, DateTime, TimeZone};
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use crate::chinese_filter::ChineseFilter;
use crate::cloudflare_uploader::{CloudflareUploader, CloudflareConfig};

#[derive(Debug, Clone)]
pub struct AutoConfig {
    pub upload_interval_hours: u64,
    pub data_dir: String,
    pub prompts_file: String,
    pub backup_dir: String,
    pub max_backups: usize,
}

pub struct AutoScheduler {
    config: AutoConfig,
    cloudflare_config: CloudflareConfig,
    chinese_filter: ChineseFilter,
    uploader: Arc<CloudflareUploader>,
    last_upload_time: Arc<Mutex<DateTime<Local>>>,
    scheduler: JobScheduler,
}

impl AutoScheduler {
    pub async fn new(
        config: AutoConfig,
        cloudflare_config: CloudflareConfig,
        chinese_filter: ChineseFilter,
    ) -> anyhow::Result<Self> {
        let uploader = Arc::new(CloudflareUploader::new(cloudflare_config.clone()));
        let scheduler = JobScheduler::new().await?;
        let last_upload_time = Arc::new(Mutex::new(Local::now()));

        Ok(Self {
            config,
            cloudflare_config,
            chinese_filter,
            uploader,
            last_upload_time,
            scheduler,
        })
    }

    /// 启动自动化调度
    pub async fn start(&mut self) -> anyhow::Result<()> {
        println!("🤖 启动自动化调度器...");

        // 测试Cloudflare连接
        self.uploader.test_connection().await?;

        // 创建必要的目录
        self.ensure_directories().await?;

        // 设置定时任务
        self.setup_scheduled_jobs().await?;

        // 启动调度器
        self.scheduler.start().await?;

        println!("✅ 自动化调度器已启动");
        println!("⏰ 上传间隔: {}小时", self.config.upload_interval_hours);
        println!("📁 数据目录: {}", self.config.data_dir);

        Ok(())
    }

    /// 设置定时任务
    async fn setup_scheduled_jobs(&mut self) -> anyhow::Result<()> {
        let uploader = Arc::clone(&self.uploader);
        let config = self.config.clone();
        let filter = self.chinese_filter.clone();
        let last_upload = Arc::clone(&self.last_upload_time);

        // 创建定时上传任务
        let cron_expr = format!("0 0 */{} * * *", self.config.upload_interval_hours);
        let upload_job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
            let uploader = Arc::clone(&uploader);
            let config = config.clone();
            let filter = filter.clone();
            let last_upload = Arc::clone(&last_upload);

            Box::pin(async move {
                println!("🔄 开始定时上传任务...");

                match Self::process_and_upload(uploader, config, filter, last_upload).await {
                    Ok(count) => {
                        if count > 0 {
                            println!("✅ 定时上传完成: {}条中文提示词", count);
                        } else {
                            println!("ℹ️  无新的中文提示词需要上传");
                        }
                    }
                    Err(e) => {
                        println!("❌ 定时上传失败: {}", e);
                    }
                }
            })
        })?;

        self.scheduler.add(upload_job).await?;

        // 创建每日备份任务
        let backup_config = self.config.clone();
        let backup_job = Job::new_async("0 0 2 * * *", move |_uuid, _l| {
            let config = backup_config.clone();
            Box::pin(async move {
                println!("💾 开始每日备份...");
                if let Err(e) = Self::create_backup(&config).await {
                    println!("❌ 备份失败: {}", e);
                } else {
                    println!("✅ 每日备份完成");
                }
            })
        })?;

        self.scheduler.add(backup_job).await?;

        println!("📅 定时任务已设置:");
        println!("  - 自动上传: 每{}小时执行一次", self.config.upload_interval_hours);
        println!("  - 每日备份: 每天02:00执行");

        Ok(())
    }

    /// 手动触发上传
    pub async fn manual_upload(&self) -> anyhow::Result<usize> {
        println!("🚀 手动触发上传...");
        Self::process_and_upload(
            Arc::clone(&self.uploader),
            self.config.clone(),
            self.chinese_filter.clone(),
            Arc::clone(&self.last_upload_time),
        ).await
    }

    /// 处理和上传提示词
    async fn process_and_upload(
        uploader: Arc<CloudflareUploader>,
        config: AutoConfig,
        filter: ChineseFilter,
        last_upload_time: Arc<Mutex<DateTime<Local>>>,
    ) -> anyhow::Result<usize> {
        let prompts_path = Path::new(&config.data_dir).join(&config.prompts_file);
        let shell_prompts_path = Path::new(&config.data_dir).join("shell_captured_prompts.md");

        let mut all_content = String::new();

        // 读取PTY模式收集的提示词
        if prompts_path.exists() {
            all_content.push_str(&fs::read_to_string(&prompts_path)?);
        }

        // 读取Shell监控模式收集的提示词
        if shell_prompts_path.exists() {
            if !all_content.is_empty() {
                all_content.push_str("\n\n");
            }
            all_content.push_str(&fs::read_to_string(&shell_prompts_path)?);
        }

        if all_content.is_empty() {
            return Ok(0);
        }

        // 获取上次上传时间
        let last_time = *last_upload_time.lock().unwrap();

        // 解析和过滤新的中文提示词
        let new_chinese_prompts = Self::extract_new_chinese_prompts(&all_content, &filter, last_time)?;

        if !new_chinese_prompts.is_empty() {
            // 上传到Cloudflare
            uploader.upload_batch(new_chinese_prompts.clone()).await?;

            // 更新上传时间
            *last_upload_time.lock().unwrap() = Local::now();

            // 创建增量备份
            Self::create_incremental_backup(&config, &new_chinese_prompts).await?;
        }

        Ok(new_chinese_prompts.len())
    }

    /// 提取新的中文提示词
    fn extract_new_chinese_prompts(
        content: &str,
        filter: &ChineseFilter,
        since: DateTime<Local>,
    ) -> anyhow::Result<Vec<String>> {
        let mut chinese_prompts = Vec::new();
        let mut current_timestamp: Option<DateTime<Local>> = None;
        let mut current_content = String::new();

        for line in content.lines() {
            // 检测时间戳行（格式：## 2024-11-18 15:30:45）
            if line.starts_with("## ") {
                // 保存上一个提示词
                if let Some(timestamp) = current_timestamp {
                    if timestamp > since {
                        if let Some(filtered) = filter.filter_prompt(&current_content) {
                            chinese_prompts.push(filtered);
                        }
                    }
                }

                // 解析新的时间戳
                let timestamp_str = line.trim_start_matches("## ");
                current_timestamp = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|dt| Local.from_local_datetime(&dt).single())
                    .flatten();
                current_content.clear();
            } else if line.starts_with("```") {
                // 跳过代码块标记
                continue;
            } else if !line.trim().is_empty() {
                // 积累内容
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }

        // 处理最后一个提示词
        if let Some(timestamp) = current_timestamp {
            if timestamp > since {
                if let Some(filtered) = filter.filter_prompt(&current_content) {
                    chinese_prompts.push(filtered);
                }
            }
        }

        println!("📊 提取到{}条新的中文提示词（自{}以来）",
            chinese_prompts.len(),
            since.format("%Y-%m-%d %H:%M:%S")
        );

        Ok(chinese_prompts)
    }

    /// 确保目录存在
    async fn ensure_directories(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.config.data_dir)?;
        fs::create_dir_all(&self.config.backup_dir)?;
        Ok(())
    }

    /// 创建备份
    async fn create_backup(config: &AutoConfig) -> anyhow::Result<()> {
        let prompts_path = Path::new(&config.data_dir).join(&config.prompts_file);

        if !prompts_path.exists() {
            return Ok(());
        }

        let backup_name = format!("prompts_backup_{}.md", Local::now().format("%Y%m%d_%H%M%S"));
        let backup_path = Path::new(&config.backup_dir).join(backup_name);

        fs::copy(&prompts_path, &backup_path)?;

        // 清理旧备份
        Self::cleanup_old_backups(config).await?;

        println!("💾 备份已创建: {}", backup_path.display());
        Ok(())
    }

    /// 创建增量备份
    async fn create_incremental_backup(config: &AutoConfig, prompts: &[String]) -> anyhow::Result<()> {
        let backup_name = format!("incremental_{}.json", Local::now().format("%Y%m%d_%H%M%S"));
        let backup_path = Path::new(&config.backup_dir).join(backup_name);

        let backup_data = serde_json::json!({
            "timestamp": Local::now().to_rfc3339(),
            "prompts": prompts,
            "count": prompts.len()
        });

        fs::write(&backup_path, serde_json::to_string_pretty(&backup_data)?)?;
        Ok(())
    }

    /// 清理旧备份
    async fn cleanup_old_backups(config: &AutoConfig) -> anyhow::Result<()> {
        let backup_dir = Path::new(&config.backup_dir);

        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<_> = fs::read_dir(backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .map_or(false, |ext| ext == "md" || ext == "json")
            })
            .collect();

        backups.sort_by_key(|entry| entry.metadata().ok().and_then(|m| m.modified().ok()));

        // 保留最新的N个备份
        if backups.len() > config.max_backups {
            let to_remove = backups.len() - config.max_backups;
            for backup in backups.iter().take(to_remove) {
                fs::remove_file(backup.path())?;
            }
            println!("🗑️  已清理{}个旧备份", to_remove);
        }

        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.scheduler.shutdown().await?;
        println!("⏹️  自动化调度器已停止");
        Ok(())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> anyhow::Result<String> {
        let prompts_path = Path::new(&self.config.data_dir).join(&self.config.prompts_file);
        let last_upload = *self.last_upload_time.lock().unwrap();

        let mut stats = String::new();
        stats.push_str("📊 Prompter 自动化统计\n");
        stats.push_str("===================\n");
        stats.push_str(&format!("上次上传时间: {}\n", last_upload.format("%Y-%m-%d %H:%M:%S")));
        stats.push_str(&format!("上传间隔: {}小时\n", self.config.upload_interval_hours));

        if prompts_path.exists() {
            let content = fs::read_to_string(&prompts_path)?;
            let total_lines = content.lines().count();
            let chinese_count = self.chinese_filter.chinese_regex.find_iter(&content).count();
            stats.push_str(&format!("提示词文件: {} ({}行)\n", prompts_path.display(), total_lines));
            stats.push_str(&format!("中文字符总数: {}\n", chinese_count));
        }

        Ok(stats)
    }
}