// API密钥管理模块 - 处理API密钥的持久化存储
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub gemini_api_key: String,
    pub perplexity_api_key: String,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            gemini_api_key: String::new(),
            perplexity_api_key: String::new(),
        }
    }
}

pub struct ApiKeyManager {
    config_file: String,
}

impl ApiKeyManager {
    pub fn new(config_file: &str) -> Self {
        Self {
            config_file: config_file.to_string(),
        }
    }

    /// 从配置文件加载API密钥
    pub fn load_api_keys(&self) -> Result<ApiKeyConfig> {
        let config_path = Path::new(&self.config_file);

        if !config_path.exists() {
            return Ok(ApiKeyConfig::default());
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| anyhow!("无法读取配置文件: {}", e))?;

        let mut api_config = ApiKeyConfig::default();

        // 解析TOML配置文件
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("gemini_api_key") {
                if let Some(value) = self.extract_toml_value(line) {
                    api_config.gemini_api_key = value;
                }
            } else if line.starts_with("perplexity_api_key") {
                if let Some(value) = self.extract_toml_value(line) {
                    api_config.perplexity_api_key = value;
                }
            }
        }

        Ok(api_config)
    }

    /// 保存API密钥到配置文件
    pub fn save_api_keys(&self, api_config: &ApiKeyConfig) -> Result<()> {
        let config_path = Path::new(&self.config_file);

        if !config_path.exists() {
            return Err(anyhow!("配置文件不存在: {}", self.config_file));
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| anyhow!("无法读取配置文件: {}", e))?;

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut in_ai_section = false;
        let mut gemini_key_updated = false;
        let mut perplexity_key_updated = false;

        for line in &mut lines {
            let trimmed = line.trim();

            // 检测是否在[ai_feedback]段落
            if trimmed == "[ai_feedback]" {
                in_ai_section = true;
                continue;
            } else if trimmed.starts_with("[") && trimmed != "[ai_feedback]" {
                in_ai_section = false;
                continue;
            }

            if in_ai_section {
                if trimmed.starts_with("gemini_api_key") {
                    *line = format!("gemini_api_key = \"{}\"", api_config.gemini_api_key);
                    gemini_key_updated = true;
                } else if trimmed.starts_with("perplexity_api_key") {
                    *line = format!("perplexity_api_key = \"{}\"", api_config.perplexity_api_key);
                    perplexity_key_updated = true;
                }
            }
        }

        // 如果没有找到对应的配置行，添加到[ai_feedback]段落
        if !gemini_key_updated || !perplexity_key_updated {
            let mut ai_section_index = None;
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[ai_feedback]" {
                    ai_section_index = Some(i);
                    break;
                }
            }

            if let Some(section_idx) = ai_section_index {
                let mut insert_idx = section_idx + 1;

                // 找到下一个段落的开始位置
                while insert_idx < lines.len() && !lines[insert_idx].trim().starts_with("[") {
                    insert_idx += 1;
                }

                if !gemini_key_updated {
                    lines.insert(insert_idx, format!("gemini_api_key = \"{}\"", api_config.gemini_api_key));
                    insert_idx += 1;
                }

                if !perplexity_key_updated {
                    lines.insert(insert_idx, format!("perplexity_api_key = \"{}\"", api_config.perplexity_api_key));
                }
            }
        }

        let updated_content = lines.join("\n");
        fs::write(config_path, updated_content)
            .map_err(|e| anyhow!("无法写入配置文件: {}", e))?;

        println!("✅ API密钥已保存到配置文件");
        Ok(())
    }

    /// 验证API密钥是否有效（基本格式检查）
    pub fn validate_api_key(&self, provider: &str, api_key: &str) -> Result<()> {
        if api_key.is_empty() {
            return Err(anyhow!("API密钥不能为空"));
        }

        match provider.to_lowercase().as_str() {
            "gemini" => {
                if !api_key.starts_with("AIzaSy") {
                    return Err(anyhow!("Gemini API密钥格式不正确，应以'AIzaSy'开头"));
                }
                if api_key.len() < 30 {
                    return Err(anyhow!("Gemini API密钥长度不正确"));
                }
            }
            "perplexity" => {
                if !api_key.starts_with("pplx-") {
                    return Err(anyhow!("Perplexity API密钥格式不正确，应以'pplx-'开头"));
                }
                if api_key.len() < 40 {
                    return Err(anyhow!("Perplexity API密钥长度不正确"));
                }
            }
            _ => {
                return Err(anyhow!("不支持的LLM提供商: {}", provider));
            }
        }

        Ok(())
    }

    /// 检查是否至少有一个有效的API密钥
    pub fn has_valid_api_key(&self, api_config: &ApiKeyConfig) -> bool {
        (!api_config.gemini_api_key.is_empty() && self.validate_api_key("gemini", &api_config.gemini_api_key).is_ok()) ||
        (!api_config.perplexity_api_key.is_empty() && self.validate_api_key("perplexity", &api_config.perplexity_api_key).is_ok())
    }

    /// 显示当前API密钥状态
    pub fn show_api_status(&self, api_config: &ApiKeyConfig) {
        println!("🔑 当前API密钥状态:");

        if api_config.gemini_api_key.is_empty() {
            println!("  📝 Gemini: 未配置");
        } else {
            match self.validate_api_key("gemini", &api_config.gemini_api_key) {
                Ok(_) => println!("  ✅ Gemini: 已配置 ({}****)", &api_config.gemini_api_key[..10]),
                Err(_) => println!("  ❌ Gemini: 格式错误"),
            }
        }

        if api_config.perplexity_api_key.is_empty() {
            println!("  📝 Perplexity: 未配置");
        } else {
            match self.validate_api_key("perplexity", &api_config.perplexity_api_key) {
                Ok(_) => println!("  ✅ Perplexity: 已配置 ({}****)", &api_config.perplexity_api_key[..10]),
                Err(_) => println!("  ❌ Perplexity: 格式错误"),
            }
        }
    }

    /// 从TOML行中提取值
    fn extract_toml_value(&self, line: &str) -> Option<String> {
        if let Some(equals_pos) = line.find('=') {
            let value_part = line[equals_pos + 1..].trim();
            if value_part.starts_with('"') && value_part.ends_with('"') && value_part.len() > 1 {
                return Some(value_part[1..value_part.len()-1].to_string());
            }
        }
        None
    }
}