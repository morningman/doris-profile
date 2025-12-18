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
        // 构建上下文
        let context = ContextBuilder::build_context(
            node,
            profile,
            &self.config.ai_diagnosis.prompt.include_context,
        );
        
        // 调用 AI
        if let Some(ref client) = self.client {
            client.get_suggestion(&context, &self.config.ai_diagnosis.prompt).await
        } else {
            Err("AI diagnosis not enabled".into())
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.config.ai_diagnosis.enabled && self.client.is_some()
    }
}

