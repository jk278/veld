//! AI Client Service
//! 使用 rig-core 统一接口支持 OpenAI & Anthropic 协议

use crate::config::AppConfig;
use rig::client::{Client, CompletionClient};
use rig::completion::{Chat, Message, Prompt};
use rig::providers::anthropic;
use rig::providers::openai;
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;
use tokio::sync::mpsc;

/// AI client error type
#[derive(Debug, thiserror::Error)]
pub enum AiError {
  #[error("No active provider configured")]
  NoActiveProvider,
  #[error("Provider not found: {0}")]
  ProviderNotFound(String),
  #[error("API key not configured for provider: {0}")]
  ApiKeyMissing(String),
  #[error("rig error: {0}")]
  Rig(String),
  #[error("Config error: {0}")]
  Config(String),
  #[error("Unsupported adapter type: {0}")]
  UnsupportedAdapter(String),
  #[error("HTTP client error: {0}")]
  HttpClient(String),
}

pub type Result<T> = StdResult<T, AiError>;

/// Chat message (兼容原有格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  pub role: String,
  pub content: String,
}

/// Provider client wrapper - 存储不同类型的 Client
enum ProviderClient {
  OpenAI {
    client: Client<openai::OpenAIResponsesExt>,
    model: String,
  },
  Anthropic {
    client: anthropic::Client,
    model: String,
  },
}

impl Clone for ProviderClient {
  fn clone(&self) -> Self {
    match self {
      ProviderClient::OpenAI { client, model } => ProviderClient::OpenAI {
        client: client.clone(),
        model: model.clone(),
      },
      ProviderClient::Anthropic { client, model } => ProviderClient::Anthropic {
        client: client.clone(),
        model: model.clone(),
      },
    }
  }
}

/// AI Client (使用 rig-core 统一接口)
/// 每次请求时动态创建 provider client，确保使用最新配置
pub struct AiClient;

impl AiClient {
  /// 创建新的 AI 客户端
  pub fn new() -> Result<Self> {
    Ok(Self)
  }

  /// 转换 ChatMessage 到 rig Message
  fn to_rig_message(msg: ChatMessage) -> Message {
    match msg.role.as_str() {
      "system" => Message::user(msg.content),  // System messages are handled as preamble
      "user" => Message::user(msg.content),
      "assistant" => Message::assistant(msg.content),
      _ => Message::user(msg.content),
    }
  }

  /// 获取当前激活的 provider client
  /// 每次调用时重新读取配置，并动态创建对应的 client
  fn get_active_client(&self) -> Result<ProviderClient> {
    // 重新加载配置以获取最新的 active_id 和 providers
    let config = AppConfig::load().map_err(|e| AiError::Config(e.to_string()))?;
    let active_id = config
      .ai
      .active_provider
      .as_ref()
      .ok_or(AiError::NoActiveProvider)?;

    // 从配置中找到激活的 provider（使用最新的 enabled 状态）
    let provider = config
      .ai
      .providers
      .iter()
      .find(|p| p.id == *active_id)
      .ok_or_else(|| AiError::ProviderNotFound(active_id.clone()))?;

    // 检查 provider 是否启用且有 API key
    if !provider.enabled {
      return Err(AiError::ProviderNotFound(format!(
        "Provider '{}' is not enabled",
        active_id
      )));
    }

    let api_key = provider
      .api_key
      .as_ref()
      .filter(|k| !k.is_empty())
      .ok_or_else(|| AiError::ApiKeyMissing(provider.id.clone()))?;

    // 动态创建 client
    let base_url = provider.base_url.as_deref();
    let model = provider.model.clone();

    let client = match provider.adapter_type.as_deref() {
      Some("openai") => {
        let builder = openai::Client::builder();
        let builder = if let Some(url) = base_url {
          builder.api_key(api_key).base_url(url)
        } else {
          builder.api_key(api_key)
        };
        let client = builder
          .build()
          .map_err(|e| AiError::HttpClient(e.to_string()))?;
        ProviderClient::OpenAI {
          client,
          model,
        }
      }
      Some("anthropic") => {
        let builder = anthropic::Client::builder();
        let builder = if let Some(url) = base_url {
          builder.api_key(api_key).base_url(url)
        } else {
          builder.api_key(api_key)
        };
        let client = builder
          .build()
          .map_err(|e| AiError::HttpClient(e.to_string()))?;
        ProviderClient::Anthropic {
          client,
          model,
        }
      }
      Some(other) => {
        return Err(AiError::UnsupportedAdapter(other.to_string()));
      }
      None => {
        return Err(AiError::UnsupportedAdapter("none".to_string()));
      }
    };

    Ok(client)
  }

