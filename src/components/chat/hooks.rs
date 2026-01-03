//! Chat hooks - Flat architecture with direct step handling
//! 聊天 Hooks - 扁平架构直接处理步骤

use super::message_list::ChatMessage;
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use crate::config::QuickPrompt;
use crate::services::agent::Step;
use crate::services::chat_with_tools;
use dioxus::document;
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use std::time::SystemTime;

// ============================================================================
// Part 1: Abort Handling
// ============================================================================

use tokio::sync::{broadcast, Mutex};

static CURRENT_ABORT_SENDER: Mutex<Option<broadcast::Sender<()>>> = Mutex::const_new(None);

async fn set_abort_sender(sender: broadcast::Sender<()>) {
  *CURRENT_ABORT_SENDER.lock().await = Some(sender);
}

pub async fn abort_streaming() {
  if let Some(sender) = CURRENT_ABORT_SENDER.lock().await.as_ref() {
    let _ = sender.send(());
  }
}

// ============================================================================
// Part 2: Message Operations
// ============================================================================

/// Message operations - direct UI updates
struct MessageOps {
  messages: Signal<Vec<ChatMessage>>,
  history: Signal<ChatHistoryData>,
  /// Current answer session ID (timestamp-based, unique per conversation)
  current_answer_session: Option<String>,
}

impl MessageOps {
  fn new(messages: Signal<Vec<ChatMessage>>, history: Signal<ChatHistoryData>) -> Self {
    Self {
      messages,
      history,
      current_answer_session: None,
    }
  }

  /// Add user message
  fn add_user(&mut self, content: String) -> String {
    let id = format!("user-{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis());
    let timestamp = now_secs();

    self.messages.push(ChatMessage {
      id: id.clone(),
      role: "user".to_string(),
      content: content.clone(),
      timestamp,
    });

    self.history.write().add_message(HistoryMessage {
      id: id.clone(),
      role: "user".to_string(),
      content,
      timestamp,
    });
    Self::sync_history(&self.history);

    // Reset current answer session for new conversation turn
    self.current_answer_session = None;

    id
  }

  /// Convert step to chat message - PURE: every step is a message
  fn add_step(&mut self, step: &Step) {
    match step {
      Step::Tool { id, name, args, result, status, timestamp } => {
        let content = serde_json::to_string(&Step::Tool {
          id: id.clone(),
          name: name.clone(),
          args: args.clone(),
          result: result.clone(),
          status: *status,
          timestamp: *timestamp,
        }).unwrap_or_default();

        // Check if this tool step already exists
        let mut msgs = self.messages.read().clone();
        if let Some(pos) = msgs.iter().position(|m| m.id == *id) {
          // Update existing step
          msgs[pos].content = content;
          self.messages.set(msgs);
        } else {
          // PURE: Just push, preserve emit order
          self.messages.push(ChatMessage {
            id: id.clone(),
            role: "assistant".to_string(),
            content,
            timestamp: *timestamp,
          });
        }

        // Reset answer session when tool completes to separate answer segments
        if matches!(status, crate::services::agent::ToolStatus::Success | crate::services::agent::ToolStatus::Error) {
          self.current_answer_session = None;
        }

      }
      Step::Info { id, text, timestamp } => {
        // Skip connection info messages (conn-*) - they're technical details
        if id.starts_with("conn-") {
          return;
        }

        // Check if this info step already exists
        let mut msgs = self.messages.read().clone();
        if let Some(pos) = msgs.iter().position(|m| m.id == *id) {
          // Update existing step
          msgs[pos].content = text.clone();
          self.messages.set(msgs);
        } else {
          // PURE: Just push
          self.messages.push(ChatMessage {
            id: id.clone(),
            role: "assistant".to_string(),
            content: text.clone(),
            timestamp: *timestamp,
          });
        }
      }
      Step::Answer { content, done, timestamp } => {
        if !content.is_empty() || *done {
          if self.current_answer_session.is_none() && !content.is_empty() {
            let timestamp_ms = SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)
              .unwrap()
              .as_millis();
            self.current_answer_session = Some(format!("answer-{}", timestamp_ms));
          }

          let answer_id = self.current_answer_session.clone()
            .unwrap_or_else(|| "answer-current".to_string());

          let mut msgs = self.messages.read().clone();

          if let Some(pos) = msgs.iter().position(|m| m.id == answer_id) {
            msgs[pos].content = format!("{}{}", msgs[pos].content, content);
            msgs[pos].timestamp = *timestamp;
            self.messages.set(msgs);
          } else {
            self.messages.push(ChatMessage {
              id: answer_id.clone(),
              role: "assistant".to_string(),
              content: content.clone(),
              timestamp: *timestamp,
            });
          }

          if *done && content.is_empty() {
            self.current_answer_session = None;
          }
        }
      }
    }
  }

