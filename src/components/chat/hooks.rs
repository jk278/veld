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
use tokio::sync::{mpsc, Mutex, broadcast};

/// Global abort sender for interrupting streaming responses
/// Use get_abort_sender() to get the current sender, and set_abort_sender() to update it
static CURRENT_ABORT_SENDER: Mutex<Option<broadcast::Sender<()>>> = Mutex::const_new(None);

/// Set the current abort sender (called internally by hooks)
async fn set_abort_sender(sender: broadcast::Sender<()>) {
  let mut current = CURRENT_ABORT_SENDER.lock().await;
  *current = Some(sender);
}

/// Get the current abort sender (call this to abort streaming)
pub async fn get_abort_sender() -> Option<broadcast::Sender<()>> {
  let current = CURRENT_ABORT_SENDER.lock().await;
  current.clone()
}

/// Abort the current streaming response
pub async fn abort_streaming() {
  if let Some(sender) = get_abort_sender().await {
    let _ = sender.send(());
  }
}

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

/// Internal agent execution logic - shared by both new chat and regeneration
async fn run_agent(
  api_messages: Vec<crate::services::ChatMessage>,
  mut messages: Signal<Vec<ChatMessage>>,
  mut chat_history: Signal<ChatHistoryData>,
  mut is_running: Signal<bool>,
  msg_counter: &mut u64,
) {
  // Mark agent as running
  is_running.set(true);

  // Create channel for streaming AgentStep updates
  let (step_tx, mut step_rx) = mpsc::unbounded_channel::<AgentStep>();

  // Create abort channel for interrupting streaming
  let (abort_tx, abort_rx) = tokio::sync::broadcast::channel::<()>(1);
  // Store globally for UI access
  tokio::spawn(async move {
    set_abort_sender(abort_tx).await;
  });

  // Create temporary assistant message ID for streaming updates
  let now_millis = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let now_secs = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs();
  *msg_counter += 1;
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
  let mut stream_complete = false;
  let mut is_streaming = false;
  let mut update_counter = 0usize;

  eprintln!("=== STARTING AGENT TASK ===");

  // Spawn agent in background
  let api_messages_clone = api_messages.clone();
  let step_tx_clone = step_tx.clone();
  tokio::spawn(async move {
    if let Err(e) = chat_with_tools(api_messages_clone, step_tx_clone, abort_rx).await {
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
      AgentStep::Chunk(chunk) => {
        if !is_streaming {
          is_streaming = true;
          // Don't clear intermediate_steps - preserve them for the final answer
        }
        final_response.push_str(&chunk);
      }
      AgentStep::Final => {
        stream_complete = true;
      }
    }

    // When stream completes, save final message
    if stream_complete {
      if !final_response.is_empty() {
        eprintln!("=== STREAM COMPLETE, SAVING MESSAGES ===");

        let current_msgs = messages.read().clone();
        if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
          // First, update the intermediate steps message
          let mut updated = current_msgs;
          updated[pos].content = intermediate_steps.join("\n");
          messages.set(updated);

          // Then, create a new message for the final response
          let now_millis_final = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
          let now_secs_final = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
          *msg_counter += 1;
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

          is_running.set(false);
          break;
        }
      } else {
        // Aborted: remove the placeholder "思考中..." message
        eprintln!("=== STREAM ABORTED, REMOVING PLACEHOLDER ===");
        let current_msgs = messages.read().clone();
        if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
          let mut updated = current_msgs;
          updated.remove(pos);
          messages.set(updated);
        }
        is_running.set(false);
        break;
      }
    }

    // Update UI in real-time (throttled)
    if !stream_complete {
      update_counter += 1;
      let should_update = if is_streaming {
        update_counter % 5 == 0
      } else {
        true
      };

      if should_update {
        let display_content = if is_streaming {
          final_response.clone()
        } else {
          intermediate_steps.join("\n")
        };

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
        } else {
          drop(current_msgs);
          messages.push(ChatMessage {
            id: assistant_msg_id.clone(),
            role: "assistant".to_string(),
            content: display_content,
            timestamp: now_secs,
          });
        }
      }
    }
  }
  eprintln!("=== STEP LOOP DONE ===");
  is_running.set(false);
}

/// Hook for the chat coroutine that handles AI calls and streaming responses
/// Stores abort sender globally - call abort_streaming() to stop
pub fn use_chat_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<String> {
  use_coroutine(move |mut rx: UnboundedReceiver<String>| {
    let mut messages = messages.clone();
    let mut chat_history = chat_history.clone();
    let mut is_running = is_agent_running.clone();
    let mut msg_counter: u64 = 0;
    async move {
      while let Some(text) = rx.next().await {
        let text: String = text;

        // Add user message
        let now_secs = SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_secs();
        msg_counter += 1;
        let user_msg_id = format!("msg-{}-{}", now_secs, msg_counter);
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

        // Run agent
        run_agent(api_messages, messages, chat_history, is_running, &mut msg_counter).await;
      }
    }
  })
}

