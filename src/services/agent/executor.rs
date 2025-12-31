//! Tool executor
//! 工具执行器

use super::types::{AgentError, Result, ToolCall};
use crate::services::mcp_client::McpClient;

/// Execute a tool call
pub fn execute_tool_call(tool_call: &ToolCall, clients: &mut [McpClient]) -> Result<String> {
  // Find client with the tool and execute
  for client in clients.iter_mut() {
    match client.call_tool(&tool_call.name, tool_call.arguments.clone()) {
      Ok(result) => {
        return Ok(serde_json::to_string(&result).unwrap_or_default());
      }
      Err(_) => continue,
    }
  }

  Err(AgentError::McpClient(format!(
    "Tool not found: {}",
    tool_call.name
  )))
}
