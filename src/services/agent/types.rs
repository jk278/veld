//! Agent types
//! 代理相关类型定义

use crate::services::ai_client::AiError;
use serde_json::Value;

/// Agent step for progressive rendering
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentStep {
  /// AI is thinking (short message, optional detailed content)
  Thinking {
    short: String,
    content: Option<String>,
  },
  /// Connecting to MCP server
  Connecting(String),
  /// Calling a tool
  ToolCall {
    name: String,
    args: Value,
  },
  /// Tool execution result
  ToolResult { name: String, result: String },
  /// Streaming content chunk
  Chunk(String),
  /// Final answer (sent after stream completes)
  Final,
}

/// MCP agent error type
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
  #[error("MCP client error: {0}")]
  McpClient(String),
  #[error("AI error: {0}")]
  Ai(String),
  #[error("No tools available")]
  NoToolsAvailable,
  #[error("Tool parse error: {0}")]
  ToolParse(String),
}

impl From<AiError> for AgentError {
  fn from(e: AiError) -> Self {
    AgentError::Ai(e.to_string())
  }
}

pub type Result<T> = std::result::Result<T, AgentError>;

/// Tool call request from AI
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolCall {
  pub name: String,
  pub arguments: Value,
}
