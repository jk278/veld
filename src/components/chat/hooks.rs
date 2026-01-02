//! Custom hooks for chat functionality
//! 聊天功能自定义 Hooks

use super::message_list::ChatMessage;
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use crate::config::QuickPrompt;
use crate::services::{chat_with_tools, AgentStep};
use dioxus::document;
use dioxus::prelude::*;
use std::time::SystemTime;

// ============================================================================
// PART 1: Type Conversions & ID Generation
// ============================================================================

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

/// Message ID generator - centralizes ID generation logic
struct MessageIdGenerator {
  counter: u64,
}

impl MessageIdGenerator {
  fn new() -> Self {
    Self { counter: 0 }
  }

  fn generate_user(&mut self) -> String {
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    self.counter += 1;
    format!("msg-{}-{}", now, self.counter)
  }

  fn generate_thinking(&mut self) -> String {
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    self.counter += 1;
    format!("thinking-{}-{}", now, self.counter)
  }

  fn generate_response(&mut self) -> String {
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    self.counter += 1;
    format!("msg-{}-{}", now, self.counter)
  }
}

// ============================================================================
// PART 2: Global Abort State (TODO: Replace with Dioxus context)
// ============================================================================

/// Global abort sender for interrupting streaming responses
/// TODO: Replace with Dioxus context to avoid global mutable state
use tokio::sync::{broadcast, Mutex};

static CURRENT_ABORT_SENDER: Mutex<Option<broadcast::Sender<()>>> = Mutex::const_new(None);

async fn set_abort_sender(sender: broadcast::Sender<()>) {
  let mut current = CURRENT_ABORT_SENDER.lock().await;
  *current = Some(sender);
}

pub async fn get_abort_sender() -> Option<broadcast::Sender<()>> {
  let current = CURRENT_ABORT_SENDER.lock().await;
  current.clone()
}

pub async fn abort_streaming() {
  if let Some(sender) = get_abort_sender().await {
    let _ = sender.send(());
  }
}

// ============================================================================
// PART 3: Agent Execution State Machine
// ============================================================================

/// Agent execution state
#[derive(Debug, Clone, Copy, PartialEq)]
enum AgentState {
  Thinking,
  Streaming,
  Complete,
}

/// Agent executor - manages the streaming response lifecycle
struct AgentExecutor {
  id_gen: MessageIdGenerator,
  thinking_id: Option<String>,
  response_id: Option<String>,
  intermediate_steps: Vec<String>,
  final_response: String,
  state: AgentState,
  update_counter: usize,
  thinking_saved: bool,
}

impl AgentExecutor {
  fn new() -> Self {
    Self {
      id_gen: MessageIdGenerator::new(),
      thinking_id: None,
      response_id: None,
      intermediate_steps: Vec::new(),
      final_response: String::new(),
      state: AgentState::Thinking,
      update_counter: 0,
      thinking_saved: false,
    }
  }

  fn initialize(&mut self) {
    self.thinking_id = Some(self.id_gen.generate_thinking());
    self.response_id = Some(self.id_gen.generate_response());
  }

  /// Process an agent step and return the action to take
  fn process_step(&mut self, step: AgentStep) -> ExecutionAction {
    match step {
      AgentStep::Connecting(msg) => {
        self.intermediate_steps.push(format!("• {}", msg));
        ExecutionAction::UpdateUI
      }
      AgentStep::Thinking { short, content } => {
        self.process_thinking(short, content)
      }
      AgentStep::ToolCall { name, .. } => {
        self.intermediate_steps.push(format!("• ⏳ 调用 {}", name));
        ExecutionAction::UpdateUI
      }
      AgentStep::ToolResult { name, .. } => {
        self.update_tool_result(name);
        ExecutionAction::UpdateUI
      }
      AgentStep::Chunk(chunk) => {
        self.process_chunk(chunk)
      }
      AgentStep::Final => {
        self.state = AgentState::Complete;
        ExecutionAction::Complete
      }
    }
  }

