//! Agent types - Chain-call architecture
//! 代理类型定义 - 链式调用架构

use crate::services::ai_client::AiError;
use serde_json::Value;

// ============================================================================
// PART 1: Step - Unified execution unit with update support
// ============================================================================

/// A step in agent execution - supports update by ID
/// 步骤：Agent 执行的最小单元，相同 ID 的步骤会被更新
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Step {
  /// Tool call with status
  /// 工具调用（带状态）
  Tool {
    id: String,
    name: String,
    args: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
    status: ToolStatus,
    timestamp: u64,
  },

  /// Info / thinking message
  /// 信息/思考中提示
  Info {
    id: String,
    text: String,
    timestamp: u64,
  },

  /// Final answer (streamed, can be updated)
  /// 最终答案（流式，可更新）
  Answer {
    content: String,
    done: bool,
    timestamp: u64,
  },
}

/// Tool execution status
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ToolStatus {
  #[serde(rename = "pending")]
  Pending,
  #[serde(rename = "running")]
  Running,
  #[serde(rename = "success")]
  Success,
  #[serde(rename = "error")]
  Error,
}

impl Step {
  /// Get step ID for updates
  pub fn id(&self) -> Option<&str> {
    match self {
      Step::Tool { id, .. } => Some(id),
      Step::Info { id, .. } => Some(id),
      Step::Answer { .. } => None,
    }
  }

  /// Check if this is an answer step
  pub fn is_answer(&self) -> bool {
    matches!(self, Step::Answer { .. })
  }

  /// Check if step is done
  pub fn is_done(&self) -> bool {
    matches!(self, Step::Answer { done: true, .. })
  }
}

/// Create steps
impl Step {
  /// Create pending tool step
  pub fn tool_pending(id: impl Into<String>, name: impl Into<String>, args: Value) -> Self {
    Step::Tool {
      id: id.into(),
      name: name.into(),
      args,
      result: None,
      status: ToolStatus::Pending,
      timestamp: now(),
    }
  }

  /// Create running tool step
  pub fn tool_running(id: impl Into<String>, name: impl Into<String>, args: Value) -> Self {
    Step::Tool {
      id: id.into(),
      name: name.into(),
      args,
      result: None,
      status: ToolStatus::Running,
      timestamp: now(),
    }
  }

  /// Create completed tool step
  pub fn tool_success(id: impl Into<String>, name: impl Into<String>, args: Value, result: impl Into<String>) -> Self {
    Step::Tool {
      id: id.into(),
      name: name.into(),
      args,
      result: Some(result.into()),
      status: ToolStatus::Success,
      timestamp: now(),
    }
  }

  /// Create info step
  pub fn info(id: impl Into<String>, text: impl Into<String>) -> Self {
    Step::Info {
      id: id.into(),
      text: text.into(),
      timestamp: now(),
    }
  }

  /// Create answer step
  pub fn answer(content: impl Into<String>, done: bool) -> Self {
    Step::Answer {
      content: content.into(),
      done,
      timestamp: now(),
    }
  }
}

fn now() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

// ============================================================================
// PART 2: Error Types
// ============================================================================

/// Agent error type
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
  #[error("MCP client error: {0}")]
  McpClient(String),

  #[error("AI error: {0}")]
  Ai(String),

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
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct ToolCall {
  pub name: String,
  pub arguments: Value,
}
