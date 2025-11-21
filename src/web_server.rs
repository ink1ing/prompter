// Web服务器模块 - 提供Web界面启动prompter
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebPrompt {
    pub id: usize,
    pub timestamp: String,
    pub content: String,
    pub saved: bool,
}

#[derive(Clone)]
pub struct WebState {
    pub prompts: Arc<Mutex<VecDeque<WebPrompt>>>,
    pub next_id: Arc<Mutex<usize>>,
}

impl WebState {
    pub fn new() -> Self {
        Self {
            prompts: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn add_prompt(&self, content: String) -> WebPrompt {
        let mut prompts = self.prompts.lock().unwrap();
        let mut id_counter = self.next_id.lock().unwrap();

        let prompt = WebPrompt {
            id: *id_counter,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            content,
            saved: false,
        };

        prompts.push_back(prompt.clone());
        *id_counter += 1;

        // 限制历史记录数量
        if prompts.len() > 100 {
            prompts.pop_front();
        }

        prompt
    }

    pub fn get_prompts(&self) -> Vec<WebPrompt> {
        let prompts = self.prompts.lock().unwrap();
        prompts.iter().cloned().collect()
    }
}

// 这里可以添加Web服务器实现
// 使用warp或axum等框架来提供REST API和静态文件服务