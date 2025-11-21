// Cloudflare 自动上传模块
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Local};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptBatch {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub prompts: Vec<ChinesePrompt>,
    pub total_count: usize,
    pub chinese_char_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChinesePrompt {
    pub id: String,
    pub timestamp: DateTime<Local>,
    pub content: String,
    pub chinese_chars: usize,
    pub word_count: usize,
}

#[derive(Debug, Clone)]
pub struct CloudflareConfig {
    pub account_id: String,
    pub zone_id: String,
    pub api_token: String,
    pub domain: String,
    pub upload_endpoint: String,
    pub github_repo: String,
}

pub struct CloudflareUploader {
    config: CloudflareConfig,
    client: Client,
}

impl CloudflareUploader {
    pub fn new(config: CloudflareConfig) -> Self {
        let client = Client::new();
        Self { config, client }
    }

    /// 创建提示词批次
    pub fn create_batch(&self, prompts: Vec<String>) -> PromptBatch {
        let chinese_prompts: Vec<ChinesePrompt> = prompts
            .into_iter()
            .map(|content| {
                let chinese_chars = content.chars()
                    .filter(|c| {
                        let code = *c as u32;
                        (code >= 0x4e00 && code <= 0x9fff) || // CJK统一表意文字
                        (code >= 0x3400 && code <= 0x4dbf) || // CJK扩展A
                        (code >= 0xff00 && code <= 0xffef)    // 全角字符
                    })
                    .count();

                ChinesePrompt {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Local::now(),
                    content: content.clone(),
                    chinese_chars,
                    word_count: content.chars().count(),
                }
            })
            .collect();

        let total_chinese_chars: usize = chinese_prompts.iter()
            .map(|p| p.chinese_chars)
            .sum();

        PromptBatch {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now(),
            prompts: chinese_prompts.clone(),
            total_count: chinese_prompts.len(),
            chinese_char_count: total_chinese_chars,
        }
    }

    /// 上传到Cloudflare Pages Functions
    pub async fn upload_via_pages_function(&self, batch: &PromptBatch) -> anyhow::Result<()> {
        let url = format!("https://{}{}", self.config.domain, self.config.upload_endpoint);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(batch)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ 成功上传批次: {} ({}条提示词)", batch.id, batch.total_count);
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("上传失败: {}", error_text);
        }

        Ok(())
    }

    /// 通过Cloudflare KV存储上传
    pub async fn upload_via_kv_storage(&self, batch: &PromptBatch) -> anyhow::Result<()> {
        // KV存储键名：prompts:YYYY-MM-DD:HH
        let key = format!("prompts:{}:{}",
            batch.timestamp.format("%Y-%m-%d"),
            batch.timestamp.format("%H")
        );

        let kv_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.config.account_id,
            "YOUR_KV_NAMESPACE_ID", // 需要替换为实际的KV命名空间ID
            key
        );

        let response = self.client
            .put(&kv_url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .header("Content-Type", "application/json")
            .json(batch)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ 成功存储到KV: {} ({}条提示词)", key, batch.total_count);
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("KV存储失败: {}", error_text);
        }

        Ok(())
    }

    /// 触发GitHub Pages重新部署
    pub async fn trigger_github_deployment(&self) -> anyhow::Result<()> {
        let url = format!("https://api.github.com/repos/{}/dispatches", self.config.github_repo);

        let payload = serde_json::json!({
            "event_type": "prompter_update",
            "client_payload": {
                "timestamp": Local::now().to_rfc3339(),
                "source": "prompter_auto_upload"
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("token {}", self.config.api_token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "prompter/1.0")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ 成功触发GitHub部署");
        } else {
            let error_text = response.text().await?;
            println!("⚠️  GitHub部署触发失败: {}", error_text);
        }

        Ok(())
    }

    /// 直接推送内容到GitHub仓库
    pub async fn push_to_github(&self, batch: &PromptBatch) -> anyhow::Result<()> {
        println!("📤 推送内容到GitHub仓库: {}", self.config.github_repo);

        // 生成文件名
        let filename = format!("prompts_{}.json", batch.timestamp.format("%Y%m%d_%H%M%S"));
        let file_path = format!("data/{}", filename);

        // 准备文件内容
        let content = serde_json::to_string_pretty(batch)?;
        let encoded_content = general_purpose::STANDARD.encode(&content);

        // 检查文件是否已存在
        let check_url = format!(
            "https://api.github.com/repos/{}/contents/{}",
            self.config.github_repo, file_path
        );

        let check_response = self.client
            .get(&check_url)
            .header("Authorization", format!("token {}", self.config.api_token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "prompter/1.0")
            .send()
            .await?;

        let mut payload = serde_json::json!({
            "message": format!("📝 新增中文提示词批次 {} ({}条)", batch.id, batch.total_count),
            "content": encoded_content,
            "branch": "main"
        });

        // 如果文件已存在，需要提供SHA值
        if check_response.status().is_success() {
            let file_info: serde_json::Value = check_response.json().await?;
            if let Some(sha) = file_info.get("sha") {
                payload["sha"] = sha.clone();
                payload["message"] = serde_json::json!(
                    format!("📝 更新中文提示词批次 {} ({}条)", batch.id, batch.total_count)
                );
            }
        }

        // 推送文件
        let push_url = format!(
            "https://api.github.com/repos/{}/contents/{}",
            self.config.github_repo, file_path
        );

        let response = self.client
            .put(&push_url)
            .header("Authorization", format!("token {}", self.config.api_token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "prompter/1.0")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ 成功推送到GitHub: {}", file_path);
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("GitHub推送失败: {}", error_text);
        }

        Ok(())
    }

    /// 完整的上传流程
    pub async fn upload_batch(&self, prompts: Vec<String>) -> anyhow::Result<()> {
        if prompts.is_empty() {
            println!("ℹ️  没有中文提示词需要上传");
            return Ok(());
        }

        let batch = self.create_batch(prompts);

        println!("📤 开始上传批次: {} ({}条提示词, {}个中文字符)",
            batch.id, batch.total_count, batch.chinese_char_count);

        // 方案1: 上传到Pages Function
        if let Err(e) = self.upload_via_pages_function(&batch).await {
            println!("❌ Pages Function上传失败: {}", e);

            // 备用方案: KV存储
            println!("🔄 尝试KV存储...");
            self.upload_via_kv_storage(&batch).await?;
        }

        // 方案2: 直接推送到GitHub (可选)
        if let Err(e) = self.push_to_github(&batch).await {
            println!("⚠️  GitHub推送失败: {}", e);
        }

        // 触发网站重新部署
        self.trigger_github_deployment().await?;

        Ok(())
    }

    /// 测试连接
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        let url = format!("https://api.cloudflare.com/client/v4/user");

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_token))
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Cloudflare API连接正常");
            Ok(())
        } else {
            anyhow::bail!("Cloudflare API连接失败: {}", response.status());
        }
    }
}