  /// Save to history
  fn save_to_history(&mut self) {
    let msgs = self.messages.read();
    for msg in msgs.iter() {
      if msg.role == "assistant" {
        self.history.write().add_message(HistoryMessage::from(msg.clone()));
      }
    }
    Self::sync_history(&self.history);
  }

  /// Build API messages from current chat
  fn build_api_messages(&self) -> Vec<crate::services::ChatMessage> {
    self
      .messages
      .read()
      .iter()
      .filter(|m| m.role != "system" && !m.id.starts_with("tool-"))
      .map(|m| crate::services::ChatMessage {
        role: m.role.clone(),
        content: m.content.clone(),
      })
      .collect()
  }

  fn sync_history(history: &Signal<ChatHistoryData>) {
    let history_clone = (*history.read()).clone();
    let _ = history.read().save();
    let mut history = history.clone();
    history.set(history_clone);
  }
}

fn now_secs() -> u64 {
  SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs()
}

// ============================================================================
// Part 3: Agent Execution
// ============================================================================

/// Execute agent and stream steps to UI
async fn execute_agent(
  messages: Vec<crate::services::ChatMessage>,
  mut ops: MessageOps,
  mut is_running: Signal<bool>,
) {
  use tokio::sync::mpsc;

  let (step_tx, mut step_rx) = mpsc::unbounded_channel::<Step>();
  let (abort_tx, abort_rx) = tokio::sync::broadcast::channel::<()>(1);

  // Set abort sender
  tokio::spawn(async move { set_abort_sender(abort_tx).await });

  // Spawn agent execution
  tokio::spawn(async move {
    if let Err(e) = chat_with_tools(messages, step_tx, abort_rx).await {
      error!("[HOOKS] Agent error: {:?}", e);
    }
  });

  // Process steps
  is_running.set(true);

  while let Some(step) = step_rx.recv().await {
    ops.add_step(&step);

    // Check if done
    if step.is_done() {
      break;
    }
  }

  // Save all messages to history when done
  ops.save_to_history();
  is_running.set(false);
}

// ============================================================================
// Part 4: Public Hooks
// ============================================================================

/// Chat coroutine hook
pub fn use_chat_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<String> {
  use_coroutine(move |mut rx: UnboundedReceiver<String>| {
    async move {
      use futures_util::stream::StreamExt;
      while let Some(text) = rx.next().await {
        let mut ops = MessageOps::new(messages, chat_history);
        ops.add_user(text);

        let api_messages = ops.build_api_messages();
        execute_agent(api_messages, ops, is_agent_running.clone()).await;
      }
    }
  })
}

/// Regenerate coroutine hook
pub fn use_regenerate_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<(String, String)> {
  use_coroutine(move |mut rx: UnboundedReceiver<(String, String)>| {
    async move {
      use futures_util::stream::StreamExt;
      while let Some((message_id, new_content)) = rx.next().await {
        let mut ops = MessageOps::new(messages, chat_history);
        ops.history.write().update_message(&message_id, new_content);

        let idx = ops.history.read().get_message_index(&message_id);
        if let Some(idx) = idx {
          ops.history.write().truncate_from_index(idx + 1);

          let history_msgs: Vec<ChatMessage> = ops
            .history
            .read()
            .get_current_session()
            .map(|s| s.messages.iter().cloned().map(Into::into).collect())
            .unwrap_or_default();
          ops.messages.set(history_msgs);
        }

        MessageOps::sync_history(&ops.history);

        let api_messages: Vec<crate::services::ChatMessage> = ops
          .history
          .read()
          .get_current_session()
          .map(|s| {
            s.messages
              .iter()
              .filter(|m| m.role != "system" && !m.id.starts_with("tool-"))
              .map(|m| crate::services::ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
              })
              .collect()
          })
          .unwrap_or_default();

        execute_agent(api_messages, ops, is_agent_running.clone()).await;
      }
    }
  })
}

