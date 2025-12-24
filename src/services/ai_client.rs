//! AI Client Service
//! AI 客户端服务，支持 Anthropic Compatible API

use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;

/// AI client error type
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("No active provider configured")]
    NoActiveProvider,
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("API key not configured for provider: {0}")]
    ApiKeyMissing(String),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = StdResult<T, AiError>;

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// AI Client for making Anthropic-compatible API requests
pub struct AiClient;

impl AiClient {
    /// Get the active provider configuration
    fn get_active_provider_config() -> Result<(String, String, String, String)> {
        let config = AppConfig::load().map_err(|e| AiError::Http(e.to_string()))?;

        let active_id = config
            .ai
            .active_provider
            .ok_or(AiError::NoActiveProvider)?;

        let provider = config
            .ai
            .providers
            .into_iter()
            .find(|p| p.id == active_id)
            .ok_or_else(|| AiError::ProviderNotFound(active_id.clone()))?;

        let api_key = provider
            .api_key
            .ok_or_else(|| AiError::ApiKeyMissing(active_id.clone()))?;

        let base_url = provider.base_url.ok_or_else(|| {
            AiError::Api(format!("Base URL not configured for {}", active_id))
        })?;

        let model = provider
            .model
            .ok_or_else(|| AiError::Api(format!("Model not configured for {}", active_id)))?;

        Ok((active_id, api_key, base_url, model))
    }

    /// Send a chat completion request using Anthropic Messages API format
    /// All configured providers must be Anthropic-compatible
    pub async fn chat_completion(messages: Vec<ChatMessage>) -> Result<String> {
        let (_provider_id, api_key, base_url, model) = Self::get_active_provider_config()?;

        // All providers use Anthropic Messages API format
        // Base URL should already point to the correct endpoint (e.g., https://api.anthropic.com/v1/messages or https://api.kimi.com/coding/v1/messages)
        let url = if base_url.ends_with("/messages") || base_url.contains("/v1/messages") {
            base_url
        } else if base_url.ends_with("/anthropic") {
            format!("{}/v1/messages", base_url)
        } else if base_url.ends_with("/coding") {
            format!("{}/v1/messages", base_url)
        } else {
            format!("{}/v1/messages", base_url)
        };

        // Extract system message (if any) and filter messages to only user/assistant
        let system_message = messages.iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        let filtered_messages: Vec<_> = messages.into_iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content
            }))
            .collect();

        // Build request body (with optional system parameter)
        let mut request_body_json = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "messages": filtered_messages,
        });

        // Add system parameter if exists
        if let Some(system) = system_message {
            if let Some(obj) = request_body_json.as_object_mut() {
                obj.insert("system".to_string(), serde_json::Value::String(system));
            }
        }

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body_json)
            .send()
            .await
            .map_err(|e: reqwest::Error| AiError::Http(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e: reqwest::Error| AiError::Http(e.to_string()))?;

        if !status.is_success() {
            return Err(AiError::Api(format!(
                "API returned {}: {}",
                status.as_u16(),
                body
            )));
        }

        // Parse Anthropic Messages API response
        Self::parse_anthropic_response(&body)
    }

    /// Parse Anthropic Messages API response
    fn parse_anthropic_response(body: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct ClaudeResponse {
            content: Vec<ClaudeContent>,
        }
        #[derive(Deserialize)]
        struct ClaudeContent {
            text: String,
        }
        let resp: ClaudeResponse =
            serde_json::from_str(body).map_err(|e: serde_json::Error| {
                AiError::Serialization(format!("Failed to parse response: {}", e))
            })?;
        Ok(resp
            .content
            .get(0)
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }
}

/// Create a simple user message
pub fn user_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content,
    }
}

/// Create a system message
pub fn system_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content,
    }
}

/// Create an assistant message
pub fn assistant_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_messages() {
        let user_msg = user_message("Hello".to_string());
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "Hello");

        let sys_msg = system_message("You are a helpful assistant".to_string());
        assert_eq!(sys_msg.role, "system");

        let asst_msg = assistant_message("Hi there!".to_string());
        assert_eq!(asst_msg.role, "assistant");
    }
}