  fn process_thinking(&mut self, short: String, content: Option<String>) -> ExecutionAction {
    if let Some(thought_content) = content {
      let formatted = Self::format_thinking_content(&thought_content);
      if !formatted.trim().is_empty() {
        self.intermediate_steps.push(formatted);
      }
    } else if !short.is_empty() {
      self.intermediate_steps.push(format!("• {}", short));
    }
    ExecutionAction::UpdateUI
  }

  fn update_tool_result(&mut self, name: String) {
    if let Some(pos) = self.intermediate_steps.iter().rposition(|s| s.contains("⏳")) {
      self.intermediate_steps[pos] = format!("• ✓ 调用 {}", name);
    } else {
      self.intermediate_steps.push(format!("• ✓ 调用 {}", name));
    }
  }

  fn process_chunk(&mut self, chunk: String) -> ExecutionAction {
    if self.state != AgentState::Streaming && !chunk.trim().is_empty() {
      self.state = AgentState::Streaming;
    }
    self.final_response.push_str(&chunk);
    ExecutionAction::StreamChunk
  }

  /// Check if UI should update based on current state
  fn should_update_ui(&mut self) -> bool {
    self.update_counter += 1;
    match self.state {
      AgentState::Streaming => self.update_counter % 5 == 0,
      _ => true,
    }
  }

  /// Get the display content for current state
  fn display_content(&self) -> String {
    match self.state {
      AgentState::Streaming => self.final_response.clone(),
      _ => self.intermediate_steps.join("\n"),
    }
  }

  /// Get the target message ID for current state
  fn target_id(&self) -> &str {
    match self.state {
      AgentState::Streaming => self.response_id.as_ref().unwrap(),
      _ => self.thinking_id.as_ref().unwrap(),
    }
  }

  /// Check if thinking message should be saved
  fn should_save_thinking(&self) -> bool {
    self.state == AgentState::Streaming
      && !self.intermediate_steps.is_empty()
      && !self.thinking_saved
  }

  fn mark_thinking_saved(&mut self) {
    self.thinking_saved = true;
  }

  fn thinking_content(&self) -> String {
    self.intermediate_steps.join("\n")
  }

  fn is_complete(&self) -> bool {
    self.state == AgentState::Complete
  }

  fn has_response(&self) -> bool {
    !self.final_response.is_empty()
  }

  fn response_id(&self) -> &str {
    self.response_id.as_ref().unwrap()
  }

  fn thinking_id(&self) -> &str {
    self.thinking_id.as_ref().unwrap()
  }

  /// Format thinking content: show text directly, filter out JSON tool calls
  fn format_thinking_content(content: &str) -> String {
    let normalized = content.replace(char::is_whitespace, "");

    if normalized.contains("\"tool_call\"")
      || normalized.contains("\"Tool_call\"")
      || normalized.contains("\"TOOL_CALL\"")
    {
      eprintln!("[FILTERED TOOL CALL JSON]");
      return String::from("• ");
    }

    let result: Vec<String> = content
      .lines()
      .map(|l| l.trim().to_string())
      .filter(|l| !l.is_empty())
      .collect();

    if result.is_empty() {
      format!("• ")
    } else {
      format!("• {}", result.join(" "))
    }
  }
}

/// Actions returned by the executor
enum ExecutionAction {
  UpdateUI,
  StreamChunk,
  Complete,
}

// ============================================================================
// PART 4: Coroutine Shared Logic
// ============================================================================

/// Shared context for chat coroutines
struct ChatContext {
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_running: Signal<bool>,
  id_gen: MessageIdGenerator,
}

impl ChatContext {
  fn new(
    messages: Signal<Vec<ChatMessage>>,
    chat_history: Signal<ChatHistoryData>,
    is_running: Signal<bool>,
  ) -> Self {
    Self {
      messages,
      chat_history,
      is_running,
      id_gen: MessageIdGenerator::new(),
    }
  }

  /// Add user message to both UI and history
  fn add_user_message(&mut self, content: String) -> String {
    let now = Self::now_secs();
    let msg_id = self.id_gen.generate_user();
    let user_msg = ChatMessage {
      id: msg_id.clone(),
      role: "user".to_string(),
      content: content.clone(),
      timestamp: now,
    };

    self.messages.push(user_msg.clone());
    self.chat_history.write().add_message(HistoryMessage::from(user_msg));
    Self::sync_history_signals(&self.chat_history);

    msg_id
  }

