//! Custom hooks for chat functionality
//! 聊天功能自定义 Hooks

use super::message_list::ChatMessage;
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use crate::config::QuickPrompt;
use crate::services::{chat_with_tools, AgentStep};
use dioxus::document;
use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use std::time::SystemTime;
use tokio::sync::mpsc;

impl From<HistoryMessage> for ChatMessage {
  fn from(msg: HistoryMessage) -> Self {
    ChatMessage {
      id: msg.id,
      role: msg.role,
      content: msg.content,
      timestamp: msg.timestamp,
    }
  }
}

impl From<ChatMessage> for HistoryMessage {
  fn from(msg: ChatMessage) -> Self {
    HistoryMessage {
      id: msg.id,
      role: msg.role,
      content: msg.content,
      timestamp: msg.timestamp,
    }
  }
}

/// Hook for the chat coroutine that handles AI calls and streaming responses
pub fn use_chat_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<String> {
  use_coroutine(move |mut rx: UnboundedReceiver<String>| {
    // Clone signals at the beginning
    let mut messages = messages.clone();
    let mut chat_history = chat_history.clone();
    let mut is_running = is_agent_running.clone();
    let mut msg_counter: u64 = 0;
    async move {
      while let Some(text) = rx.next().await {
        let text: String = text;

        // Mark agent as running
        is_running.set(true);

        // Add user message
        let now_millis = SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis();
        let now_secs = SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_secs();
        msg_counter += 1;
        let user_msg_id = format!("msg-{}-{}", now_millis, msg_counter);
        let user_msg = ChatMessage {
          id: user_msg_id.clone(),
          role: "user".to_string(),
          content: text.clone(),
          timestamp: now_secs,
        };
        messages.push(user_msg.clone());

        // Update history
        chat_history.write().add_message(HistoryMessage {
          id: user_msg_id.clone(),
          role: "user".to_string(),
          content: text.clone(),
          timestamp: now_secs,
        });
        let history_clone = (*chat_history.read()).clone();
        let _ = chat_history.read().save();
        // Trigger UI update for session list
        chat_history.set(history_clone);

        // Build message history for API (exclude system errors)
        let api_messages: Vec<crate::services::ChatMessage> = messages
          .read()
          .iter()
          .filter(|m| m.role != "system")
          .map(|m| crate::services::ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
          })
          .collect();

        // Create channel for streaming AgentStep updates
        let (step_tx, mut step_rx) = mpsc::unbounded_channel::<AgentStep>();

        // Create temporary assistant message ID for streaming updates
        let now_millis = SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis();
        let now_secs = SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_secs();
        msg_counter += 1;
        let assistant_msg_id = format!("msg-{}-{}", now_millis, msg_counter);

        // Add initial placeholder message
        messages.push(ChatMessage {
          id: assistant_msg_id.clone(),
          role: "assistant".to_string(),
          content: "思考中...".to_string(),
          timestamp: now_secs,
        });

        // Track intermediate steps and final answer
        // IMPORTANT: Don't update chat_history during agent execution to avoid
        // triggering the use_effect sync that overwrites our message updates
        let mut intermediate_steps = Vec::new();
        let mut final_response = String::new();

        eprintln!("=== STARTING AGENT TASK ===");

        // Spawn agent in background (but process steps in this coroutine context)
        let api_messages_clone = api_messages.clone();
        let step_tx_clone = step_tx.clone();
        tokio::spawn(async move {
          if let Err(e) = chat_with_tools(api_messages_clone, step_tx_clone).await {
            eprintln!("[HOOKS] Agent error: {:?}", e);
          }
        });

        // Process steps as they arrive
        eprintln!("=== ENTERING STEP LOOP ===");
        let mut step_count = 0;

        while let Some(step) = step_rx.recv().await {
          step_count += 1;
          eprintln!("=== PROCESSING STEP {} ===", step_count);

          // ALERT: STEP FORMAT - If you modify these formats, update the placeholder detection
          // in the message sync effect to match! Otherwise steps will flicker/disappear.
          match step {
            AgentStep::Connecting(msg) => {
              intermediate_steps.push(format!("• {}", msg));
            }
            AgentStep::Thinking { short, content } => {
              // Format thinking content: show text directly, filter out JSON
              if let Some(thought_content) = content {
                let formatted = format_thinking_content(&thought_content);
                if !formatted.trim().is_empty() {
                  intermediate_steps.push(formatted);
                }
              } else if !short.is_empty() {
                intermediate_steps.push(format!("• {}", short));
              }
            }
            AgentStep::ToolCall { name, .. } => {
              // Show loading state
              intermediate_steps.push(format!("• ⏳ 调用 {}", name));
            }
            AgentStep::ToolResult { name, .. } => {
              // Find and update the last loading step to completed
              if let Some(pos) = intermediate_steps.iter().rposition(|s| s.contains("⏳")) {
                intermediate_steps[pos] = format!("• ✓ 调用 {}", name);
              } else {
                intermediate_steps.push(format!("• ✓ 调用 {}", name));
              }
            }
            AgentStep::Final(text) => {
              final_response = text.clone();
              // NOTE: Don't update chat_history yet - do it after the loop
            }
          }

          // When final response arrives, create a separate message bubble for it
          if !final_response.is_empty() {
            eprintln!("=== FINAL RESPONSE RECEIVED, CREATING NEW MESSAGE ===");

            // First, update the steps message to show completion
            let current_msgs = messages.read().clone();
            if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
              let mut updated = current_msgs;
              updated[pos].content = intermediate_steps.join("\n");
              messages.set(updated);
            }

            // Then, create a new message for the final response
            let now_millis_final = SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)
              .unwrap()
              .as_millis();
            let now_secs_final = SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)
              .unwrap()
              .as_secs();
            msg_counter += 1;
            let final_msg_id = format!("msg-{}-{}", now_millis_final, msg_counter);

            messages.push(ChatMessage {
              id: final_msg_id.clone(),
              role: "assistant".to_string(),
              content: final_response.clone(),
              timestamp: now_secs_final,
            });

            // Save both messages to history
            chat_history.write().add_message(HistoryMessage {
              id: assistant_msg_id.clone(),
              role: "assistant".to_string(),
              content: intermediate_steps.join("\n"),
              timestamp: now_secs,
            });
            chat_history.write().add_message(HistoryMessage {
              id: final_msg_id.clone(),
              role: "assistant".to_string(),
              content: final_response.clone(),
              timestamp: now_secs_final,
            });
            let history_clone = (*chat_history.read()).clone();
            let _ = chat_history.read().save();
            chat_history.set(history_clone);

            // Mark agent as no longer running
            is_running.set(false);

            // Break out of the loop since we're done
            break;
          }

          // Update the steps message (still in progress)
          let display_content = intermediate_steps.join("\n");
          eprintln!(
            "=== UPDATING SIGNAL (step {}, content length: {}) ===",
            step_count,
            display_content.len()
          );

          let current_msgs = messages.read().clone();
          if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
            let mut updated = current_msgs;
            updated[pos].content = display_content;
            updated[pos].timestamp = now_secs;
            messages.set(updated);
            eprintln!("=== MESSAGE UPDATED (via set) ===");
          } else {
            eprintln!("=== WARNING: MESSAGE NOT FOUND IN LIST ===");
            drop(current_msgs);
            messages.push(ChatMessage {
              id: assistant_msg_id.clone(),
              role: "assistant".to_string(),
              content: display_content,
              timestamp: now_secs,
            });
          }
        }
        eprintln!("=== STEP LOOP DONE ===");

        // Safety: Reset running state if loop ends without Final response (error case)
        is_running.set(false);
      }
    }
  })
}