/// Chat coroutine with quick prompt support
pub fn use_chat_coroutine_with_prefix(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<(String, Option<QuickPrompt>)> {
  use_coroutine(move |mut rx: UnboundedReceiver<(String, Option<QuickPrompt>)>| {
    async move {
      use futures_util::stream::StreamExt;
      while let Some((text, prompt)) = rx.next().await {
        let display_content = if let Some(ref p) = prompt {
          format!("{}{}", p.prefix, text)
        } else {
          text.clone()
        };

        let mut ops = MessageOps::new(messages, chat_history);
        ops.add_user(display_content);

        let api_messages = ops.build_api_messages();
        execute_agent(api_messages, ops, is_agent_running.clone()).await;
      }
    }
  })
}

// ============================================================================
// Part 5: UI Interaction Hooks
// ============================================================================

/// Scroll manager
struct ScrollManager;

impl ScrollManager {
  const AUTO_MODE: &'static str = "auto";
  const MANUAL_MODE: &'static str = "manual";
  const BOTTOM_THRESHOLD: i64 = 50;
  const TOP_THRESHOLD: i64 = 150;

  fn init_script(container_id: &str) -> String {
    format!(
      r#"(function() {{
        const container = document.getElementById("{}");
        if (!container) return;
        window.__veldScrollState = '{}';
        container.addEventListener('scroll', () => {{
          const scrollTop = container.scrollTop;
          const scrollHeight = container.scrollHeight;
          const clientHeight = container.clientHeight;
          const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
          if (distanceFromBottom > {}) {{
            window.__veldScrollState = '{}';
          }} else if (distanceFromBottom < {}) {{
            window.__veldScrollState = '{}';
          }}
        }}, {{ passive: true }});
      }})()"#,
      container_id,
      Self::AUTO_MODE,
      Self::TOP_THRESHOLD,
      Self::MANUAL_MODE,
      Self::BOTTOM_THRESHOLD,
      Self::AUTO_MODE
    )
  }

  fn scroll_to_bottom_script(container_id: &str) -> String {
    format!(
      r#"(function() {{
        const container = document.getElementById("{}");
        if (!container) return;
        if (window.__veldScrollState === '{}') {{
          container.scrollTo({{ top: container.scrollHeight, behavior: "smooth" }});
        }}
      }})()"#,
      container_id,
      Self::AUTO_MODE
    )
  }

  fn force_scroll_to_bottom_script(container_id: &str) -> String {
    format!(
      r#"(function() {{
        const container = document.getElementById("{}");
        if (!container) return;
        if (window.__veldScrollState !== '{}') {{
          container.scrollTo({{ top: container.scrollHeight, behavior: "smooth" }});
        }}
      }})()"#,
      container_id,
      Self::MANUAL_MODE
    )
  }

  fn init(container_id: &str) {
    document::eval(Self::init_script(container_id).as_str());
  }

  fn scroll_if_auto(container_id: &str) {
    document::eval(Self::scroll_to_bottom_script(container_id).as_str());
  }

  fn force_scroll(container_id: &str) {
    document::eval(Self::force_scroll_to_bottom_script(container_id).as_str());
  }
}

/// Auto scroll hook
pub fn use_auto_scroll(
  messages: Signal<Vec<ChatMessage>>,
  mut last_message_count: Signal<usize>,
  scroll_container_id: String,
) {
  use_effect(move || {
    let current_count = messages().len();
    let prev_count = last_message_count();

    if current_count != prev_count {
      last_message_count.set(current_count);
    }

    if current_count > prev_count {
      ScrollManager::scroll_if_auto(&scroll_container_id);
    }
  });
}

/// Streaming scroll hook
pub fn use_streaming_scroll(
  messages: Signal<Vec<ChatMessage>>,
  scroll_container_id: String,
  is_agent_running: Signal<bool>,
) {
  let mut last_content = use_signal(|| String::new());

  use_effect(move || {
    let msgs = messages();
    let is_running = is_agent_running();

    if !is_running {
      return;
    }

    if let Some(last_msg) = msgs.last() {
      if last_msg.role == "assistant" {
        let current_content = last_msg.content.clone();

        if current_content != last_content() {
          last_content.set(current_content.clone());
          ScrollManager::force_scroll(&scroll_container_id);
        }
      }
    }
  });
}

/// Scroll state init hook
pub fn use_scroll_state_init(scroll_container_id: String) {
  use_effect(move || {
    ScrollManager::init(&scroll_container_id);
  });
}