/// 辅助函数：生成示例的Cloudflare Pages Function代码
pub fn generate_pages_function_code() -> String {
    r#"
// /functions/api/prompts.js
// Cloudflare Pages Function - 接收提示词上传

export async function onRequestPost(context) {
  try {
    const batch = await context.request.json();

    // 验证数据格式
    if (!batch.prompts || !Array.isArray(batch.prompts)) {
      return new Response('Invalid batch format', { status: 400 });
    }

    // 存储到KV
    const key = `prompts:${batch.timestamp}:${batch.id}`;
    await context.env.PROMPTER_KV.put(key, JSON.stringify(batch));

    // 更新索引
    const indexKey = 'prompt_index';
    const existingIndex = await context.env.PROMPTER_KV.get(indexKey, 'json') || [];
    existingIndex.push({
      id: batch.id,
      timestamp: batch.timestamp,
      count: batch.total_count,
      chinese_chars: batch.chinese_char_count
    });

    // 保持最近100个批次
    const recentIndex = existingIndex.slice(-100);
    await context.env.PROMPTER_KV.put(indexKey, JSON.stringify(recentIndex));

    return new Response(JSON.stringify({
      success: true,
      batch_id: batch.id,
      stored_prompts: batch.total_count
    }), {
      headers: { 'Content-Type': 'application/json' }
    });

  } catch (error) {
    return new Response(`Error: ${error.message}`, { status: 500 });
  }
}
"#.to_string()
}