/// Hook for chat coroutine with quick prompt support
/// Accepts (text, Option<prompt>) tuples and injects the prompt prefix as a system message
pub fn use_chat_coroutine_with_prefix(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<(String, Option<QuickPrompt>)> {
  use_coroutine(
    move |mut rx: UnboundedReceiver<(String, Option<QuickPrompt>)>| {
      // Clone signals at the beginning
      let mut messages = messages.clone();
      let mut chat_history = chat_history.clone();
      let mut is_running = is_agent_running.clone();
      let mut msg_counter: u64 = 0;
      async move {
        while let Some((text, prompt)) = rx.next().await {
          let (text, prompt): (String, Option<QuickPrompt>) = (text, prompt);

          // Mark agent as running
          is_running.set(true);

          // If a quick prompt is selected, inject system message first
          if let Some(_) = prompt {
            let now_secs = SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)
              .unwrap()
              .as_secs();
            msg_counter += 1;
            let _system_msg_id = format!("sys-{}-{}", now_secs, msg_counter);

            // Add system message with the prompt prefix (not visible in UI, but sent to API)
            // We'll inject this in the API call, not in the UI messages
          }

          // Add user message
          let now_millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
          let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
          msg_counter += 1;
          let user_msg_id = format!("msg-{}-{}", now_millis, msg_counter);

          // Prepend prompt prefix to user content if prompt is selected
          let display_content = if let Some(ref p) = prompt {
            format!("{}{}", p.prefix, text)
          } else {
            text.clone()
          };

          let user_msg = ChatMessage {
            id: user_msg_id.clone(),
            role: "user".to_string(),
            content: display_content.clone(),
            timestamp: now_secs,
          };
          messages.push(user_msg.clone());

          // Update history
          chat_history.write().add_message(HistoryMessage {
            id: user_msg_id.clone(),
            role: "user".to_string(),
            content: display_content,
            timestamp: now_secs,
          });
          let history_clone = (*chat_history.read()).clone();
          let _ = chat_history.read().save();
          chat_history.set(history_clone);

          // Build message history for API (exclude system errors)
          let api_messages: Vec<crate::services::ChatMessage> = messages
            .read()
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| crate::services::ChatMessage {
              role: m.role.clone(),
              content: m.content.clone(),
            })
            .collect();

          // Create channel for streaming AgentStep updates
          let (step_tx, mut step_rx) = mpsc::unbounded_channel::<AgentStep>();

          // Create temporary assistant message ID for streaming updates
          let now_millis = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
          let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
          msg_counter += 1;
          let assistant_msg_id = format!("msg-{}-{}", now_millis, msg_counter);

          // Add initial placeholder message
          messages.push(ChatMessage {
            id: assistant_msg_id.clone(),
            role: "assistant".to_string(),
            content: "思考中...".to_string(),
            timestamp: now_secs,
          });

          // Track intermediate steps and final answer
          let mut intermediate_steps = Vec::new();
          let mut final_response = String::new();

          eprintln!("=== STARTING AGENT TASK ===");

          // Spawn agent in background
          let api_messages_clone = api_messages.clone();
          let step_tx_clone = step_tx.clone();
          tokio::spawn(async move {
            if let Err(e) = chat_with_tools(api_messages_clone, step_tx_clone).await {
              eprintln!("[HOOKS] Agent error: {:?}", e);
            }
          });

          // Process steps as they arrive
          eprintln!("=== ENTERING STEP LOOP ===");
          let mut step_count = 0;

          while let Some(step) = step_rx.recv().await {
            step_count += 1;
            eprintln!("=== PROCESSING STEP {} ===", step_count);

            match step {
              AgentStep::Connecting(msg) => {
                intermediate_steps.push(format!("• {}", msg));
              }
              AgentStep::Thinking { short, content } => {
                if let Some(thought_content) = content {
                  let formatted = format_thinking_content(&thought_content);
                  if !formatted.trim().is_empty() {
                    intermediate_steps.push(formatted);
                  }
                } else if !short.is_empty() {
                  intermediate_steps.push(format!("• {}", short));
                }
              }
              AgentStep::ToolCall { name, .. } => {
                intermediate_steps.push(format!("• ⏳ 调用 {}", name));
              }
              AgentStep::ToolResult { name, .. } => {
                if let Some(pos) = intermediate_steps.iter().rposition(|s| s.contains("⏳")) {
                  intermediate_steps[pos] = format!("• ✓ 调用 {}", name);
                } else {
                  intermediate_steps.push(format!("• ✓ 调用 {}", name));
                }
              }
              AgentStep::Final(text) => {
                final_response = text.clone();
              }
            }

            // When final response arrives
            if !final_response.is_empty() {
              eprintln!("=== FINAL RESPONSE RECEIVED, CREATING NEW MESSAGE ===");

              let current_msgs = messages.read().clone();
              if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
                let mut updated = current_msgs;
                updated[pos].content = intermediate_steps.join("\n");
                messages.set(updated);
              }

              let now_millis_final = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
              let now_secs_final = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
              msg_counter += 1;
              let final_msg_id = format!("msg-{}-{}", now_millis_final, msg_counter);

              messages.push(ChatMessage {
                id: final_msg_id.clone(),
                role: "assistant".to_string(),
                content: final_response.clone(),
                timestamp: now_secs_final,
              });

              chat_history.write().add_message(HistoryMessage {
                id: assistant_msg_id.clone(),
                role: "assistant".to_string(),
                content: intermediate_steps.join("\n"),
                timestamp: now_secs,
              });
              chat_history.write().add_message(HistoryMessage {
                id: final_msg_id.clone(),
                role: "assistant".to_string(),
                content: final_response.clone(),
                timestamp: now_secs_final,
              });
              let history_clone = (*chat_history.read()).clone();
              let _ = chat_history.read().save();
              chat_history.set(history_clone);

              is_running.set(false);
              break;
            }

            let display_content = intermediate_steps.join("\n");
            eprintln!(
              "=== UPDATING SIGNAL (step {}, content length: {}) ===",
              step_count,
              display_content.len()
            );

            let current_msgs = messages.read().clone();
            if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
              let mut updated = current_msgs;
              updated[pos].content = display_content;
              updated[pos].timestamp = now_secs;
              messages.set(updated);
              eprintln!("=== MESSAGE UPDATED (via set) ===");
            } else {
              eprintln!("=== WARNING: MESSAGE NOT FOUND IN LIST ===");
              drop(current_msgs);
              messages.push(ChatMessage {
                id: assistant_msg_id.clone(),
                role: "assistant".to_string(),
                content: display_content,
                timestamp: now_secs,
              });
            }
          }
          eprintln!("=== STEP LOOP DONE ===");

          is_running.set(false);
        }
      }
    },
  )
}

