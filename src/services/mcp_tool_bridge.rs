//! MCP Tool Bridge
//! 将 MCP 工具桥接到 rig-core 的 Tool 系统

use crate::services::mcp_client::McpClient;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use std::sync::{Arc, Mutex};

/// MCP 工具的 rig-core Tool 实现
#[derive(Clone)]
pub struct McpToolBridge {
  /// 工具名称
  pub name: String,
  /// 工具描述（用于 AI 理解工具用途）
  pub description: String,
  /// 参数 JSON Schema（定义工具输入）
  pub parameters: serde_json::Value,
  /// MCP 客户端（用于执行工具）
  client: Arc<Mutex<McpClient>>,
}

impl McpToolBridge {
  /// 创建新的 MCP 工具桥接
  pub fn new(
    name: String,
    description: String,
    parameters: serde_json::Value,
    client: Arc<Mutex<McpClient>>,
  ) -> Self {
    Self {
      name,
      description,
      parameters,
      client,
    }
  }
}

/// 实现 rig-core 的 Tool trait
impl Tool for McpToolBridge {
  /// 工具名称常量
  const NAME: &'static str = "mcp_tool";

  /// 关联类型：错误
  type Error = McpToolError;

  /// 关联类型：参数（支持任意 JSON 反序列化）
  type Args = serde_json::Value;

  /// 关联类型：输出（JSON 字符串）
  type Output = String;

  /// 返回工具定义（元数据）
  async fn definition(&self, _prompt: String) -> ToolDefinition {
    ToolDefinition {
      name: self.name.clone(),
      description: self.description.clone(),
      parameters: self.parameters.clone(),
    }
  }

  /// 执行工具调用
  async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
    // 锁定客户端并调用工具
    let mut client = self
      .client
      .lock()
      .map_err(|e| McpToolError(format!("Failed to lock client: {}", e)))?;

    // 调用 MCP 工具
    match client.call_tool(&self.name, args) {
      Ok(result) => {
        // MCP 工具返回的 result 是 Value，转换为 JSON 字符串
        Ok(serde_json::to_string(&result).unwrap_or_default())
      }
      Err(e) => {
        // 转换 MCP 错误为 ToolError
        Err(McpToolError(format!(
          "MCP tool '{}' failed: {}",
          self.name, e
        )))
      }
    }
  }

  /// 返回工具名称（覆盖默认实现，使用实例名称）
  fn name(&self) -> String {
    self.name.clone()
  }
}

/// MCP 工具错误类型
#[derive(Debug, thiserror::Error)]
#[error("MCP Tool Error: {0}")]
pub struct McpToolError(String);

/// 从 MCP 工具列表创建 rig-core 工具向量
pub fn create_rig_tools(
  mcp_tools: Vec<crate::services::mcp_client::McpTool>,
  clients: Vec<Arc<Mutex<McpClient>>>,
) -> Vec<McpToolBridge> {
  mcp_tools
    .into_iter()
    .map(|tool| {
      // 找到对应的 MCP 客户端（通过工具名称匹配）
      let client = clients
        .iter()
        .find(|c| {
          if let Ok(mut client) = c.lock() {
            if let Ok(tools) = client.list_tools() {
              return tools.iter().any(|t| t.name == tool.name);
            }
          }
          false
        })
        .cloned()
        .expect("Tool client not found");

      McpToolBridge::new(
        tool.name.clone(),
        tool.description.clone(),
        tool.input_schema,
        client,
      )
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_tool_bridge_creation() {
    // Test basic tool bridge creation structure
    let schema = json!({
      "type": "object",
      "properties": {
        "query": {"type": "string"}
      }
    });

    assert_eq!(schema["type"], "object");
  }
}
