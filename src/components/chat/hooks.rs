//! Custom hooks for chat functionality
//! 聊天功能自定义 Hooks

use dioxus::prelude::*;
use dioxus::document;
use crate::services::{chat_with_tools, AgentStep};
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use super::message_list::ChatMessage;
use std::time::SystemTime;
use futures_util::stream::StreamExt;
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
///
/// # IMPORTANT: Agent Step Detection
/// This hook contains agent step detection logic that must stay in sync with
/// the placeholder detection in the message sync effect. If you modify the
/// step format (emojis/prefixes), you MUST update BOTH places.
pub fn use_chat_coroutine(
    messages: Signal<Vec<ChatMessage>>,
    chat_history: Signal<ChatHistoryData>,
) -> Coroutine<String> {
    use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        let mut messages = messages.clone();
        let mut chat_history = chat_history.clone();
        let mut msg_counter: u64 = 0;
        async move {
            while let Some(text) = rx.next().await {
                let text: String = text;

                // Add user message
                let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
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
                let history_clone = { (*chat_history.read()).clone() };
                let _ = chat_history.read().save();
                // Trigger UI update for session list
                chat_history.set(history_clone);

                // Build message history for API (exclude system errors)
                let api_messages: Vec<crate::services::ChatMessage> = messages.read().iter().filter(|m| m.role != "system").map(|m| {
                    crate::services::ChatMessage {
                        role: m.role.clone(),
                        content: m.content.clone(),
                    }
                }).collect();

                // Create channel for streaming AgentStep updates
                let (step_tx, mut step_rx) = mpsc::unbounded_channel::<AgentStep>();

                // Create temporary assistant message ID for streaming updates
                let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
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
                let api_messages_clone = api_messages.clone();

                // Spawn agent in background (but process steps in this coroutine context)
                tokio::spawn(async move {
                    let _ = chat_with_tools(api_messages_clone, step_tx).await;
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
                            intermediate_steps.push(format!("- 🔌 {}", msg));
                        }
                        AgentStep::Thinking { short, content } => {
                            // Use collapsible details if there's content, otherwise just show short text
                            if let Some(thought_content) = content {
                                intermediate_steps.push(format!("- 🤔 {}\n<details><summary>查看思考内容</summary>\n{}\n</details>", short, thought_content));
                            } else {
                                intermediate_steps.push(format!("- 🤔 {}", short));
                            }
                        }
                        AgentStep::ToolCall { name, .. } => {
                            intermediate_steps.push(format!("- 🔧 调用: {}", name));
                        }
                        AgentStep::ToolResult { name, .. } => {
                            intermediate_steps.push(format!("- ✅ 完成: {}", name));
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
                        let now_millis_final = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                        let now_secs_final = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
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
                        let history_clone = { (*chat_history.read()).clone() };
                        let _ = chat_history.read().save();
                        chat_history.set(history_clone);

                        // Break out of the loop since we're done
                        break;
                    }

                    // Update the steps message (still in progress)
                    let display_content = intermediate_steps.join("\n");
                    eprintln!("=== UPDATING SIGNAL (step {}, content length: {}) ===", step_count, display_content.len());

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
            }
        }
    })
}

/// Hook for message sync with chat history
///
/// # CRITICAL: Agent Step Detection
/// This effect syncs messages with the current session. It checks for unsaved
/// placeholder messages (agent in progress) to avoid overwriting in-progress
/// agent updates.
///
/// ALERT: If you modify the step format (emojis/prefixes), you MUST update BOTH places:
/// 1. The placeholder detection here (has_unsaved_placeholder check)
/// 2. The step formatting in use_chat_coroutine (around line 125-140)
///
/// Otherwise, use_effect will overwrite in-progress agent updates, causing steps to flicker/disappear.
pub fn use_message_sync(
    mut messages: Signal<Vec<ChatMessage>>,
    chat_history: Signal<ChatHistoryData>,
) {
    use_effect(move || {
        let _ = chat_history(); // Track chat_history dependency
        if let Some(session) = chat_history().get_current_session() {
            let current_msgs: Vec<ChatMessage> = session.messages.iter().cloned().map(Into::into).collect();

            // Check if messages has an unsaved placeholder (agent in progress)
            // Format: "- 🔌", "- 🤔", "- 🔧", "- ✅"
            let has_unsaved_placeholder = messages().iter().any(|m| {
                m.content == "思考中..." ||
                m.content.contains("- 🔌") ||
                m.content.contains("- 🤔") ||
                m.content.contains("- 🔧") ||
                m.content.contains("- ✅")
            });

            // Only sync if there's no in-progress agent
            if !has_unsaved_placeholder && messages() != current_msgs {
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
            document::eval(format!(
                r#"(function() {{
                    const container = document.getElementById("{}");
                    if (!container) return;

                    // Check if we should auto-scroll (only if currently in auto mode)
                    if (window.__veldScrollState === 'auto') {{
                        container.scrollTo({{ top: container.scrollHeight, behavior: "smooth" }});
                    }}
                }})()"#,
                container_id
            ).as_str());
        }
    });
}

/// Hook for initializing scroll state tracking
pub fn use_scroll_state_init(scroll_container_id: String) {
    use_effect(move || {
        let container_id = scroll_container_id.clone();
        document::eval(format!(
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
        ).as_str());
    });
}