/// Format thinking content: show text directly, filter out JSON tool calls
fn format_thinking_content(content: &str) -> String {
  let lines: Vec<&str> = content.lines().collect();
  let mut result = Vec::new();
  let mut in_json = false;
  let mut json_lines = Vec::new();

  for line in lines {
    let trimmed = line.trim();

    // Detect JSON start
    if trimmed.contains("{\"tool_call\"")
      || trimmed.contains("\"tool_call\"")
      || (trimmed.starts_with("{")
        && (trimmed.contains("\"name\"") || trimmed.contains("\"arguments\"")))
    {
      in_json = true;
      json_lines.push(line);
      continue;
    }

    if in_json {
      json_lines.push(line);
      if trimmed.contains("}") {
        in_json = false;
        // Log filtered JSON for debugging
        eprintln!("[FILTERED JSON TOOL CALL] {}", json_lines.join("\n"));
        json_lines.clear();
      }
      continue;
    }

    if !trimmed.is_empty() {
      result.push(line.to_string());
    }
  }

  // Handle unclosed JSON (shouldn't happen, but log if it does)
  if !json_lines.is_empty() {
    eprintln!(
      "[FILTERED JSON TOOL CALL - UNCLOSSED] {}",
      json_lines.join("\n")
    );
  }

  format!("• {}", result.join("\n"))
}

