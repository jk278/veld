//! MCP Agent Service - rig-core native integration
//! MCP 代理服务 - rig-core 原生集成

mod types;

pub use types::{AgentError, Result, Step, ToolStatus};

use crate::config::AppConfig;
use crate::services::mcp_client::McpClient;
use crate::services::mcp_tool_bridge::McpToolBridge;
use dioxus::logger::tracing::{info, warn};
use futures_util::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::client::CompletionClient;
use rig::completion::Message;
use rig::message::{ToolResultContent, UserContent};
use rig::providers::anthropic;
use rig::providers::openai;
use rig::streaming::{StreamedAssistantContent, StreamedUserContent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

/// Global atomic counter for unique step IDs
static STEP_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Get next unique step ID
fn next_step_id() -> u64 {
  STEP_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// ============================================================================
// MCP Connection
// ============================================================================

/// 连接到 MCP 服务器并返回工具桥接
async fn connect_mcp_servers(
  tx: &mpsc::UnboundedSender<Step>,
) -> Option<(Vec<McpToolBridge>, Vec<Arc<Mutex<McpClient>>>)> {
  let (sync_tx, sync_rx) = oneshot::channel();
  let config = AppConfig::load().ok()?;

  let server_configs: Vec<_> = config
    .mcp
    .servers
    .iter()
    .map(|s| (s.name.clone(), s.command.clone(), s.args.clone(), s.env.clone()))
    .collect();

  tokio::task::spawn_blocking(move || {
    let mut results: Vec<(String, Option<McpClient>, Vec<crate::services::mcp_client::McpTool>)> = Vec::new();

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
      let _ = tx.send(Step::Info {
        id: format!("conn-{}-{}", name, next_step_id()),
        text: format!("{}: 加载了 {} 个工具", name, tools.len()),
        timestamp: now(),
      });
      all_tools.extend(tools);
      clients.push(Arc::new(Mutex::new(client)));
    }
  }

  if all_tools.is_empty() {
    let _ = tx.send(Step::Info {
      id: format!("no-tools-{}", next_step_id()),
      text: "没有加载到工具，切换到普通对话".to_string(),
      timestamp: now(),
    });
    return None;
  }

  // 创建 rig-core 工具桥接
  let rig_tools = all_tools
    .into_iter()
    .map(|tool| {
      // 找到对应的客户端（通过工具名称匹配）
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
    .collect();

  Some((rig_tools, clients))
}

// ============================================================================
// Main Chat Entry Point
// ============================================================================

/// 使用 MCP 工具的聊天（rig-core 原生实现）
/// - 使用 rig-core 的 .tool() 注册 MCP 工具
/// - 使用 .multi_turn() 自动处理工具链式调用
/// - 流式响应通过 Step 发送
pub async fn chat_with_mcp_tools(
  messages: Vec<crate::services::ChatMessage>,
  tx: mpsc::UnboundedSender<Step>,
  mut abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  // 连接到 MCP 服务器并获取工具
  let (tools, _clients) = match connect_mcp_servers(&tx).await {
    Some(result) => result,
    None => {
      return stream_simple_chat(messages, tx, abort_rx).await;
    }
  };

  // 构建系统提示（工具定义）
  let tools_prompt = build_tools_prompt(&tools);
  let system_instructions = format!(
    "You are an AI assistant with access to MCP (Model Context Protocol) tools.\n\n\
     Available tools:\n{}\n\n\
     Use tools when needed to help the user. You can chain multiple tools together.",
    tools_prompt
  );

  // 解析消息
  let (rig_messages, preamble) = parse_messages(messages, Some(system_instructions));

  // 获取 AI 客户端配置
  let config = AppConfig::load().map_err(|e| AgentError::Ai(e.to_string()))?;
  let active_id = config
    .ai
    .active_provider
    .as_ref()
    .ok_or_else(|| AgentError::Ai("No active provider".to_string()))?;

  let provider = config
    .ai
    .providers
    .iter()
    .find(|p| p.id == *active_id)
    .ok_or_else(|| AgentError::Ai(format!("Provider not found: {}", active_id)))?;

  if !provider.enabled {
    return Err(AgentError::Ai(format!("Provider not enabled: {}", active_id)));
  }

  let api_key = provider
    .api_key
    .as_ref()
    .filter(|k| !k.is_empty())
    .ok_or_else(|| AgentError::Ai("No API key".to_string()))?;

  // 使用 rig-core 的流式 API with tools
  match provider.adapter_type.as_deref() {
    Some("openai") => {
      // 使用 OpenAI Completions API 以获得更好的兼容性
      // 中文 AI provider 通常只支持 Chat Completions API，不支持 Responses API
      let builder = openai::CompletionsClient::builder();
      let builder = if let Some(url) = &provider.base_url {
        builder.api_key(api_key).base_url(url)
      } else {
        builder.api_key(api_key)
      };
      // CompletionsClient<H = reqwest::Client> 默认使用 reqwest::Client
      let client: openai::CompletionsClient = builder
        .build()
        .map_err(|e| AgentError::Ai(e.to_string()))?;

      // 构建 agent - 按正确顺序添加组件
      let agent = if tools.is_empty() {
        // 无工具：直接使用 AgentBuilder
        if let Some(pre) = &preamble {
          Arc::new(client.agent(&provider.model).preamble(pre).build())
        } else {
          Arc::new(client.agent(&provider.model).build())
        }
      } else {
        // 有工具：先添加第一个工具（转换类型），然后添加剩余工具
        let first_tool = &tools[0];
        let mut builder_simple = client.agent(&provider.model).tool(first_tool.clone());
        for tool in &tools[1..] {
          builder_simple = builder_simple.tool(tool.clone());
        }
        if let Some(pre) = &preamble {
          Arc::new(builder_simple.preamble(pre).build())
        } else {
          Arc::new(builder_simple.build())
        }
      };

      // 分离最后一条用户消息和历史
      let (prompt, history) = split_prompt_and_history(rig_messages);

      // DEBUG: 记录 prompt 内容
      info!("📤 Sending to AI - prompt: '{}', history_len: {}", prompt, history.len());
      if prompt.is_empty() {
        warn!("⚠️  Prompt is empty! This might cause API errors");
      }

      // 使用流式 API with multi_turn for tool chaining
      // 显式指定 hook 类型为 ()，因为 () 实现了 StreamingPromptHook<M>
      let stream = rig::agent::StreamingPromptRequest::<_, ()>::new(agent, prompt)
        .with_history(history)
        .multi_turn(10) // 允许最多 10 次工具链式调用
        .await;

      // 直接处理流式响应
      use tokio::select;
      let mut full_response = String::new();
      let mut stream = stream;
      // 存储 call_id -> 工具名的映射，用于 ToolResult 时获取工具名
      let mut tool_name_map: HashMap<String, String> = HashMap::new();
      // 存储 call_id -> 参数的映射，用于 ToolResult 时保留原始参数
      let mut tool_args_map: HashMap<String, serde_json::Value> = HashMap::new();

      loop {
        select! {
          _ = abort_rx.recv() => {
            let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
            return Ok(String::new());
          }
          item = stream.next() => {
            match item {
              Some(Ok(MultiTurnStreamItem::StreamAssistantItem(content))) => {
                match content {
                  // 处理流式文本回答
                  StreamedAssistantContent::Text(text) => {
                    full_response.push_str(&text.text);
                    let _ = tx.send(Step::Answer { content: text.text, done: false, timestamp: now() });
                  }
                  // 处理思考过程
                  StreamedAssistantContent::Reasoning(reasoning) => {
                    let reasoning_text = reasoning.reasoning.join("\n");
                    info!("🤔 Reasoning: {}", reasoning_text);
                    let info_id = format!("reasoning-{}", next_step_id());
                    let _ = tx.send(Step::Info {
                      id: info_id,
                      text: format!("思考中...\n{}", reasoning_text),
                      timestamp: now(),
                    });
                  }
                  // 处理工具调用 - 发送到 UI 显示
                  StreamedAssistantContent::ToolCall(tool_call) => {
                    info!("🔧 Tool call: {} (id: {})", tool_call.function.name, tool_call.id);
                    // 存储工具名映射：call_id -> 工具名
                    tool_name_map.insert(tool_call.id.clone(), tool_call.function.name.clone());
                    // 存储参数映射：call_id -> 参数
                    tool_args_map.insert(tool_call.id.clone(), tool_call.function.arguments.clone());
                    // Use tool_call.id (call_id) which matches tool_result.id
                    let tool_id = format!("tool-{}", tool_call.id);
                    let _ = tx.send(Step::Tool {
                      id: tool_id,
                      name: tool_call.function.name.clone(),
                      args: tool_call.function.arguments.clone(),
                      result: None,
                      status: ToolStatus::Pending,
                      timestamp: now(),
                    });
                  }
                  // 处理其他流式内容（ToolCallDelta, ReasoningDelta等）
                  _ => {}
                }
              }
              // 处理工具结果（当工具执行完成后）
              Some(Ok(MultiTurnStreamItem::StreamUserItem(content))) => {
                match content {
                  StreamedUserContent::ToolResult(tool_result) => {
                    info!("🔧 Tool result: {}", tool_result.id);
                    // tool_result.id 是 call_id，从映射中查找工具名和参数
                    let tool_name = tool_name_map.get(&tool_result.id)
                      .cloned()
                      .unwrap_or_else(|| tool_result.id.clone());
                    let tool_args = tool_args_map.get(&tool_result.id)
                      .cloned()
                      .unwrap_or_else(|| serde_json::json!({}));
                    let tool_id = format!("tool-{}", tool_result.id);

                    // Convert ToolResultContent to String
                    let result_str: String = match tool_result.content.iter().next() {
                      Some(ToolResultContent::Text(text_obj)) => text_obj.text.clone(),
                      Some(ToolResultContent::Image(_)) => "[Image]".to_string(),
                      None => String::new(),
                    };

                    let _ = tx.send(Step::Tool {
                      id: tool_id,
                      name: tool_name,
                      args: tool_args,
                      result: Some(result_str),
                      status: ToolStatus::Success,
                      timestamp: now(),
                    });
                  }
                }
              }
              Some(Ok(MultiTurnStreamItem::FinalResponse(_final_resp))) => {
                // All content has already been streamed via Text events
                // Just send the completion signal (done=true with empty content)
                let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
                return Ok(full_response);
              }
              // 处理其他可能的 MultiTurnStreamItem 变体（非穷尽枚举）
              Some(Ok(_)) => {}
              Some(Err(e)) => {
                let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
                return Err(AgentError::Ai(e.to_string()));
              }
              None => {
                let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
                return Ok(full_response);
              }
            }
          }
        }
      }
    }
    Some("anthropic") => {
      // 使用与 ai_client.rs 相同的 builder 模式
      // Client<H = reqwest::Client> 默认使用 reqwest::Client
      let builder = anthropic::Client::builder();
      let builder = if let Some(url) = &provider.base_url {
        builder.api_key(api_key).base_url(url)
      } else {
        builder.api_key(api_key)
      };
      let client: anthropic::Client = builder
        .build()
        .map_err(|e| AgentError::Ai(e.to_string()))?;

      // 构建 agent - 按正确顺序添加组件
      let agent = if tools.is_empty() {
        // 无工具：直接使用 AgentBuilder
        if let Some(pre) = &preamble {
          Arc::new(client.agent(&provider.model).preamble(pre).max_tokens(4096).build())
        } else {
          Arc::new(client.agent(&provider.model).max_tokens(4096).build())
        }
      } else {
        // 有工具：先添加第一个工具（转换类型），然后添加剩余工具
        let first_tool = &tools[0];
        let mut builder_simple = client.agent(&provider.model).max_tokens(4096).tool(first_tool.clone());
        for tool in &tools[1..] {
          builder_simple = builder_simple.tool(tool.clone());
        }
        if let Some(pre) = &preamble {
          Arc::new(builder_simple.preamble(pre).build())
        } else {
          Arc::new(builder_simple.build())
        }
      };

      let (prompt, history) = split_prompt_and_history(rig_messages);

      let stream = rig::agent::StreamingPromptRequest::<_, ()>::new(agent, prompt)
        .with_history(history)
        .multi_turn(10)
        .await;

      // 直接处理流式响应
      use tokio::select;
      let mut full_response = String::new();
      let mut stream = stream;
      // 存储 call_id -> 工具名的映射，用于 ToolResult 时获取工具名
      let mut tool_name_map: HashMap<String, String> = HashMap::new();
      // 存储 call_id -> 参数的映射，用于 ToolResult 时保留原始参数
      let mut tool_args_map: HashMap<String, serde_json::Value> = HashMap::new();

      loop {
        select! {
          _ = abort_rx.recv() => {
            let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
            return Ok(String::new());
          }
          item = stream.next() => {
            match item {
              Some(Ok(MultiTurnStreamItem::StreamAssistantItem(content))) => {
                match content {
                  // 处理流式文本回答
                  StreamedAssistantContent::Text(text) => {
                    full_response.push_str(&text.text);
                    let _ = tx.send(Step::Answer { content: text.text, done: false, timestamp: now() });
                  }
                  // 处理思考过程
                  StreamedAssistantContent::Reasoning(reasoning) => {
                    let reasoning_text = reasoning.reasoning.join("\n");
                    info!("🤔 Reasoning: {}", reasoning_text);
                    let info_id = format!("reasoning-{}", next_step_id());
                    let _ = tx.send(Step::Info {
                      id: info_id,
                      text: format!("思考中...\n{}", reasoning_text),
                      timestamp: now(),
                    });
                  }
                  // 处理工具调用 - 发送到 UI 显示
                  StreamedAssistantContent::ToolCall(tool_call) => {
                    info!("🔧 Tool call: {} (id: {})", tool_call.function.name, tool_call.id);
                    // 存储工具名映射：call_id -> 工具名
                    tool_name_map.insert(tool_call.id.clone(), tool_call.function.name.clone());
                    // 存储参数映射：call_id -> 参数
                    tool_args_map.insert(tool_call.id.clone(), tool_call.function.arguments.clone());
                    // Use tool_call.id (call_id) which matches tool_result.id
                    let tool_id = format!("tool-{}", tool_call.id);
                    let _ = tx.send(Step::Tool {
                      id: tool_id,
                      name: tool_call.function.name.clone(),
                      args: tool_call.function.arguments.clone(),
                      result: None,
                      status: ToolStatus::Pending,
                      timestamp: now(),
                    });
                  }
                  // 处理其他流式内容（ToolCallDelta, ReasoningDelta等）
                  _ => {}
                }
              }
              // 处理工具结果（当工具执行完成后）
              Some(Ok(MultiTurnStreamItem::StreamUserItem(content))) => {
                match content {
                  StreamedUserContent::ToolResult(tool_result) => {
                    info!("🔧 Tool result: {}", tool_result.id);
                    // tool_result.id 是 call_id，从映射中查找工具名和参数
                    let tool_name = tool_name_map.get(&tool_result.id)
                      .cloned()
                      .unwrap_or_else(|| tool_result.id.clone());
                    let tool_args = tool_args_map.get(&tool_result.id)
                      .cloned()
                      .unwrap_or_else(|| serde_json::json!({}));
                    let tool_id = format!("tool-{}", tool_result.id);

                    // Convert ToolResultContent to String
                    let result_str: String = match tool_result.content.iter().next() {
                      Some(ToolResultContent::Text(text_obj)) => text_obj.text.clone(),
                      Some(ToolResultContent::Image(_)) => "[Image]".to_string(),
                      None => String::new(),
                    };

                    let _ = tx.send(Step::Tool {
                      id: tool_id,
                      name: tool_name,
                      args: tool_args,
                      result: Some(result_str),
                      status: ToolStatus::Success,
                      timestamp: now(),
                    });
                  }
                }
              }
              Some(Ok(MultiTurnStreamItem::FinalResponse(_final_resp))) => {
                // All content has already been streamed via Text events
                // Just send the completion signal (done=true with empty content)
                let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
                return Ok(full_response);
              }
              // 处理其他可能的 MultiTurnStreamItem 变体（非穷尽枚举）
              Some(Ok(_)) => {}
              Some(Err(e)) => {
                let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
                return Err(AgentError::Ai(e.to_string()));
              }
              None => {
                let _ = tx.send(Step::Answer { content: String::new(), done: true, timestamp: now() });
                return Ok(full_response);
              }
            }
          }
        }
      }
    }
    Some(other) => {
      return Err(AgentError::Ai(format!("Unsupported provider: {}", other)));
    }
    None => {
      return Err(AgentError::Ai("No provider type".to_string()));
    }
  };
}

/// 简单聊天（无工具）
async fn stream_simple_chat(
  messages: Vec<crate::services::ChatMessage>,
  tx: mpsc::UnboundedSender<Step>,
  mut abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  use crate::services::AiClient;
  use tokio::select;

  let client = AiClient::new()?;
  let (preamble, prompt) = parse_simple_messages(messages);
  let (tx_response, mut rx_response) = mpsc::channel(64);

  // 启动 AI 请求
  tokio::spawn(async move {
    let _ = client.chat_simple(&prompt, preamble.as_deref(), tx_response).await;
  });

  let mut full = String::new();

  loop {
    select! {
      _ = abort_rx.recv() => {
        let _ = tx.send(Step::Answer {
          content: String::new(),
          done: true,
          timestamp: now(),
        });
        return Ok(String::new());
      }
      chunk_result = rx_response.recv() => {
        match chunk_result {
          Some(Ok(chunk)) => {
            full.push_str(&chunk);
            let _ = tx.send(Step::Answer {
              content: chunk,
              done: false,
              timestamp: now(),
            });
          }
          Some(Err(e)) => {
            let _ = tx.send(Step::Answer {
              content: String::new(),
              done: true,
              timestamp: now(),
            });
            return Err(AgentError::Ai(e.to_string()));
          }
          None => {
            let _ = tx.send(Step::Answer {
              content: String::new(),
              done: true,
              timestamp: now(),
            });
            return Ok(full);
          }
        }
      }
    }
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn build_tools_prompt(tools: &[McpToolBridge]) -> String {
  tools
    .iter()
    .map(|tool| {
      let schema = format_tool_schema(&tool.parameters);
      format!("**{}**: {}\n\n{}", tool.name, tool.description, schema)
    })
    .collect::<Vec<_>>()
    .join("\n\n")
}

fn format_tool_schema(schema: &serde_json::Value) -> String {
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

fn parse_messages(
  messages: Vec<crate::services::ChatMessage>,
  system_instructions: Option<String>,
) -> (Vec<rig::completion::Message>, Option<String>) {
  let mut rig_messages = Vec::new();
  let mut system_content = system_instructions.unwrap_or_default();
  let mut last_user_idx = None;
  let mut last_user_msg: Option<String> = None;

  // 找到最后一条用户消息
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
          // 保存最后一条用户消息，作为 prompt
          last_user_msg = Some(msg.content);
        } else {
          rig_messages.push(rig::completion::Message::user(msg.content));
        }
      }
      "assistant" => {
        rig_messages.push(rig::completion::Message::assistant(msg.content));
      }
      _ => {}
    }
  }

  // 如果有最后一条用户消息，添加到 rig_messages 末尾
  // 这样 split_prompt_and_history 就可以提取它
  if let Some(user_msg) = last_user_msg {
    rig_messages.push(rig::completion::Message::user(user_msg));
  }

  let preamble = if system_content.is_empty() {
    None
  } else {
    Some(system_content.trim().to_string())
  };

  info!("📝 parse_messages: {} rig_messages, preamble: {}", rig_messages.len(), preamble.is_some());
  (rig_messages, preamble)
}

fn parse_simple_messages(
  messages: Vec<crate::services::ChatMessage>,
) -> (Option<String>, String) {
  let mut system_content = String::new();
  let mut last_user_content = String::new();

  for msg in messages {
    match msg.role.as_str() {
      "system" => {
        system_content.push_str(&msg.content);
        system_content.push('\n');
      }
      "user" => {
        last_user_content = msg.content;
      }
      _ => {}
    }
  }

  let preamble = if system_content.is_empty() {
    None
  } else {
    Some(system_content.trim().to_string())
  };

  (preamble, last_user_content)
}

fn split_prompt_and_history(
  mut messages: Vec<rig::completion::Message>,
) -> (String, Vec<rig::completion::Message>) {
  if messages.is_empty() {
    warn!("split_prompt_and_history: messages is empty");
    return (String::new(), Vec::new());
  }

  // DEBUG: 记录输入消息
  info!("🔍 split_prompt_and_history: {} messages", messages.len());

  // 找到最后一条用户消息并提取文本内容
  // rig-core 的 Message::User { content } 中 content 是 OneOrMany<UserContent>
  // UserContent::Text(Text { text }) 包含实际的文本内容
  let (last_user_idx, prompt_content): (usize, String) = messages
    .iter()
    .enumerate()
    .rev()
    .find_map(|(idx, m)| {
      if let Message::User { content } = m {
        // 遍历 OneOrMany<UserContent> 查找文本内容
        for user_content in content.iter() {
          if let UserContent::Text(text_obj) = user_content {
            let text = text_obj.text.clone();
            if !text.is_empty() {
              info!("✅ Found user message at idx {}: '{}'", idx, text);
              return Some((idx, text));
            }
          }
        }
        warn!("⚠️  User message at idx {} has no text content", idx);
        None
      } else {
        None
      }
    })
    .unwrap_or_else(|| {
      warn!("⚠️  No user message found, using empty prompt");
      (messages.len(), String::new())
    });

  if !prompt_content.is_empty() {
    messages.remove(last_user_idx);
    info!("📤 Extracted prompt: '{}', history: {} messages", prompt_content, messages.len());
    (prompt_content, messages)
  } else {
    warn!("⚠️  Prompt is empty after extraction!");
    (String::new(), messages)
  }
}

/// Get current Unix timestamp in seconds
///
/// # WARNING: Do NOT use for unique ID generation!
///
/// `now()` returns **second-level precision** (as_secs()), which means:
/// - Multiple calls within the same second return the SAME value
/// - Using this for IDs causes duplicates when events happen quickly
/// - This has caused multiple bugs in regenerate/restart scenarios
///
/// ## Correct alternatives:
/// - For unique IDs: Use `next_step_id()` (atomic counter)
/// - For timestamps only: Use for `timestamp` fields where uniqueness is NOT required
///
/// ## History of bugs caused by this:
/// 1. Info step IDs like `conn-Context7-1767434311` appeared multiple times in history
/// 2. Regenerate found first duplicate instead of last, truncating from wrong position
/// 3. Required fixing `add_message` deduplication AND switching to atomic counters
fn now() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

// ============================================================================
// Backward Compatibility
// ============================================================================

/// 保留的旧接口（转发到新实现）
pub async fn chat_with_tools(
  messages: Vec<crate::services::ChatMessage>,
  tx: mpsc::UnboundedSender<Step>,
  abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  chat_with_mcp_tools(messages, tx, abort_rx).await
}