  /// Build API messages from current session (excluding system messages)
  fn build_api_messages(&self) -> Vec<crate::services::ChatMessage> {
    self
      .messages
      .read()
      .iter()
      .filter(|m| m.role != "system")
      .map(|m| crate::services::ChatMessage {
        role: m.role.clone(),
        content: m.content.clone(),
      })
      .collect()
  }

  /// Sync history to disk and trigger reactivity
  fn sync_history_signals(chat_history: &Signal<ChatHistoryData>) {
    let history_clone = (*chat_history.read()).clone();
    let _ = chat_history.read().save();
    // Clone the signal and set to trigger reactivity
    let mut chat = chat_history.clone();
    chat.set(history_clone);
  }

  fn now_secs() -> u64 {
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs()
  }
}

/// Execute the agent with proper UI updates
async fn execute_agent(
  api_messages: Vec<crate::services::ChatMessage>,
) -> tokio::sync::mpsc::UnboundedReceiver<AgentStep> {
  use tokio::sync::mpsc;

  let (step_tx, step_rx) = mpsc::unbounded_channel();
  let (abort_tx, abort_rx) = tokio::sync::broadcast::channel::<()>(1);
  tokio::spawn(async move { set_abort_sender(abort_tx).await });

  // Spawn the agent service
  tokio::spawn(async move {
    if let Err(e) = chat_with_tools(api_messages, step_tx, abort_rx).await {
      eprintln!("[HOOKS] Agent error: {:?}", e);
    }
  });

  step_rx
}

/// Process the agent stream with UI updates
async fn process_agent_stream(
  mut step_rx: tokio::sync::mpsc::UnboundedReceiver<AgentStep>,
  ctx: &mut ChatContext,
) {
  let mut executor = AgentExecutor::new();
  executor.initialize();

  ctx.is_running.set(true);

  eprintln!(
    "=== STARTING AGENT: thinking={}, response={} ===",
    executor.thinking_id(),
    executor.response_id()
  );

  while let Some(step) = step_rx.recv().await {
    let _action = executor.process_step(step);

    // Save thinking message on first stream chunk
    if executor.should_save_thinking() {
      let current_msgs = ctx.messages.read();
      let thinking_exists = current_msgs.iter().any(|m| m.id == executor.thinking_id());
      drop(current_msgs);

      if !thinking_exists {
        ctx.chat_history.write().add_message(HistoryMessage {
          id: executor.thinking_id().to_string(),
          role: "assistant".to_string(),
          content: executor.thinking_content(),
          timestamp: ChatContext::now_secs(),
        });
        executor.mark_thinking_saved();
      }
    }

    // Handle completion
    if executor.is_complete() {
      if executor.has_response() {
        let now = ChatContext::now_secs();
        let response_id = executor.response_id().to_string();
        let content = executor.final_response.clone();

        // Add to UI
        ctx.messages.push(ChatMessage {
          id: response_id.clone(),
          role: "assistant".to_string(),
          content: content.clone(),
          timestamp: now,
        });

        // Save to history
        ctx.chat_history.write().add_message(HistoryMessage {
          id: response_id,
          role: "assistant".to_string(),
          content,
          timestamp: now,
        });
        ChatContext::sync_history_signals(&ctx.chat_history);
      }

      ctx.is_running.set(false);
      break;
    }

    // Real-time UI updates
    if executor.should_update_ui() {
      update_ui_message(ctx, &executor);
    }
  }

  ctx.is_running.set(false);
}

/// Update a single message in the UI
fn update_ui_message(ctx: &mut ChatContext, executor: &AgentExecutor) {
  let display_content = executor.display_content();
  let target_id = executor.target_id().to_string();
  let timestamp = ChatContext::now_secs();

  let current_msgs = ctx.messages.read().clone();

  if current_msgs.iter().any(|m| m.id == target_id) {
    // Update existing message
    let mut updated = current_msgs;
    if let Some(msg) = updated.iter_mut().find(|m| m.id == target_id) {
      msg.content = display_content;
    }
    ctx.messages.set(updated);
  } else if !display_content.is_empty() {
    // Add new message
    ctx.messages.push(ChatMessage {
      id: target_id,
      role: "assistant".to_string(),
      content: display_content,
      timestamp,
    });
  }
}

