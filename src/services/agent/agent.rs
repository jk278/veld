//! MCP Agent - Main entry point
//! MCP 代理主入口

use super::executor::execute_tool_call;
use super::parser::parse_tool_call;
use super::stream::stream_chat_response;
use super::types::{AgentError, AgentStep, Result};
use crate::config::AppConfig;
use crate::services::ai_client::{AiClient, ChatMessage};
use crate::services::mcp_client::{McpClient, McpTool};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

/// Build tools prompt for AI (generic MCP tool schema handling)
fn build_tools_prompt(tools: &[McpTool]) -> String {
  if tools.is_empty() {
    return "No tools available.".to_string();
  }

  tools
    .iter()
    .map(|tool| {
      // Generic JSON Schema parsing for any MCP tool
      let schema = format_tool_schema(&tool.input_schema);
      format!("**{}**: {}\n\n{}", tool.name, tool.description, schema)
    })
    .collect::<Vec<_>>()
    .join("\n\n")
}

/// Format tool input schema for AI (generic JSON Schema handler)
fn format_tool_schema(schema: &Value) -> String {
  let properties = schema.get("properties");
  let required = schema.get("required");

  match properties {
    Some(props) if props.is_object() => {
      let required_list: Vec<String> = required
        .and_then(|r| r.as_array())
        .map(|arr| {
          arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
        })
        .unwrap_or_default();

      props
        .as_object()
        .unwrap()
        .iter()
        .map(|(param_name, param_def)| {
          let param_type = param_def
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("any");

          let description = param_def
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");

          let is_required = required_list.contains(param_name);
          let req_marker = if is_required { " (required)" } else { "" };

          format!(
            "- `{}: {}`{}: {}",
            param_name, param_type, req_marker, description
          )
        })
        .collect::<Vec<_>>()
        .join("\n")
    }
    _ => "No parameters defined.".to_string(),
  }
}

