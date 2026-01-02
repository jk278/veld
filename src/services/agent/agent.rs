//! MCP Agent - Tool is Result, Result is Tool
//! MCP 代理 - 工具即结果，结果即工具

use super::executor::execute_tool_call;
use super::types::{AgentError, Result, Step, ToolCall};
use crate::config::AppConfig;
use crate::services::ai_client::{AiClient, ChatMessage};
use crate::services::mcp_client::{McpClient, McpTool};
use dioxus::logger::tracing::{debug, info, warn};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

// ============================================================================
// Part 1: Tool Prompt Builder
// ============================================================================

fn build_tools_prompt(tools: &[McpTool]) -> String {
  if tools.is_empty() {
    return "No tools available.".to_string();
  }

  tools
    .iter()
    .map(|tool| {
      let schema = format_tool_schema(&tool.input_schema);
      format!("**{}**: {}\n\n{}", tool.name, tool.description, schema)
    })
    .collect::<Vec<_>>()
    .join("\n\n")
}

fn format_tool_schema(schema: &Value) -> String {
  let properties = schema.get("properties");
  let required = schema.get("required");

  match properties {
    Some(props) if props.is_object() => {
      let required_list: Vec<String> = required
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

      props
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, def)| {
          let param_type = def.get("type").and_then(|t| t.as_str()).unwrap_or("any");
          let description = def.get("description").and_then(|d| d.as_str()).unwrap_or("");
          let is_required = required_list.contains(name);
          let req_marker = if is_required { " (required)" } else { "" };
          format!("- `{}: {}`{}: {}", name, param_type, req_marker, description)
        })
        .collect::<Vec<_>>()
        .join("\n")
    }
    _ => "No parameters defined.".to_string(),
  }
}

// ============================================================================
// Part 2: MCP Connection
// ============================================================================

struct McpConnections {
  clients: Vec<McpClient>,
  tools: Vec<McpTool>,
}

async fn connect_mcp_servers(tx: &mpsc::UnboundedSender<Step>) -> Option<McpConnections> {
  let (sync_tx, sync_rx) = oneshot::channel();
  let config = AppConfig::load().ok()?;

  let server_configs: Vec<_> = config
    .mcp
    .servers
    .iter()
    .map(|s| (s.name.clone(), s.command.clone(), s.args.clone(), s.env.clone()))
    .collect();

  tokio::task::spawn_blocking(move || {
    let mut results: Vec<(String, Option<McpClient>, Vec<McpTool>)> = Vec::new();

    for (name, command, args, env) in server_configs {
      match McpClient::connect(&command, &args, env.as_ref()) {
        Ok(mut client) => match client.list_tools() {
          Ok(tools) => {
            info!("[MCP] {} loaded {} tools", name, tools.len());
            results.push((name, Some(client), tools));
          }
          Err(e) => {
            warn!("[MCP] Failed to list tools for {}: {}", name, e);
            results.push((name, None, Vec::new()));
          }
        },
        Err(e) => {
          warn!("[MCP] Failed to connect to {}: {}", name, e);
          results.push((name, None, Vec::new()));
        }
      }
    }

    let _ = sync_tx.send(results);
  });

  let results = tokio::time::timeout(std::time::Duration::from_secs(90), sync_rx)
    .await
    .ok()?
    .ok()?;

  let mut clients = Vec::new();
  let mut all_tools = Vec::new();

  for (name, client, tools) in results {
    if let Some(client) = client {
      let _ = tx.send(Step::info(format!("conn-{}", name), format!("{}: 加载了 {} 个工具", name, tools.len())));
      all_tools.extend(tools);
      clients.push(client);
    }
  }

  if all_tools.is_empty() {
    let _ = tx.send(Step::info("no-tools", "没有加载到工具，切换到普通对话".to_string()));
    return None;
  }

  Some(McpConnections {
    clients,
    tools: all_tools,
  })
}

// ============================================================================
// Part 3: Tool Call Extraction
// ============================================================================

/// Extract tool call from text (supports embedded in content)
fn extract_tool_call(text: &str) -> Option<ToolCall> {
  let trimmed = text.trim();

  // Try direct JSON parse first (handles pure tool call)
  if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
    if let Some(tc) = v.get("tool_call") {
      return serde_json::from_value(tc.clone()).ok();
    }
  }

  // Fallback: extract embedded tool_call from mixed content
  if trimmed.contains("\"tool_call\"") {
    // Find the position after "tool_call":
    if let Some(key_pos) = trimmed.find("\"tool_call\"") {
      // Find the colon after the key
      let after_key = &trimmed[key_pos + "\"tool_call\"".len()..];
      if let Some(colon_pos) = after_key.find(':') {
        // Find the opening brace
        let after_colon = &after_key[colon_pos + 1..];
        if let Some(brace_start) = after_colon.find('{') {
          let from_brace = &after_colon[brace_start + 1..];

          // Find matching closing brace and collect chars to avoid UTF-8 boundary issues
          let mut brace_count = 1usize;
          let mut in_string = false;
          let mut brace_content = String::new();

          for c in from_brace.chars() {
            brace_content.push(c);
            match c {
              '"' if !in_string => in_string = true,
              '"' if in_string => in_string = false,
              '{' if !in_string => brace_count += 1,
              '}' if !in_string => {
                brace_count -= 1;
                if brace_count == 0 {
                  break;
                }
              }
              _ => {}
            }

            // Safety limit: don't scan too far (but allow large tool calls)
            if brace_content.len() > 50000 {
              break;
            }
          }

          if !brace_content.is_empty() {
            debug!("[AGENT] Extracted tool JSON ({} chars)", brace_content.len());
            return serde_json::from_str(&brace_content).ok();
          }
        }
      }
    }
  }

  None
}