/// Run the complete agent execution pipeline
async fn run_agent_pipeline(
  api_messages: Vec<crate::services::ChatMessage>,
  ctx: &mut ChatContext,
) {
  let step_rx = execute_agent(api_messages).await;
  process_agent_stream(step_rx, ctx).await;
}

// ============================================================================
// PART 5: Public Coroutine Hooks
// ============================================================================

/// Hook for the chat coroutine that handles AI calls and streaming responses
pub fn use_chat_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<String> {
  use_coroutine(move |mut rx: UnboundedReceiver<String>| {
    let mut ctx = ChatContext::new(messages, chat_history, is_agent_running);
    async move {
      use futures_util::stream::StreamExt;
      while let Some(text) = rx.next().await {
        ctx.add_user_message(text);
        let api_messages = ctx.build_api_messages();
        run_agent_pipeline(api_messages, &mut ctx).await;
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
    let mut ctx = ChatContext::new(messages, chat_history, is_agent_running);
    async move {
      use futures_util::stream::StreamExt;
      while let Some((message_id, new_content)) = rx.next().await {
        // Update the message
        ctx.chat_history.write().update_message(&message_id, new_content);

        // Get index and truncate everything after (drop read before write)
        let index = ctx.chat_history.read().get_message_index(&message_id);
        if let Some(idx) = index {
          ctx.chat_history.write().truncate_from_index(idx + 1);
        }

        ChatContext::sync_history_signals(&ctx.chat_history);

        // Sync UI to remove old messages
        let current_msgs: Vec<ChatMessage> = ctx
          .chat_history
          .read()
          .get_current_session()
          .map(|s| s.messages.iter().cloned().map(Into::into).collect())
          .unwrap_or_default();
        ctx.messages.set(current_msgs);

        // Build API messages and run
        let api_messages: Vec<crate::services::ChatMessage> = ctx
          .chat_history
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

        run_agent_pipeline(api_messages, &mut ctx).await;
      }
    }
  })
}

/// Hook for chat coroutine with quick prompt support
pub fn use_chat_coroutine_with_prefix(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<(String, Option<QuickPrompt>)> {
  use_coroutine(move |mut rx: UnboundedReceiver<(String, Option<QuickPrompt>)>| {
    let mut ctx = ChatContext::new(messages, chat_history, is_agent_running);
    async move {
      use futures_util::stream::StreamExt;
      while let Some((text, prompt)) = rx.next().await {
        // Prepend prompt prefix to user content
        let display_content = if let Some(ref p) = prompt {
          format!("{}{}", p.prefix, text)
        } else {
          text.clone()
        };

        ctx.add_user_message(display_content);
        let api_messages = ctx.build_api_messages();
        run_agent_pipeline(api_messages, &mut ctx).await;
      }
    }
  })
}

// ============================================================================
// PART 6: UI Interaction Hooks
// ============================================================================

/// Scroll manager - encapsulates scroll-related JavaScript interactions
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

/// Hook for message sync with chat history
pub fn use_message_sync(
  mut messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) {
  use_effect(move || {
    let _ = messages();
    let _ = chat_history();
    let is_running = is_agent_running();

    if let Some(session) = chat_history().get_current_session() {
      // Skip sync if session is empty (prevents overwriting user input)
      if session.messages.is_empty() {
        return;
      }

      let current_msgs: Vec<ChatMessage> =
        session.messages.iter().cloned().map(Into::into).collect();

      // Only sync if not running and messages differ
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

    if current_count != prev_count {
      last_message_count.set(current_count);
    }

    // Only scroll if new messages were added
    if current_count > prev_count {
      ScrollManager::scroll_if_auto(&scroll_container_id);
    }
  });
}

/// Hook for auto-scroll during streaming response
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

/// Hook for initializing scroll state tracking
pub fn use_scroll_state_init(scroll_container_id: String) {
  use_effect(move || {
    ScrollManager::init(&scroll_container_id);
  });
}
