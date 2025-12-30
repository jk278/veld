//! AI Client Service
//! 使用 genai 统一接口支持 OpenAI & Anthropic 协议

use crate::config::AppConfig;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage as GenaiChatMessage, ChatRequest};
use genai::resolver::{
  AuthData, Endpoint, Error as ResolverError, ModelMapper, ServiceTargetResolver,
};
use genai::{Client, ModelIden, ServiceTarget};
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use std::sync::Arc;

/// AI client error type
#[derive(Debug, thiserror::Error)]
pub enum AiError {
  #[error("No active provider configured")]
  NoActiveProvider,
  #[error("Provider not found: {0}")]
  ProviderNotFound(String),
  #[error("API key not configured for provider: {0}")]
  ApiKeyMissing(String),
  #[error("genai error: {0}")]
  Genai(String),
  #[error("Config error: {0}")]
  Config(String),
}

pub type Result<T> = StdResult<T, AiError>;

impl From<genai::Error> for AiError {
  fn from(e: genai::Error) -> Self {
    AiError::Genai(e.to_string())
  }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  pub role: String,
  pub content: String,
}

/// 规范化自定义 API 端点 URL
///
/// # 规则
/// - 已含完整路径（`/completions` 或 `/messages`）→ 不补全
/// - 含版本号结尾（如 `/v1`, `/v2`）→ 补全对应路径
/// - 否则统一补全 `/v1/...`
fn normalize_endpoint_url(url: &str, adapter: AdapterKind) -> String {
  let url = url.trim().trim_end_matches('/');

  let is_complete_openai = url.ends_with("/completions");
  let is_complete_anthropic = url.ends_with("/messages");

  // 检测是否以版本号结尾 (/v1, /v2, /v4 等)
  let has_version_suffix = regex::Regex::new(r"/v\d+$").unwrap().is_match(url);

  match adapter {
    AdapterKind::OpenAI if !is_complete_openai => {
      if has_version_suffix {
        format!("{}/chat/completions", url)
      } else {
        format!("{}/v1/chat/completions", url)
      }
    }
    AdapterKind::Anthropic if !is_complete_anthropic => {
      if has_version_suffix {
        format!("{}/messages", url)
      } else {
        format!("{}/v1/messages", url)
      }
    }
    _ => url.to_string(),
  }
}

/// AI Client (使用 genai 统一接口)
pub struct AiClient {
  client: Arc<Client>,
}

impl AiClient {
  /// 创建新的 AI 客户端
  pub fn new() -> Result<Self> {
    let config = AppConfig::load().map_err(|e| AiError::Config(e.to_string()))?;
    let model_mapper = Self::create_model_mapper(&config)?;

    // 创建 ServiceTargetResolver 来支持自定义 base_url 和 auth
    let mut adapter_map: std::collections::HashMap<
      String,
      (AdapterKind, Option<String>, Option<String>),
    > = std::collections::HashMap::new();
    for provider in &config.ai.providers {
      if provider.enabled {
        let adapter = match provider.adapter_type.as_deref() {
          Some("openai") => AdapterKind::OpenAI,
          Some("anthropic") => AdapterKind::Anthropic,
          _ => continue,
        };
        adapter_map.insert(
          provider.model.clone(),
          (adapter, provider.base_url.clone(), provider.api_key.clone()),
        );
      }
    }

    let target_resolver = ServiceTargetResolver::from_resolver_fn(
      move |service_target: ServiceTarget| -> StdResult<ServiceTarget, ResolverError> {
        let model_name: &str = service_target.model.model_name.as_ref();

        if let Some((adapter_kind, base_url, api_key)) = adapter_map.get(model_name) {
          // 自定义 endpoint - 使用规范化函数兼容不同 URL 格式
          let endpoint = base_url
            .as_ref()
            .filter(|u| !u.is_empty())
            .map(|url| Endpoint::from_owned(normalize_endpoint_url(url, *adapter_kind)))
            .unwrap_or_else(|| service_target.endpoint.clone());

          // API key
          let auth = api_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .map(|key| AuthData::from_single(key.clone()))
            .unwrap_or_else(|| service_target.auth.clone());

          let model = ModelIden::new(*adapter_kind, model_name);
          Ok(ServiceTarget {
            endpoint,
            auth,
            model,
          })
        } else {
          // 没有找到配置，使用原值
          Ok(service_target)
        }
      },
    );

    Ok(Self {
      client: Arc::new(
        Client::builder()
          .with_service_target_resolver(target_resolver)
          .with_model_mapper(model_mapper)
          .build(),
      ),
    })
  }

