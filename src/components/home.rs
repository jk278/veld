//! Home page - Chat interface with session history
//! 首页 - AI 聊天对话界面（带历史会话）

use crate::chat_history::ChatHistoryData;
use crate::components::chat::message_list::ChatMessage;
use crate::components::chat::*;
use crate::config::AppConfig;
use crate::hooks::{RESPONSIVE_BREAKPOINT, use_window_size};
use crate::theme::use_theme;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex, OnceLock};

// Global trigger for activating input (shared with main.rs)
pub static ACTIVATE_INPUT_TRIGGER: OnceLock<Arc<Mutex<u64>>> = OnceLock::new();

// Re-export for use in other modules
pub use crate::components::chat::UiSession;

/// Home page component - Chat interface with sidebar
#[component]
pub fn Home() -> Element {
  let _theme_mode = use_theme();
  let window_width = use_window_size();

  // Chat messages state
  let messages = use_signal(Vec::<ChatMessage>::new);
  let input_text = use_signal(String::new);

  // Auto-scroll state
  let scroll_container_id = "chat-messages-container";
  let last_message_count = use_signal(|| 0);

  // Sidebar collapse state (persisted to config file)
  // NOTE: This is user-controlled state, NOT auto-collapsed on window resize
  // Default to collapsed to match CSS default and avoid flash
  let mut sidebar_collapsed = use_signal(|| {
    AppConfig::load()
      .map(|c| c.ui.sidebar_collapsed)
      .unwrap_or(true)
  });

  // Agent running state (prevents sync conflicts during execution)
  let is_agent_running = use_signal(|| false);

  // Persist sidebar state to config when changed
  use_effect(move || {
    let collapsed = sidebar_collapsed();
    if let Ok(mut config) = AppConfig::load() {
      config.update_sidebar_collapsed(collapsed);
    }
  });

  // Initialize scroll state tracking
  use_scroll_state_init(scroll_container_id.to_string());

  // Session history state
  let chat_history = use_signal(|| ChatHistoryData::load().unwrap_or_default());

  // Session list for sidebar (derived from history) - use_memo for auto-update
  let sessions = use_memo(move || {
    let history = chat_history();
    history
      .sessions
      .iter()
      .map(|s| UiSession {
        id: s.id.clone(),
        title: s.title.clone(),
        is_current: history.current_session_id.as_ref() == Some(&s.id),
      })
      .collect::<Vec<_>>()
  });

  // Active provider and MCP server (cached, updated on switch)
  let active_provider_id = use_signal(|| {
    AppConfig::load()
      .ok()
      .and_then(|c| c.ai.active_provider)
      .unwrap_or_else(|| "claude".to_string())
  });

  // Sync messages with current session
  use_message_sync(
    messages.clone(),
    chat_history.clone(),
    is_agent_running.clone(),
  );

  // Auto-scroll to bottom when new messages arrive
  use_auto_scroll(
    messages.clone(),
    last_message_count.clone(),
    scroll_container_id.to_string(),
  );

  // Auto-scroll during streaming response
  use_streaming_scroll(
    messages.clone(),
    scroll_container_id.to_string(),
    is_agent_running.clone(),
  );

  // Chat coroutine for AI calls
  let tx = use_chat_coroutine(messages.clone(), chat_history.clone(), is_agent_running);
  let tx_with_prefix =
    use_chat_coroutine_with_prefix(messages.clone(), chat_history.clone(), is_agent_running);

  // Regenerate coroutine for editing messages
  let tx_regenerate =
    use_regenerate_coroutine(messages.clone(), chat_history.clone(), is_agent_running);

  // Edit state for messages
  let mut edit_state = use_signal(|| Option::<crate::components::chat::message_list::MessageEdit>::None);

  // Filter enabled providers and MCP servers (reactive - updates on config change)
  let enabled_providers = use_signal(|| {
    AppConfig::load()
      .ok()
      .map(|c| {
        c.ai
          .providers
          .iter()
          .filter(|p| p.enabled)
          .cloned()
          .collect::<Vec<_>>()
      })
      .unwrap_or_default()
  });
  let enabled_mcp_servers = use_signal(|| {
    AppConfig::load()
      .ok()
      .map(|c| {
        c.mcp
          .servers
          .iter()
          .filter(|s| s.enabled)
          .cloned()
          .collect::<Vec<_>>()
      })
      .unwrap_or_default()
  });

  // Create handlers
  let new_chat_handler = use_new_chat_handler(
    chat_history.clone(),
    messages.clone(),
    active_provider_id.clone(),
  );

  let switch_session = use_switch_session_handler(chat_history.clone(), messages.clone());

  let delete_session = use_delete_session_handler(chat_history.clone());

  let switch_provider = use_switch_provider_handler(
    active_provider_id.clone(),
    enabled_providers.clone(),
    enabled_mcp_servers.clone(),
  );

  let send_message_handler = use_send_message_handler(input_text.clone(), tx.clone());

  // Wrapper handlers for EventHandler compatibility (create closures that clone the handler)
  let new_chat_for_sidebar = {
    let mut handler = new_chat_handler.clone();
    move |_: MouseEvent| handler()
  };
  let new_chat_for_header = {
    let mut handler = new_chat_handler.clone();
    move |_: MouseEvent| handler()
  };
  let send_message = {
    let mut handler = send_message_handler.clone();
    move |_: MouseEvent| handler()
  };

  // Get current provider info for rendering
  let (_active_provider_name, has_api_key) = get_active_provider_info();

  // Get current session title - use_memo for auto-update when session changes
  let current_session_title = use_memo(move || {
    chat_history()
      .get_current_session()
      .map(|s| s.title.clone())
      .unwrap_or_else(|| "New Chat".to_string())
  });

  // Get sessions list for rendering (clone to owned Vec to fix lifetime issues)
  let sessions_list = sessions().clone();

  // Sidebar close handler
  let sidebar_close_handler = {
    let mut collapsed = sidebar_collapsed.clone();
    move |_| collapsed.set(true)
  };

  // Monitor global trigger for activating input (from hotkey/tray)
  let mut last_processed = use_signal(|| {
    ACTIVATE_INPUT_TRIGGER
      .get()
      .and_then(|t| t.lock().ok().map(|g| *g))
      .unwrap_or(0u64)
  });

  use_resource(move || {
    async move {
      loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        if let Some(trigger) = ACTIVATE_INPUT_TRIGGER.get() {
          if let Ok(count) = trigger.lock() {
            let current = *count;
            if current > last_processed() {
              last_processed.set(current);
              // Focus input and type "/" using JavaScript
              let _ = dioxus::document::eval(
                r#"
                                    const textarea = document.querySelector('textarea.chat-input-textarea');
                                    if (textarea) {
                                        textarea.focus();
                                        textarea.value = '/';
                                        textarea.dispatchEvent(new Event('input', { bubbles: true }));
                                    }
                                "#,
              );
            }
          }
        }
      }
    }
  });

  rsx! {
    div {
      class: "flex flex-1 overflow-hidden h-full relative gap-0 lg:gap-4",

      // Sidebar - Session History (drawer overlay, not in flex flow)
      ChatSidebar {
        sessions: sessions_list,
        sidebar_collapsed: sidebar_collapsed(),
        on_new_chat: new_chat_for_sidebar,
        on_switch_session: switch_session,
        on_delete_session: delete_session,
        on_close: sidebar_close_handler,
        on_auto_collapse: {
          let mut collapsed = sidebar_collapsed.clone();
          let width = window_width.clone();
          move |_| {
            if width() < RESPONSIVE_BREAKPOINT {
              collapsed.set(true);
            }
          }
        },
      }

      // Main chat area (full width)
      div {
        class: "flex-1 flex flex-col bg-bg-primary overflow-hidden",

        // Header
        ChatHeader {
          current_session_title: current_session_title(),
          sidebar_collapsed: sidebar_collapsed(),
          on_toggle_sidebar: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
          on_new_chat: new_chat_for_header,
        }

        // Messages area (scrollable)
        MessageList {
          messages: messages.read().clone(),
          has_api_key,
          scroll_container_id: scroll_container_id.to_string(),
          edit_state: edit_state(),
          is_agent_running: is_agent_running(),
          on_edit: Callback::new(move |(message_id, content): (String, String)| {
            if content.is_empty() {
              // Cancel edit
              edit_state.set(None);
            } else {
              // Start edit or confirm edit
              let current = edit_state();
              if current.as_ref().map(|e| &e.message_id) == Some(&message_id) && current.as_ref().map(|e| e.is_editing).unwrap_or(false) {
                // Confirm edit - send to regenerate coroutine
                tx_regenerate.send((message_id.clone(), content));
                edit_state.set(None);
              } else {
                // Start edit mode
                edit_state.set(Some(crate::components::chat::MessageEdit {
                  message_id,
                  is_editing: true,
                  edit_content: content,
                }));
              }
            }
          }),
          on_regenerate: Callback::new(move |assistant_message_id: String| {
            // Find the user message before this assistant message
            let msgs = messages.read();
            if let Some(pos) = msgs.iter().position(|m| m.id == assistant_message_id) {
              // Look backwards for the first user message
              for i in (0..pos).rev() {
                if msgs[i].role == "user" {
                  let user_msg_id = msgs[i].id.clone();
                  let user_msg_content = msgs[i].content.clone();
                  // Trigger regeneration by "editing" the user message (content unchanged)
                  tx_regenerate.send((user_msg_id, user_msg_content));
                  break;
                }
              }
            }
          }),
        }

        // Input area
        InputArea {
          input_text: input_text.clone(),
          has_api_key,
          on_send: send_message,
          tx: tx.clone(),
          tx_with_prefix: tx_with_prefix.clone(),
          is_agent_running: is_agent_running.clone(),
          active_provider_id: active_provider_id(),
          enabled_providers: enabled_providers(),
          enabled_mcp_servers: enabled_mcp_servers(),
          on_switch_provider: switch_provider,
        }
      }
    }
  }
}

/// Helper function to get active provider info
///
/// IMPORTANT: A provider is only usable if it's enabled AND has a non-empty API key
fn get_active_provider_info() -> (String, bool) {
  let config_result = AppConfig::load();
  let (name, has_key) = match config_result {
    Ok(config) => {
      // Use get_usable_provider which checks: exists + enabled + has API key
      match config.get_usable_provider() {
        Some(provider) => (provider.name.clone(), true),
        None => {
          // Active provider is not usable - show warning
          let active_id = config.ai.active_provider.as_deref().unwrap_or("none");
          eprintln!(
            "[WARN] Active provider '{}' is not usable (missing, disabled, or no API key)",
            active_id
          );
          ("No Usable Provider".to_string(), false)
        }
      }
    }
    Err(_) => ("No Provider".to_string(), false),
  };
  println!("[DEBUG] Provider state: name={}, has_key={}", name, has_key);
  (name, has_key)
}
