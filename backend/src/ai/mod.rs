mod openai_client;
mod context_builder;

pub use openai_client::OpenAiClient;
pub use context_builder::ContextBuilder;

use crate::config::AiConfig;
use crate::models::*;

pub struct AiDiagnosisService {
    config: AiConfig,
    client: Option<OpenAiClient>,
}

impl AiDiagnosisService {
    pub fn new(config: AiConfig) -> Self {
        let client = if config.ai_diagnosis.enabled {
            Some(OpenAiClient::new(&config.ai_diagnosis.openai))
        } else {
            None
        };
        
        Self { config, client }
    }
    
    pub async fn generate_suggestion(
        &self,
        node: &ExecutionTreeNode,
        profile: &Profile,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.generate_suggestion_with_language(node, profile, "zh").await
    }
    
    pub async fn generate_suggestion_with_language(
        &self,
        node: &ExecutionTreeNode,
        profile: &Profile,
        language: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // 构建上下文
        let context = ContextBuilder::build_context(
            node,
            profile,
            &self.config.ai_diagnosis.prompt.include_context,
        );
        
        // 根据语言调整 system message
        let mut prompt_config = self.config.ai_diagnosis.prompt.clone();
        if language == "zh" || language == "chinese" {
            prompt_config.system_message = "You are an expert Doris database performance analyst. Analyze the provided execution plan node and provide specific, actionable optimization suggestions in Chinese. Focus on practical recommendations based on the node's metrics and context.".to_string();
        } else {
            prompt_config.system_message = "You are an expert Doris database performance analyst. Analyze the provided execution plan node and provide specific, actionable optimization suggestions in English. Focus on practical recommendations based on the node's metrics and context.".to_string();
        }
        
        // 调用 AI
        if let Some(ref client) = self.client {
            client.get_suggestion(&context, &prompt_config).await
        } else {
            Err("AI diagnosis not enabled".into())
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.config.ai_diagnosis.enabled && self.client.is_some()
    }
}