  /// 创建模型映射器 - 根据配置中的 adapter_type 映射到对应适配器
  fn create_model_mapper(config: &AppConfig) -> Result<ModelMapper> {
    // 创建模型名到适配器类型的映射
    let mut adapter_map: std::collections::HashMap<String, AdapterKind> =
      std::collections::HashMap::new();

    for provider in &config.ai.providers {
      if let Some(ref adapter_type) = provider.adapter_type {
        let adapter = match adapter_type.as_str() {
          "openai" => AdapterKind::OpenAI,
          "anthropic" => AdapterKind::Anthropic,
          _ => continue, // Unknown adapter type, skip
        };
        adapter_map.insert(provider.model.clone(), adapter);
      }
    }

    Ok(ModelMapper::from_mapper_fn(move |model_iden: ModelIden| {
      let model_name: &str = model_iden.model_name.as_ref();

      // 如果配置中指定了适配器类型，使用配置的
      if let Some(&adapter) = adapter_map.get(model_name) {
        return Ok(ModelIden::new(adapter, model_name));
      }

      // 否则使用 genai 默认自动检测
      Ok(model_iden)
    }))
  }

  /// 获取当前激活的模型
  fn get_active_model(config: &AppConfig) -> Result<String> {
    let active_id = config
      .ai
      .active_provider
      .as_ref()
      .ok_or(AiError::NoActiveProvider)?;
    let provider = config
      .ai
      .providers
      .iter()
      .find(|p| p.id == *active_id)
      .ok_or_else(|| AiError::ProviderNotFound(active_id.clone()))?;

    Ok(provider.model.clone())
  }

  /// 转换消息格式
  fn to_genai_request(messages: Vec<ChatMessage>) -> Result<ChatRequest> {
    let genai_messages = messages
      .into_iter()
      .map(|m| match m.role.as_str() {
        "system" => GenaiChatMessage::system(m.content),
        "user" => GenaiChatMessage::user(m.content),
        "assistant" => GenaiChatMessage::assistant(m.content),
        _ => GenaiChatMessage::user(m.content),
      })
      .collect();

    Ok(ChatRequest::new(genai_messages))
  }

  /// 发送聊天请求（非流式）
  pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<String> {
    let config = AppConfig::load().map_err(|e| AiError::Config(e.to_string()))?;
    let model = Self::get_active_model(&config)?;

    let chat_req = Self::to_genai_request(messages)?;
    let response = self.client.exec_chat(&model, chat_req, None).await?;

    Ok(response.first_text().unwrap_or_default().to_string())
  }

  /// 发送聊天请求（流式）- 返回完整响应
  pub async fn chat_completion_stream(&self, messages: Vec<ChatMessage>) -> Result<String> {
    // TODO: 实现流式响应
    // 目前使用非流式实现
    self.chat_completion(messages).await
  }
}

impl Default for AiClient {
  fn default() -> Self {
    Self::new().expect("Failed to create AiClient")
  }
}

/// 创建用户消息
pub fn user_message(content: String) -> ChatMessage {
  ChatMessage {
    role: "user".to_string(),
    content,
  }
}

/// 创建系统消息
pub fn system_message(content: String) -> ChatMessage {
  ChatMessage {
    role: "system".to_string(),
    content,
  }
}

/// 创建助手消息
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

  #[test]
  fn test_to_genai_request() {
    let messages = vec![
      system_message("You are helpful".to_string()),
      user_message("Hello".to_string()),
    ];

    let req = to_genai_request(messages).unwrap();
    assert_eq!(req.messages.len(), 2);
  }
}
