use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::config::{OpenAiConfig, PromptConfig};

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub struct OpenAiClient {
    config: OpenAiConfig,
    client: Client,
}

impl OpenAiClient {
    pub fn new(config: &OpenAiConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            config: config.clone(),
            client,
        }
    }
    
    pub async fn get_suggestion(
        &self,
        context: &str,
        prompt_config: &PromptConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 检查 API key
        if self.config.api_key.is_empty() {
            return Err("OpenAI API key is not configured".into());
        }
        
        // 构建消息
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: prompt_config.system_message.clone(),
            },
            Message {
                role: "user".to_string(),
                content: format!("请分析以下 Doris 执行计划节点，并提供具体的优化建议：\n\n{}", context),
            },
        ];
        
        // 构建请求
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };
        
        // 调用 OpenAI API
        let response = self.client
            .post(&self.config.api_endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error {}: {}", status, error_text).into());
        }
        
        // 解析响应
        let chat_response: ChatResponse = response.json().await?;
        
        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from OpenAI".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let config = OpenAiConfig {
            api_key: "test".to_string(),
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            timeout_seconds: 30,
        };
        
        let client = OpenAiClient::new(&config);
        assert_eq!(client.config.model, "gpt-4");
    }
}