/// Process chat with MCP tool support
/// Sends AgentStep updates through the channel for progressive rendering
/// abort_rx: optional receiver for abort signal (send any value to abort)
pub async fn chat_with_tools(
  messages: Vec<ChatMessage>,
  tx: mpsc::UnboundedSender<AgentStep>,
  mut abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  let client = AiClient::new()?;

  // Connect to MCP servers concurrently
  let (sync_tx, mut sync_rx) = oneshot::channel();
  let config = AppConfig::load().map_err(|e| AgentError::Ai(e.to_string()))?;
  let server_configs: Vec<(String, String, Vec<String>, Option<std::collections::HashMap<String, String>>)> = config
    .mcp
    .servers
    .iter()
    .map(|s| {
      (
        s.name.clone(),
        s.command.clone(),
        s.args.clone(),
        s.env.clone(),
      )
    })
    .collect();

  // Use spawn_blocking to run blocking MCP operations in a separate thread
  // This avoids blocking the async runtime and integrates properly with Dioxus
  tokio::task::spawn_blocking(move || {
    let mut results: Vec<(String, McpClient, Vec<McpTool>)> = Vec::new();

    for (name, command, args, env) in server_configs {
      eprintln!("[MCP] Connecting to {}...", name);
      match McpClient::connect(&command, &args, env.as_ref()) {
        Ok(mut client) => match client.list_tools() {
          Ok(tools) => {
            eprintln!("[MCP] {} loaded {} tools", name, tools.len());
            results.push((name, client, tools));
          }
          Err(e) => {
            eprintln!("[MCP] Failed to list tools for {}: {}", name, e);
          }
        },
        Err(e) => {
          eprintln!("[MCP] Failed to connect to {}: {}", name, e);
        }
      }
    }

    let _ = sync_tx.send(results);
  });

  // Wait with timeout (90 seconds for npx to download packages on first run)
  let results = match tokio::time::timeout(std::time::Duration::from_secs(90), sync_rx).await {
    Ok(Ok(r)) => r,
    Ok(Err(_)) => {
      let _ = tx.send(AgentStep::Connecting(
        "连接失败，切换到普通对话".to_string(),
      ));
      return stream_chat_response(client, messages, tx, abort_rx.resubscribe()).await;
    }
    Err(_) => {
      let _ = tx.send(AgentStep::Connecting(
        "连接超时，切换到普通对话".to_string(),
      ));
      return stream_chat_response(client, messages, tx, abort_rx.resubscribe()).await;
    }
  };

  let mut all_tools: Vec<McpTool> = Vec::new();
  let mut clients: Vec<McpClient> = Vec::new();

  for (name, client, tools) in results {
    let _ = tx.send(AgentStep::Connecting(format!(
      "{}: 加载了 {} 个工具",
      name,
      tools.len()
    )));
    all_tools.extend(tools);
    clients.push(client);
  }

  if all_tools.is_empty() {
    let _ = tx.send(AgentStep::Connecting(
      "没有加载到工具，切换到普通对话".to_string(),
    ));
    return stream_chat_response(client, messages, tx, abort_rx.resubscribe()).await;
  }

  // Build system prompt with tool definitions
  let tools_prompt = build_tools_prompt(&all_tools);

  // Build system instructions - strict format enforcement
  let system_instructions = format!(
        "You are an AI assistant with access to MCP (Model Context Protocol) tools.\n\n\
        Available MCP tools:\n{}\n\n\
        CRITICAL OUTPUT FORMAT RULES:\n\
        1. To use a tool: Respond with ONLY a JSON object (no other text): {{\"tool_call\": {{\"name\": \"tool_name\", \"arguments\": {{...}}}}}}\n\
        2. To respond to user: Use normal text (no JSON)\n\
        3. NEVER mix JSON with other text - the JSON must be the ENTIRE response\n\
        4. After tool result is returned, you can then respond normally to the user\n\n\
        Example:\n\
        User: Search for Rust documentation\n\
        Assistant: {{\"tool_call\": {{\"name\": \"search-docs\", \"arguments\": {{\"query\": \"Rust\"}}}}}}\n\n\
        (Then after receiving tool result, you respond with actual answer)",
        tools_prompt
    );

  // Add tools context to messages (system message for Anthropic API)
  let mut enhanced_messages: Vec<ChatMessage> = messages.clone();
  enhanced_messages.insert(
    0,
    ChatMessage {
      role: "system".to_string(),
      content: system_instructions,
    },
  );

  // Agent loop
  let max_iterations = 10;
  let mut current_messages = enhanced_messages;

  for iteration in 0..max_iterations {
    // Get AI response stream
    let mut rx = client.chat(current_messages.clone()).await.map_err(|e| {
      eprintln!("[MCP] AI error: {}", e);
      AgentError::Ai(e.to_string())
    })?;

    // Stream response: accumulate to detect tool call, or stream directly
    let mut accumulated = String::new();
    let mut is_tool_call = false;

    // Read first chunks to check for tool call (tool calls usually start immediately)
    loop {
      tokio::select! {
        // Check for abort signal
        _ = abort_rx.recv() => {
          eprintln!("[MCP] Aborted by user");
          let _ = tx.send(AgentStep::Final);
          return Ok(String::new());
        }
        // Receive stream chunk
        chunk_result = rx.recv() => {
          let chunk = match chunk_result {
            Some(Ok(c)) => c,
            Some(Err(e)) => {
              eprintln!("[MCP] Stream error: {}", e);
              return Err(AgentError::Ai(e.to_string()));
            }
            None => break, // Stream ended
          };
          accumulated.push_str(&chunk);

          // Check if we have enough content to detect tool call
          if accumulated.len() > 50 {
            if parse_tool_call(&accumulated).is_ok() {
              is_tool_call = true;
              // Collect remaining response
              loop {
                tokio::select! {
                  _ = abort_rx.recv() => {
                    eprintln!("[MCP] Aborted during tool call collection");
                    let _ = tx.send(AgentStep::Final);
                    return Ok(String::new());
                  }
                  chunk_result = rx.recv() => {
                    match chunk_result {
                      Some(Ok(c)) => accumulated.push_str(&c),
                      Some(Err(e)) => {
                        eprintln!("[MCP] Stream error: {}", e);
                        return Err(AgentError::Ai(e.to_string()));
                      }
                      None => break,
                    }
                  }
                }
              }
              break;
            } else {
              // Not a tool call, stream accumulated and continue streaming
              if !accumulated.is_empty() {
                let _ = tx.send(AgentStep::Chunk(accumulated.clone()));
              }

              // Stream remaining chunks directly
              loop {
                tokio::select! {
                  _ = abort_rx.recv() => {
                    eprintln!("[MCP] Aborted during streaming");
                    let _ = tx.send(AgentStep::Final);
                    return Ok(String::new());
                  }
                  chunk_result = rx.recv() => {
                    match chunk_result {
                      Some(Ok(chunk)) => {
                        let _ = tx.send(AgentStep::Chunk(chunk));
                      }
                      Some(Err(e)) => {
                        eprintln!("[MCP] Stream error: {}", e);
                        let _ = tx.send(AgentStep::Final);
                        return Err(AgentError::Ai(e.to_string()));
                      }
                      None => {
                        let _ = tx.send(AgentStep::Final);
                        return Ok(String::new());
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }

    // If accumulated but not decided (short response), check for tool call
    if !is_tool_call && !accumulated.is_empty() {
      if parse_tool_call(&accumulated).is_ok() {
        is_tool_call = true;
      } else {
        // Not a tool call, stream it (no byte-level splitting)
        let _ = tx.send(AgentStep::Chunk(accumulated.clone()));
        let _ = tx.send(AgentStep::Final);
        return Ok(String::new());
      }
    }

    // Handle tool call path
    if is_tool_call {
      let response = accumulated;

      match parse_tool_call(&response) {
        Ok(tool_call) => {
          // Tool call detected - send as Thinking step (not streamed)
          let _ = tx.send(AgentStep::Thinking {
            short: format!("思考中 (第{}轮)...", iteration + 1),
            content: Some(response.clone()),
          });

          // Send tool call step
          let _ = tx.send(AgentStep::ToolCall {
            name: tool_call.name.clone(),
            args: tool_call.arguments.clone(),
          });

          // Execute tool call
          let tool_result = execute_tool_call(&tool_call, &mut clients)?;

          // Send tool result step
          let _ = tx.send(AgentStep::ToolResult {
            name: tool_call.name.clone(),
            result: tool_result.clone(),
          });

          // Add assistant message with tool call
          current_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: response,
          });

          // Add tool result as user message
          current_messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!("Tool result: {}", tool_result),
          });

          // Continue loop
        }
        Err(_) => {
          // Parse failed, stream as normal response
          for char_chunk in response.chars().collect::<Vec<char>>().chunks(10) {
            let _ = tx.send(AgentStep::Chunk(char_chunk.iter().collect()));
          }
          let _ = tx.send(AgentStep::Final);
          return Ok(response);
        }
      }
    }
  }

  // Max iterations reached
  eprintln!("[MCP] Maximum iterations reached");
  let _ = tx.send(AgentStep::Final);
  Err(AgentError::Ai("Maximum iterations reached".to_string()))
}