// ============================================================================
// Part 4: Agent Execution - Tool is Result
// ============================================================================

/// Main entry point: stream answer, detect and execute tools inline
pub async fn chat_with_tools(
  messages: Vec<ChatMessage>,
  tx: mpsc::UnboundedSender<Step>,
  mut abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  let client = AiClient::new()?;

  // Try to connect to MCP servers
  let mcp = match connect_mcp_servers(&tx).await {
    Some(connections) => connections,
    None => {
      return stream_answer_simple(client, messages, tx, abort_rx).await;
    }
  };

  // Build system prompt with tools
  let tools_prompt = build_tools_prompt(&mcp.tools);
  let system_instructions = format!(
    "You are an AI assistant with access to MCP (Model Context Protocol) tools.\n\n\
     Available MCP tools:\n{}\n\n\
     OUTPUT FORMAT:\n\
     - Your response is a continuous stream (no separation between tools and answer)\n\
     - To use a tool: embed {{\"tool_call\": {{\"name\": \"tool_name\", \"arguments\": {{...}}}}}} in your response\n\
     - You can use multiple tools in one response\n\
     - After tool results are provided, continue your response naturally",
    tools_prompt
  );

  // Build messages
  let mut current_messages = messages.clone();
  current_messages.insert(
    0,
    ChatMessage {
      role: "system".to_string(),
      content: system_instructions,
    },
  );

  let mut clients = mcp.clients;
  let mut tool_counter = 0usize;
  let mut accumulated = String::new();
  let mut answer_buffer = String::new();

  // Start streaming
  let mut rx = client.chat(current_messages.clone()).await?;

  loop {
    tokio::select! {
      _ = abort_rx.recv() => {
        let _ = tx.send(Step::answer("", true));
        return Ok(String::new());
      }
      chunk_result = rx.recv() => {
        let chunk = match chunk_result {
          Some(Ok(c)) => c,
          Some(Err(e)) => {
            let _ = tx.send(Step::answer("", true));
            return Err(AgentError::Ai(e.to_string()));
          }
          None => {
            if !answer_buffer.is_empty() {
              let _ = tx.send(Step::answer(&answer_buffer, true));
            }
            return Ok(accumulated);
          }
        };

        accumulated.push_str(&chunk);
        answer_buffer.push_str(&chunk);

        // Detect tool call in accumulated content
        while let Some(tool_call) = extract_tool_call(&accumulated) {
          tool_counter += 1;
          let tool_id = format!("tool-{}", tool_counter);

          info!("[AGENT] Detected tool call: {}", tool_call.name);

          // IMPORTANT: Flush answer buffer BEFORE executing tool
          // This ensures any text before the tool call is displayed first
          if !answer_buffer.is_empty() {
            let _ = tx.send(Step::answer(&answer_buffer.clone(), false));
            answer_buffer.clear();
          }

          // Emit: tool pending
          let _ = tx.send(Step::tool_pending(&tool_id, &tool_call.name, tool_call.arguments.clone()));

          // Emit: tool running
          let _ = tx.send(Step::tool_running(&tool_id, &tool_call.name, tool_call.arguments.clone()));

          // Execute tool
          let result = execute_tool_call(&tool_call, &mut clients)?;

          // Emit: tool success
          let _ = tx.send(Step::tool_success(&tool_id, &tool_call.name, tool_call.arguments.clone(), &result));

          // Append tool result to message history for AI context
          current_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: format!("{{\"tool_call\": {{\"name\": \"{}\", \"arguments\": {}}}}}",
              tool_call.name,
              serde_json::to_string(&tool_call.arguments).unwrap_or_default()
            ),
          });
          current_messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!("Tool result: {}", result),
          });

          // Continue streaming from where we left off
          rx = client.chat(current_messages.clone()).await?;
          accumulated.clear();

          // Continue the loop to receive more chunks
          break;
        }

        // Stream answer chunk (only if no tool call detected)
        if extract_tool_call(&accumulated).is_none() {
          if !answer_buffer.is_empty() {
            let _ = tx.send(Step::answer(&answer_buffer.clone(), false));
            answer_buffer.clear();
          }
        }
      }
    }
  }
}

/// Stream answer without tools (fallback)
async fn stream_answer_simple(
  client: AiClient,
  messages: Vec<ChatMessage>,
  tx: mpsc::UnboundedSender<Step>,
  mut abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  let mut rx = client.chat(messages).await?;
  let mut full = String::new();

  loop {
    tokio::select! {
      _ = abort_rx.recv() => {
        let _ = tx.send(Step::answer("", true));
        return Ok(String::new());
      }
      chunk_result = rx.recv() => {
        match chunk_result {
          Some(Ok(chunk)) => {
            full.push_str(&chunk);
            let _ = tx.send(Step::answer(&chunk, false));
          }
          Some(Err(e)) => {
            let _ = tx.send(Step::answer("", true));
            return Err(AgentError::Ai(e.to_string()));
          }
          None => {
            let _ = tx.send(Step::answer("", true));
            return Ok(full);
          }
        }
      }
    }
  }
}