  /// 聊天响应 - 支持完整对话历史（用于工具链式调用）
  /// 使用 StreamingPromptRequest API 实现真正的流式响应
  pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<Result<String>>> {
    let client = self.get_active_client()?;
    let (prompt, rig_messages, preamble) = Self::parse_messages(messages)?;
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
      match client {
        ProviderClient::OpenAI { client, model } => {
          // Build agent with preamble
          let agent = if let Some(pre) = preamble {
            client.agent(&model).preamble(&pre).build()
          } else {
            client.agent(&model).build()
          };

          // Use simple Chat trait to get complete response
          // This preserves tool chain calling support through chat_with_tools
          let response = agent.chat(&prompt, rig_messages).await;
          match response {
            Ok(text) => {
              let _ = tx.send(Ok(text)).await;
            }
            Err(e) => {
              let _ = tx.send(Err(AiError::Rig(e.to_string()))).await;
            }
          }
        }
        ProviderClient::Anthropic { client, model } => {
          // Anthropic requires max_tokens to be set
          let agent = if let Some(pre) = preamble {
            client
              .agent(&model)
              .preamble(&pre)
              .max_tokens(4096)
              .build()
          } else {
            client.agent(&model).max_tokens(4096).build()
          };

          // Use simple Chat trait to get complete response
          let response = agent.chat(&prompt, rig_messages).await;
          match response {
            Ok(text) => {
              let _ = tx.send(Ok(text)).await;
            }
            Err(e) => {
              let _ = tx.send(Err(AiError::Rig(e.to_string()))).await;
            }
          }
        }
      }
    });

    Ok(rx)
  }

  /// 解析消息列表，转换为 rig Message 格式
  /// 返回 (prompt, rig_messages, system_preamble)
  /// prompt: 最后一条用户消息内容
  /// rig_messages: 历史消息（不包含最后一条用户消息）
  /// system_preamble: 系统提示词
  fn parse_messages(messages: Vec<ChatMessage>) -> Result<(String, Vec<Message>, Option<String>)> {
    let mut rig_messages = Vec::new();
    let mut system_content = String::new();
    let mut last_user_content = String::new();

    // 分离最后一条用户消息作为 prompt
    let mut last_user_idx = None;
    for (idx, msg) in messages.iter().enumerate() {
      if msg.role == "user" {
        last_user_idx = Some(idx);
      }
    }

    for (idx, msg) in messages.into_iter().enumerate() {
      match msg.role.as_str() {
        "system" => {
          system_content.push_str(&msg.content);
          system_content.push('\n');
        }
        "user" => {
          if Some(idx) == last_user_idx {
            // 这是最后一条用户消息，作为 prompt
            last_user_content = msg.content;
          } else {
            // 其他用户消息加入历史
            rig_messages.push(Self::to_rig_message(msg));
          }
        }
        "assistant" => {
          rig_messages.push(Self::to_rig_message(msg));
        }
        _ => {
          rig_messages.push(Self::to_rig_message(msg));
        }
      }
    }

    let preamble = if system_content.is_empty() {
      None
    } else {
      Some(system_content.trim().to_string())
    };

    Ok((last_user_content, rig_messages, preamble))
  }

  /// 收集完整响应（辅助函数）
  pub async fn collect(rx: &mut mpsc::Receiver<Result<String>>) -> Result<String> {
    let mut result = String::new();
    while let Some(chunk) = rx.recv().await {
      result.push_str(&chunk?);
    }
    Ok(result)
  }

  /// 简单聊天（无工具，流式响应）
  pub async fn chat_simple(
    &self,
    prompt: &str,
    preamble: Option<&str>,
    tx: mpsc::Sender<Result<String>>,
  ) -> Result<()> {
    let client = self.get_active_client()?;
    let prompt_owned = prompt.to_string();
    let preamble_str = preamble.unwrap_or("").to_string();

    tokio::spawn(async move {
      let result: std::result::Result<(), AiError> = match client {
        ProviderClient::OpenAI { client, model } => {
          let agent = if !preamble_str.is_empty() {
            client.agent(&model).preamble(&preamble_str).build()
          } else {
            client.agent(&model).build()
          };

          match agent.prompt(&prompt_owned).await {
            Ok(response) => {
              let _ = tx.send(Ok(response)).await;
            }
            Err(e) => {
              let _ = tx.send(Err(AiError::Rig(e.to_string()))).await;
            }
          }
          Ok(())
        }
        ProviderClient::Anthropic { client, model } => {
          let agent = if !preamble_str.is_empty() {
            client
              .agent(&model)
              .preamble(&preamble_str)
              .max_tokens(4096)
              .build()
          } else {
            client.agent(&model).max_tokens(4096).build()
          };

          match agent.prompt(&prompt_owned).await {
            Ok(response) => {
              let _ = tx.send(Ok(response)).await;
            }
            Err(e) => {
              let _ = tx.send(Err(AiError::Rig(e.to_string()))).await;
            }
          }
          Ok(())
        }
      };

      let _ = result;
    });

    Ok(())
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
  fn test_parse_messages_simple() {
    let messages = vec![
      system_message("You are helpful".to_string()),
      user_message("Hello".to_string()),
    ];

    let (prompt, history, preamble) = AiClient::parse_messages(messages).unwrap();
    assert_eq!(prompt, "Hello");
    assert!(history.is_empty());
    assert!(preamble.is_some());
    assert_eq!(preamble.unwrap(), "You are helpful");
  }

  #[test]
  fn test_parse_messages_with_history() {
    let messages = vec![
      system_message("You are helpful".to_string()),
      user_message("Hello".to_string()),
      assistant_message("Hi!".to_string()),
      user_message("How are you?".to_string()),
    ];

    let (prompt, history, preamble) = AiClient::parse_messages(messages).unwrap();
    assert_eq!(prompt, "How are you?");
    assert_eq!(history.len(), 2); // "Hello" user message + "Hi!" assistant message
    assert!(preamble.is_some());
  }

  #[test]
  fn test_parse_messages_with_system_only() {
    let messages = vec![
      system_message("You are helpful".to_string()),
      user_message("Hello".to_string()),
    ];

    let (prompt, history, preamble) = AiClient::parse_messages(messages).unwrap();
    assert_eq!(prompt, "Hello");
    assert!(history.is_empty());
    assert_eq!(preamble.unwrap(), "You are helpful");
  }
}
