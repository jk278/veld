//! MCP Agent Service
//! MCP 代理服务，负责工具调用与 AI 交互循环

use crate::config::AppConfig;
use crate::services::ai_client::{AiClient, ChatMessage};
use crate::services::mcp_client::{McpClient, McpTool};
use serde_json::Value;
use tokio::sync::mpsc;

/// Agent step for progressive rendering
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentStep {
    /// AI is thinking (short message, optional detailed content)
    Thinking { short: String, content: Option<String> },
    /// Connecting to MCP server
    Connecting(String),
    /// Calling a tool
    ToolCall { name: String, args: serde_json::Value },
    /// Tool execution result
    ToolResult { name: String, result: String },
    /// Final answer
    Final(String),
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

pub type Result<T> = std::result::Result<T, AgentError>;

/// Tool call request from AI
#[derive(Debug, Clone, serde::Deserialize)]
struct ToolCall {
    name: String,
    arguments: Value,
}

/// Process chat with MCP tool support
/// Sends AgentStep updates through the channel for progressive rendering
pub async fn chat_with_tools(
    messages: Vec<ChatMessage>,
    tx: mpsc::UnboundedSender<AgentStep>,
) -> Result<String> {
    // Load enabled MCP servers
    let config = AppConfig::load().map_err(|e| AgentError::McpClient(e.to_string()))?;
    let enabled_servers = config.get_enabled_mcps();

    if enabled_servers.is_empty() {
        // No MCP servers, just do normal chat
        let response = AiClient::chat_completion(messages)
            .await
            .map_err(|e| AgentError::Ai(e.to_string()))?;
        let _ = tx.send(AgentStep::Final(response.clone()));
        return Ok(response);
    }

    // Connect to all MCP servers and collect tools
    let _ = tx.send(AgentStep::Connecting(format!("连接到 {} 个MCP服务器...", enabled_servers.len())));

    let (sync_tx, sync_rx) = std::sync::mpsc::channel();
    let server_configs: Vec<_> = enabled_servers.iter().map(|s| {
        (s.name.clone(), s.command.clone(), s.args.clone(), s.env.clone())
    }).collect();

    std::thread::spawn(move || {
        let mut results: Vec<(String, McpClient, Vec<McpTool>)> = Vec::new();

        for (name, command, args, env) in server_configs {
            eprintln!("[MCP] Connecting to {}...", name);
            match McpClient::connect(&command, &args, env.as_ref()) {
                Ok(mut client) => {
                    match client.list_tools() {
                        Ok(tools) => {
                            eprintln!("[MCP] {} loaded {} tools", name, tools.len());
                            results.push((name, client, tools));
                        }
                        Err(e) => {
                            eprintln!("[MCP] Failed to list tools for {}: {}", name, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[MCP] Failed to connect to {}: {}", name, e);
                }
            }
        }

        let _ = sync_tx.send(results);
    });

    // Wait with timeout (90 seconds for npx to download packages on first run)
    let results = match sync_rx.recv_timeout(std::time::Duration::from_secs(90)) {
        Ok(r) => r,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            let _ = tx.send(AgentStep::Connecting("连接超时，切换到普通对话".to_string()));
            let response = AiClient::chat_completion(messages)
                .await
                .map_err(|e| AgentError::Ai(e.to_string()))?;
            let _ = tx.send(AgentStep::Final(response.clone()));
            return Ok(response);
        }
        Err(_) => {
            let _ = tx.send(AgentStep::Connecting("连接失败，切换到普通对话".to_string()));
            let response = AiClient::chat_completion(messages)
                .await
                .map_err(|e| AgentError::Ai(e.to_string()))?;
            let _ = tx.send(AgentStep::Final(response.clone()));
            return Ok(response);
        }
    };

    let mut all_tools: Vec<McpTool> = Vec::new();
    let mut clients: Vec<McpClient> = Vec::new();

    for (name, client, tools) in results {
        let _ = tx.send(AgentStep::Connecting(format!("{}: 加载了 {} 个工具", name, tools.len())));
        all_tools.extend(tools);
        clients.push(client);
    }

    if all_tools.is_empty() {
        let _ = tx.send(AgentStep::Connecting("没有加载到工具，切换到普通对话".to_string()));
        let response = AiClient::chat_completion(messages)
            .await
            .map_err(|e| AgentError::Ai(e.to_string()))?;
        let _ = tx.send(AgentStep::Final(response.clone()));
        return Ok(response);
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
    enhanced_messages.insert(0, ChatMessage {
        role: "system".to_string(),
        content: system_instructions,
    });

    // Agent loop
    let max_iterations = 10;
    let mut current_messages = enhanced_messages;

    for iteration in 0..max_iterations {
        // Get AI response
        let response = AiClient::chat_completion(current_messages.clone())
            .await
            .map_err(|e| {
                eprintln!("[MCP] AI error: {}", e);
                AgentError::Ai(e.to_string())
            })?;

        let response_preview = if response.len() > 100 {
            format!("{}...", &response[..100])
        } else {
            response.clone()
        };
        eprintln!("[MCP] [ITERATION {}] Response ({} chars): {}", iteration + 1, response.len(), response_preview);

        // Check if response contains a tool call
        match parse_tool_call(&response) {
            Ok(tool_call) => {
                eprintln!("[MCP] [ITERATION {}] Tool call detected: {:?}", iteration + 1, tool_call);

                // Send thinking step with AI response as content (user can expand to see tool call request)
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
            Err(e) => {
                eprintln!("[MCP] [ITERATION {}] No tool call: {}", iteration + 1, e);
                // No tool call, return final response
                let _ = tx.send(AgentStep::Final(response.clone()));
                return Ok(response);
            }
        }
    }

    // Max iterations reached
    eprintln!("[MCP] Maximum iterations reached");
    let _ = tx.send(AgentStep::Final("达到最大迭代次数".to_string()));
    Err(AgentError::Ai("Maximum iterations reached".to_string()))
}

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
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();

            props.as_object()
                .unwrap()
                .iter()
                .map(|(param_name, param_def)| {
                    let param_type = param_def.get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("any");

                    let description = param_def.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");

                    let is_required = required_list.contains(param_name);
                    let req_marker = if is_required { " (required)" } else { "" };

                    format!("- `{}: {}`{}: {}", param_name, param_type, req_marker, description)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => "No parameters defined.".to_string()
    }
}

/// Parse tool call from AI response
fn parse_tool_call(response: &str) -> Result<ToolCall> {
    let response_trimmed = response.trim();

    // Remove markdown code blocks
    let cleaned = if response_trimmed.starts_with("```") {
        let lines: Vec<&str> = response_trimmed.lines().collect();
        let start_idx = if lines[0].contains("json") { 1 } else { 1 };
        let end_idx = lines.iter().rposition(|l| *l == "```").unwrap_or(lines.len());
        lines[start_idx..end_idx].join("\n")
    } else {
        response_trimmed.to_string()
    };

    eprintln!("[MCP] Parsing tool call from (first 300 chars): {}...", &cleaned.chars().take(300).collect::<String>());

    // Strategy 1: Try direct parse (entire response is JSON)
    if let Ok(v) = serde_json::from_str::<Value>(&cleaned) {
        eprintln!("[MCP] Strategy 1: Direct JSON parse successful, has tool_call key: {}", v.get("tool_call").is_some());
        if let Some(tc) = v.get("tool_call") {
            eprintln!("[MCP] Strategy 1: Found tool_call: {}", tc);
            return Ok(serde_json::from_value(tc.clone())
                .map_err(|e| AgentError::ToolParse(e.to_string()))?);
        }
    } else {
        eprintln!("[MCP] Strategy 1: Not valid JSON, trying extraction...");
    }

    // Strategy 2: Extract JSON from text (when JSON is embedded in response)
    // Find the last occurrence of {"tool_call": ...} pattern
    if let Some(start) = cleaned.find(r#"{"tool_call":"#) {
        let from_start = &cleaned[start..];
        // Find matching closing brace for the tool_call object
        let mut brace_count = 0;
        let mut in_string = false;
        let mut end_idx = 0;

        for (i, c) in from_start.chars().enumerate() {
            match c {
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_idx = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end_idx > 0 {
            let json_str = &from_start[..end_idx];
            eprintln!("[MCP] Extracted JSON: {}", json_str);

            if let Ok(v) = serde_json::from_str::<Value>(json_str) {
                if let Some(tc) = v.get("tool_call") {
                    eprintln!("[MCP] Found tool_call: {}", tc);
                    return Ok(serde_json::from_value(tc.clone())
                        .map_err(|e| AgentError::ToolParse(e.to_string()))?);
                }
            }
        }
    }

    // Strategy 3: Look for tool_call pattern with regex-like approach
    // Find "name": "xxx", "arguments": {...} pattern
    if cleaned.contains("\"tool_call\"") {
        // Extract the tool_call value
        if let Some(start) = cleaned.find("\"tool_call\"") {
            // Find the opening brace after "tool_call":
            let after_key = &cleaned[start..];
            if let Some(brace_start) = after_key.find('{') {
                let from_brace = &after_key[brace_start..];
                // Find matching closing brace with string awareness
                let mut brace_count = 0;
                let mut in_string = false;
                let mut end_idx = 0;

                for (i, c) in from_brace.chars().enumerate() {
                    match c {
                        '"' if !in_string => in_string = true,
                        '"' if in_string => in_string = false,
                        '{' if !in_string => brace_count += 1,
                        '}' if !in_string => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_idx = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if end_idx > 0 {
                    let tool_call_json = &from_brace[..end_idx];
                    eprintln!("[MCP] Extracted tool_call JSON: {}", tool_call_json);

                    if let Ok(tc) = serde_json::from_str::<Value>(tool_call_json) {
                        eprintln!("[MCP] Found tool_call: {}", tc);
                        return Ok(serde_json::from_value(tc.clone())
                            .map_err(|e| AgentError::ToolParse(e.to_string()))?);
                    }
                }
            }
        }
    }

    eprintln!("[MCP] No tool_call found in response");
    Err(AgentError::ToolParse("No tool call found".to_string()))
}

/// Execute a tool call
fn execute_tool_call(tool_call: &ToolCall, clients: &mut [McpClient]) -> Result<String> {
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
