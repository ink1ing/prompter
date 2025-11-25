// 反馈间隔配置模块 - 管理用户的反馈推送频率设置
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local, Duration};

/// 反馈间隔类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntervalType {
    /// 基于时间间隔（小时）
    Time { hours: u32 },
    /// 基于提示词数量
    Number { count: u32 },
}

/// 反馈间隔配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackInterval {
    /// 间隔类型
    pub interval_type: IntervalType,
    /// 上次发送时间
    pub last_sent_time: Option<DateTime<Local>>,
    /// 上次发送后的提示词计数
    pub prompt_count_since_last: u32,
    /// 配置文件路径
    #[serde(skip)]
    config_path: PathBuf,
}

impl Default for FeedbackInterval {
    fn default() -> Self {
        Self {
            interval_type: IntervalType::Time { hours: 24 }, // 默认每24小时
            last_sent_time: None,
            prompt_count_since_last: 0,
            config_path: Self::get_default_config_path(),
        }
    }
}

impl FeedbackInterval {
    /// 获取默认配置文件路径
    fn get_default_config_path() -> PathBuf {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".prompter").join("feedback_interval.json")
    }

    /// 从文件加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::get_default_config_path();

        if !config_path.exists() {
            println!("📋 未找到反馈间隔配置，使用默认设置（24小时）");
            let mut config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let mut config: FeedbackInterval = serde_json::from_str(&content)?;
        config.config_path = config_path;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        // 确保配置目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&self.config_path, content)?;

        Ok(())
    }

    /// 设置基于时间的间隔
    pub fn set_time_based(&mut self, hours: u32) -> Result<()> {
        if hours == 0 {
            return Err(anyhow!("⚠️ 时间间隔必须大于0小时"));
        }

        self.interval_type = IntervalType::Time { hours };
        self.save()?;

        println!("✅ 已设置反馈间隔：每 {} 小时推送一次", hours);
        Ok(())
    }

    /// 设置基于数量的间隔
    pub fn set_number_based(&mut self, count: u32) -> Result<()> {
        if count == 0 {
            return Err(anyhow!("⚠️ 提示词数量必须大于0"));
        }

        self.interval_type = IntervalType::Number { count };
        // 重置计数器
        self.prompt_count_since_last = 0;
        self.save()?;

        println!("✅ 已设置反馈间隔：每 {} 个提示词推送一次", count);
        Ok(())
    }

    /// 增加提示词计数
    pub fn increment_prompt_count(&mut self) -> Result<()> {
        self.prompt_count_since_last += 1;
        self.save()?;
        Ok(())
    }

    /// 检查是否应该发送反馈
    pub fn should_send_feedback(&self) -> bool {
        match &self.interval_type {
            IntervalType::Time { hours } => {
                if let Some(last_sent) = self.last_sent_time {
                    let now = Local::now();
                    let elapsed = now.signed_duration_since(last_sent);
                    let threshold = Duration::hours(*hours as i64);

                    let should_send = elapsed >= threshold;

                    if should_send {
                        println!("⏰ 距离上次推送已过 {} 小时，触发反馈推送",
                            elapsed.num_hours());
                    } else {
                        let remaining_hours = (threshold - elapsed).num_hours();
                        println!("⏳ 距离下次推送还有约 {} 小时", remaining_hours);
                    }

                    should_send
                } else {
                    // 首次发送
                    println!("🎯 首次运行，触发反馈推送");
                    true
                }
            }
            IntervalType::Number { count } => {
                let should_send = self.prompt_count_since_last >= *count;

                if should_send {
                    println!("📊 已收集 {} 个提示词（目标: {}），触发反馈推送",
                        self.prompt_count_since_last, count);
                } else {
                    println!("📈 当前已收集 {} 个提示词，还需 {} 个触发推送",
                        self.prompt_count_since_last, count - self.prompt_count_since_last);
                }

                should_send
            }
        }
    }

    /// 标记已发送反馈
    pub fn mark_feedback_sent(&mut self) -> Result<()> {
        self.last_sent_time = Some(Local::now());
        self.prompt_count_since_last = 0;
        self.save()?;

        println!("✅ 已记录反馈发送时间");
        Ok(())
    }

    /// 获取当前配置的描述
    pub fn get_config_description(&self) -> String {
        let type_desc = match &self.interval_type {
            IntervalType::Time { hours } => format!("⏰ 时间间隔: 每 {} 小时", hours),
            IntervalType::Number { count } => format!("📊 数量间隔: 每 {} 个提示词", count),
        };

        let status_desc = if let Some(last_sent) = self.last_sent_time {
            format!("\n📅 上次推送: {}", last_sent.format("%Y-%m-%d %H:%M:%S"))
        } else {
            "\n📅 尚未推送过".to_string()
        };

        let count_desc = match &self.interval_type {
            IntervalType::Number { .. } => {
                format!("\n📈 当前计数: {}", self.prompt_count_since_last)
            }
            _ => String::new()
        };

        format!("{}{}{}", type_desc, status_desc, count_desc)
    }

    /// 重置配置为默认值
    pub fn reset_to_default(&mut self) -> Result<()> {
        self.interval_type = IntervalType::Time { hours: 24 };
        self.last_sent_time = None;
        self.prompt_count_since_last = 0;
        self.save()?;

        println!("🔄 已重置为默认配置（每24小时）");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_time_based_interval() {
        let mut interval = FeedbackInterval::default();
        interval.set_time_based(1).unwrap();

        assert!(matches!(interval.interval_type, IntervalType::Time { hours: 1 }));
        assert!(interval.should_send_feedback()); // 首次应该发送

        interval.mark_feedback_sent().unwrap();
        assert!(!interval.should_send_feedback()); // 刚发送完不应该再发送
    }

    #[test]
    fn test_number_based_interval() {
        let mut interval = FeedbackInterval::default();
        interval.set_number_based(5).unwrap();

        assert!(matches!(interval.interval_type, IntervalType::Number { count: 5 }));

        for _ in 0..4 {
            interval.increment_prompt_count().unwrap();
            assert!(!interval.should_send_feedback());
        }

        interval.increment_prompt_count().unwrap();
        assert!(interval.should_send_feedback());

        interval.mark_feedback_sent().unwrap();
        assert_eq!(interval.prompt_count_since_last, 0);
    }

    #[test]
    fn test_config_override() {
        let mut interval = FeedbackInterval::default();

        // 首先设置时间间隔
        interval.set_time_based(12).unwrap();
        assert!(matches!(interval.interval_type, IntervalType::Time { hours: 12 }));

        // 然后设置数量间隔，应该覆盖时间间隔
        interval.set_number_based(100).unwrap();
        assert!(matches!(interval.interval_type, IntervalType::Number { count: 100 }));

        // 再次设置时间间隔，应该覆盖数量间隔
        interval.set_time_based(6).unwrap();
        assert!(matches!(interval.interval_type, IntervalType::Time { hours: 6 }));
    }
}