/// Hook for regeneration from a specific message
pub fn use_regenerate_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<(String, String)> {
  use_coroutine(move |mut rx: UnboundedReceiver<(String, String)>| {
    let mut messages = messages.clone();
    let mut chat_history = chat_history.clone();
    let mut is_running = is_agent_running.clone();
    let mut msg_counter: u64 = 0;
    async move {
      while let Some((message_id, new_content)) = rx.next().await {
        let (message_id, new_content) = (message_id, new_content);

        // Update the message
        chat_history.write().update_message(&message_id, new_content.clone());

        // Get index and truncate everything after
        let idx = chat_history.read().get_message_index(&message_id);
        if let Some(index) = idx {
          chat_history.write().truncate_from_index(index + 1);
        }

        // Save and sync
        let history_clone = (*chat_history.read()).clone();
        let _ = chat_history.read().save();
        chat_history.set(history_clone);

        // Immediately sync UI to remove old messages before starting generation
        let current_msgs: Vec<ChatMessage> = chat_history
          .read()
          .get_current_session()
          .map(|s| s.messages.iter().cloned().map(Into::into).collect())
          .unwrap_or_default();
        messages.set(current_msgs);

        // Build API messages from truncated history
        let api_messages: Vec<crate::services::ChatMessage> = chat_history
          .read()
          .get_current_session()
          .map(|s| {
            s.messages
              .iter()
              .filter(|m| m.role != "system")
              .map(|m| crate::services::ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
              })
              .collect()
          })
          .unwrap_or_default();

        // Run agent
        run_agent(api_messages, messages, chat_history, is_running, &mut msg_counter).await;
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

          // Create abort channel for interrupting streaming
          let (abort_tx, abort_rx) = tokio::sync::broadcast::channel::<()>(1);
          // Store globally for UI access
          tokio::spawn(async move {
            set_abort_sender(abort_tx).await;
          });

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
          let mut stream_complete = false;
          let mut is_streaming = false; // Track if we're in streaming mode
          let mut update_counter = 0usize; // Throttle UI updates

          eprintln!("=== STARTING AGENT TASK ===");

          // Spawn agent in background
          let api_messages_clone = api_messages.clone();
          let step_tx_clone = step_tx.clone();
          tokio::spawn(async move {
            if let Err(e) = chat_with_tools(api_messages_clone, step_tx_clone, abort_rx).await {
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
              AgentStep::Chunk(chunk) => {
                // Start streaming mode - clear intermediate steps and show response
                if !is_streaming {
                  is_streaming = true;
                  intermediate_steps.clear();
                }
                final_response.push_str(&chunk);
              }
              AgentStep::Final => {
                stream_complete = true;
              }
            }

            // When stream completes, save final message
            if stream_complete && !final_response.is_empty() {
              eprintln!("=== STREAM COMPLETE, SAVING FINAL MESSAGE ===");

              let current_msgs = messages.read().clone();
              if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
                let mut updated = current_msgs;
                updated[pos].content = final_response.clone();
                messages.set(updated);

                chat_history.write().add_message(HistoryMessage {
                  id: assistant_msg_id.clone(),
                  role: "assistant".to_string(),
                  content: final_response.clone(),
                  timestamp: now_secs,
                });
                let history_clone = (*chat_history.read()).clone();
                let _ = chat_history.read().save();
                chat_history.set(history_clone);

                is_running.set(false);
                break;
              }
            }

            // Update UI in real-time (throttled to avoid excessive re-renders)
            if !stream_complete {
              update_counter += 1;

              // Only update UI every 5 chunks during streaming to reduce re-renders
              let should_update = if is_streaming {
                update_counter % 5 == 0
              } else {
                true // Always update for tool steps
              };

              if should_update {
                let display_content = if is_streaming {
                // Show streaming response
                final_response.clone()
              } else {
                // Show intermediate steps
                intermediate_steps.join("\n")
              };

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
              } else {
                drop(current_msgs);
                messages.push(ChatMessage {
                  id: assistant_msg_id.clone(),
                  role: "assistant".to_string(),
                  content: display_content,
                  timestamp: now_secs,
                });
              }
              }
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
  let mut result = Vec::new();
  let mut in_tool_call = false;
  let mut brace_depth = 0;

  for line in content.lines() {
    if in_tool_call {
      // Continue processing multi-line JSON
      for c in line.chars() {
        if c == '{' {
          brace_depth += 1;
        } else if c == '}' {
          brace_depth -= 1;
          if brace_depth == 0 {
            in_tool_call = false;
            eprintln!("[FILTERED TOOL CALL END]");
            break;
          }
        }
      }
      continue; // Skip entire line while in tool call
    }

    // Check for tool_call pattern
    if let Some(start_idx) = line.find("{\"tool_call\"") {
      // Count braces to find where JSON ends
      let mut depth = 0;
      let mut found_end = false;
      let chars: Vec<char> = line.chars().collect();

      for i in start_idx..chars.len() {
        if chars[i] == '{' {
          depth += 1;
        } else if chars[i] == '}' {
          depth -= 1;
          if depth == 0 {
            found_end = true;
            eprintln!("[FILTERED TOOL CALL (single line)]");
            // Keep content before the tool call
            if start_idx > 0 {
              let before = line[..start_idx].trim();
              if !before.is_empty() {
                result.push(before.to_string());
              }
            }
            break;
          }
        }
      }

      if !found_end {
        // Multi-line JSON, start filtering
        in_tool_call = true;
        brace_depth = 1;
        // Keep content before the tool call
        if start_idx > 0 {
          let before = line[..start_idx].trim();
          if !before.is_empty() {
            result.push(before.to_string());
          }
        }
      }
      continue;
    }

    // Normal line
    if !line.trim().is_empty() {
      result.push(line.to_string());
    }
  }

  // Handle unclosed tool call
  if in_tool_call {
    eprintln!("[WARNING] Unclosed tool call JSON at end of content");
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
