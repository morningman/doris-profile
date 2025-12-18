use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    pub ai_diagnosis: AiDiagnosisConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiDiagnosisConfig {
    pub enabled: bool,
    pub provider: String,
    pub openai: OpenAiConfig,
    pub prompt: PromptConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub api_endpoint: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptConfig {
    pub system_message: String,
    pub include_context: ContextConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextConfig {
    pub sql_statement: bool,
    pub query_summary: bool,
    pub child_nodes: bool,
    pub max_child_nodes: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultSuggestionsConfig {
    pub suggestions: HashMap<String, SeveritySuggestions>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeveritySuggestions {
    #[serde(default)]
    pub critical: Vec<String>,
    #[serde(default)]
    pub high: Vec<String>,
    #[serde(default)]
    pub medium: Vec<String>,
    #[serde(default)]
    pub low: Vec<String>,
}

pub struct ConfigLoader;

impl ConfigLoader {
    /// Load AI configuration from config/ai_config.yaml
    /// Environment variable OPENAI_API_KEY will override the api_key in config file
    pub fn load_ai_config() -> Result<AiConfig, Box<dyn std::error::Error>> {
        // Try to find config file in multiple locations
        let possible_paths = vec![
            "backend/config/ai_config.yaml",
            "config/ai_config.yaml",
            "./ai_config.yaml",
        ];
        
        let mut config_path = None;
        for path in possible_paths {
            if Path::new(path).exists() {
                config_path = Some(path);
                break;
            }
        }
        
        let config_content = if let Some(path) = config_path {
            fs::read_to_string(path)?
        } else {
            return Err("AI config file not found".into());
        };
        
        let mut config: AiConfig = serde_yaml::from_str(&config_content)?;
        
        // Override API key from environment variable if set
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                config.ai_diagnosis.openai.api_key = api_key;
            }
        }
        
        Ok(config)
    }
    
    /// Load default suggestions configuration from config/default_suggestions.yaml
    pub fn load_default_suggestions() -> Result<DefaultSuggestionsConfig, Box<dyn std::error::Error>> {
        // Try to find config file in multiple locations
        let possible_paths = vec![
            "backend/config/default_suggestions.yaml",
            "config/default_suggestions.yaml",
            "./default_suggestions.yaml",
        ];
        
        let mut config_path = None;
        for path in possible_paths {
            if Path::new(path).exists() {
                config_path = Some(path);
                break;
            }
        }
        
        let config_content = if let Some(path) = config_path {
            fs::read_to_string(path)?
        } else {
            return Err("Default suggestions config file not found".into());
        };
        
        let config: DefaultSuggestionsConfig = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
    
    /// Get a default AI config for cases where loading fails
    pub fn default_ai_config() -> AiConfig {
        AiConfig {
            ai_diagnosis: AiDiagnosisConfig {
                enabled: false,
                provider: "openai".to_string(),
                openai: OpenAiConfig {
                    api_key: String::new(),
                    api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                    model: "gpt-4".to_string(),
                    temperature: 0.7,
                    max_tokens: 1000,
                    timeout_seconds: 30,
                },
                prompt: PromptConfig {
                    system_message: "You are an expert Doris database performance analyst.".to_string(),
                    include_context: ContextConfig {
                        sql_statement: true,
                        query_summary: true,
                        child_nodes: true,
                        max_child_nodes: 3,
                    },
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_ai_config() {
        let config = ConfigLoader::default_ai_config();
        assert_eq!(config.ai_diagnosis.enabled, false);
        assert_eq!(config.ai_diagnosis.provider, "openai");
    }
}