/// Hook for message sync with chat history
///
/// Syncs messages with current session. Skips sync if:
/// - Session has no messages (prevents overwriting user input)
/// - Agent is in progress (has placeholder markers)
pub fn use_message_sync(
  mut messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) {
  // Track both messages and chat_history explicitly
  let messages_dep = messages.clone();
  let chat_history_dep = chat_history.clone();
  let is_running_dep = is_agent_running.clone();

  use_effect(move || {
    // Explicitly read all signals to track dependencies
    let _ = messages_dep();
    let _ = chat_history_dep();
    let is_running = is_running_dep();

    if let Some(session) = chat_history().get_current_session() {
      // Only sync if the session has messages
      // This prevents overwriting user input when switching to an empty session
      if session.messages.is_empty() {
        return;
      }

      let current_msgs: Vec<ChatMessage> =
        session.messages.iter().cloned().map(Into::into).collect();

      // Only sync if there's no in-progress agent (using explicit flag instead of content detection)
      if !is_running && messages() != current_msgs {
        messages.set(current_msgs);
      }
    }
  });
}

/// Hook for auto-scroll to bottom when new messages arrive
pub fn use_auto_scroll(
  messages: Signal<Vec<ChatMessage>>,
  mut last_message_count: Signal<usize>,
  scroll_container_id: String,
) {
  use_effect(move || {
    let current_count = messages().len();
    let prev_count = last_message_count();

    // Update last message count
    if current_count != prev_count {
      last_message_count.set(current_count);
    }

    // Only scroll if new messages were added
    if current_count > prev_count {
      let container_id = scroll_container_id.clone();
      document::eval(
        format!(
          r#"(function() {{
                    const container = document.getElementById("{}");
                    if (!container) return;

                    // Check if we should auto-scroll (only if currently in auto mode)
                    if (window.__veldScrollState === 'auto') {{
                        container.scrollTo({{ top: container.scrollHeight, behavior: "smooth" }});
                    }}
                }})()"#,
          container_id
        )
        .as_str(),
      );
    }
  });
}

/// Hook for initializing scroll state tracking
pub fn use_scroll_state_init(scroll_container_id: String) {
  use_effect(move || {
    let container_id = scroll_container_id.clone();
    document::eval(
      format!(
        r#"(function() {{
                const container = document.getElementById("{}");
                if (!container) return;

                // Initialize scroll state
                window.__veldScrollState = 'auto'; // 'auto' or 'manual'

                container.addEventListener('scroll', () => {{
                    const scrollTop = container.scrollTop;
                    const scrollHeight = container.scrollHeight;
                    const clientHeight = container.clientHeight;
                    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

                    // If user scrolled up more than 150px from bottom, switch to manual mode
                    if (distanceFromBottom > 150) {{
                        window.__veldScrollState = 'manual';
                    }} else if (distanceFromBottom < 50) {{
                        // User scrolled back near bottom, switch back to auto mode
                        window.__veldScrollState = 'auto';
                    }}
                }}, {{ passive: true }});
            }})()"#,
        container_id
      )
      .as_str(),
    );
  });
}
