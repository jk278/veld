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
// PART 3: UI Instructions - Single Direction Data Flow
// ============================================================================

/// UI 操作指令 - 单向数据流的核心
enum UiInstruction {
  /// 添加新消息到 UI
  AddMessage {
    id: String,
    role: String,
    content: String,
    timestamp: u64,
  },

  /// 更新现有消息内容
  UpdateMessage {
    id: String,
    content: String,
  },

  /// 完成消息，触发持久化
  FinalizeMessage {
    id: String,
  },
}

// ============================================================================
// PART 4: Message Store - Single Source of Truth
// ============================================================================

/// 消息存储 - 管理所有消息状态（单一数据源）
struct MessageStore {
  messages: Signal<Vec<ChatMessage>>,
  history: Signal<ChatHistoryData>,
  is_running: Signal<bool>,
}

impl MessageStore {
  fn new(
    messages: Signal<Vec<ChatMessage>>,
    history: Signal<ChatHistoryData>,
    is_running: Signal<bool>,
  ) -> Self {
    Self {
      messages,
      history,
      is_running,
    }
  }

  /// 处理 UI 指令 - 唯一的状态修改入口
  fn handle_instruction(&mut self, instruction: UiInstruction) {
    match instruction {
      UiInstruction::AddMessage { id, role, content, timestamp } => {
        self.add_message(id, role, content, timestamp);
      }
      UiInstruction::UpdateMessage { id, content } => {
        self.update_message(id, content);
      }
      UiInstruction::FinalizeMessage { id } => {
        self.finalize_message(id);
      }
    }
  }

  /// 添加消息到 UI
  fn add_message(&mut self, id: String, role: String, content: String, timestamp: u64) {
    self.messages.push(ChatMessage {
      id,
      role,
      content,
      timestamp,
    });
  }

  /// 更新 UI 中的消息
  fn update_message(&mut self, id: String, content: String) {
    let current = self.messages.read().clone();

    if let Some(pos) = current.iter().position(|m| m.id == id) {
      let mut updated = current;
      updated[pos].content = content;
      self.messages.set(updated);
    } else {
      // 消息不存在，添加新消息
      self.add_message(id, "assistant".to_string(), content, Self::now_secs());
    }
  }

  /// 完成消息，保存到历史记录
  fn finalize_message(&mut self, id: String) {
    // 从 UI 找到消息并保存到历史
    let current = self.messages.read().clone();
    if let Some(msg) = current.iter().find(|m| m.id == id) {
      self.history.write().add_message(HistoryMessage::from(msg.clone()));
      Self::sync_history(&self.history);
    }
  }

  /// 添加用户消息（同时添加到 UI 和历史）
  fn add_user_message(&mut self, content: String) -> String {
    let id = MessageIdGenerator::new().generate_user();
    let timestamp = Self::now_secs();

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

    id
  }

  /// 从历史重建 UI（用于会话切换等场景）
  fn rebuild_from_history(&mut self) {
    if let Some(session) = self.history.read().get_current_session() {
      let messages: Vec<ChatMessage> =
        session.messages.iter().cloned().map(Into::into).collect();
      self.messages.set(messages);
    }
  }

  /// 同步历史到磁盘并触发响应式更新
  fn sync_history(history: &Signal<ChatHistoryData>) {
    let history_clone = (*history.read()).clone();
    let _ = history.read().save();
    let mut history = history.clone();
    history.set(history_clone);
  }

  fn now_secs() -> u64 {
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs()
  }

  /// 构建给 API 的消息列表
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
}

// ============================================================================
// PART 5: Stream Processor - Clear Responsibility
// ============================================================================

/// 处理状态
#[derive(Debug, Clone, Copy, PartialEq)]
enum ProcessingState {
  CollectingThinking,
  StreamingResponse,
  Complete,
}

/// 处理 Agent 事件流，生成 UI 指令
struct StreamProcessor {
  id_gen: MessageIdGenerator,
  thinking_id: Option<String>,
  response_id: Option<String>,
  intermediate_steps: Vec<String>,
  final_response: String,
  state: ProcessingState,
  update_counter: usize,
}

impl StreamProcessor {
  fn new() -> Self {
    Self {
      id_gen: MessageIdGenerator::new(),
      thinking_id: None,
      response_id: None,
      intermediate_steps: Vec::new(),
      final_response: String::new(),
      state: ProcessingState::CollectingThinking,
      update_counter: 0,
    }
  }

  fn initialize(&mut self) {
    self.thinking_id = Some(self.id_gen.generate_thinking());
    self.response_id = Some(self.id_gen.generate_response());
  }

  /// 处理单个 Agent Step，返回 UI 指令列表
  fn process_step(&mut self, step: AgentStep) -> Vec<UiInstruction> {
    match step {
      AgentStep::Connecting(msg) => {
        self.intermediate_steps.push(format!("• {}", msg));
        self.update_thinking_ui()
      }
      AgentStep::Thinking { short, content } => {
        self.process_thinking_content(short, content);
        self.update_thinking_ui()
      }
      AgentStep::ToolCall { name, .. } => {
        self.intermediate_steps.push(format!("• ⏳ 调用 {}", name));
        self.update_thinking_ui()
      }
      AgentStep::ToolResult { name, .. } => {
        self.update_tool_result(name);
        self.update_thinking_ui()
      }
      AgentStep::Chunk(chunk) => {
        self.process_chunk(chunk)
      }
      AgentStep::Final => {
        self.state = ProcessingState::Complete;
        vec![UiInstruction::FinalizeMessage {
          id: self.response_id.clone().unwrap(),
        }]
      }
    }
  }

  fn process_thinking_content(&mut self, short: String, content: Option<String>) {
    if let Some(thought_content) = content {
      let formatted = Self::format_thinking_content(&thought_content);
      if !formatted.trim().is_empty() {
        self.intermediate_steps.push(formatted);
      }
    } else if !short.is_empty() {
      self.intermediate_steps.push(format!("• {}", short));
    }
  }

  fn update_tool_result(&mut self, name: String) {
    if let Some(pos) = self.intermediate_steps.iter().rposition(|s| s.contains("⏳")) {
      self.intermediate_steps[pos] = format!("• ✓ 调用 {}", name);
    } else {
      self.intermediate_steps.push(format!("• ✓ 调用 {}", name));
    }
  }

  fn process_chunk(&mut self, chunk: String) -> Vec<UiInstruction> {
    // 首次进入流式状态
    if self.state == ProcessingState::CollectingThinking && !chunk.trim().is_empty() {
      self.state = ProcessingState::StreamingResponse;

      // 先添加到 final_response，避免丢失第一个chunk
      self.final_response.push_str(&chunk);

      // 返回两个指令：完成思考，开始响应
      return vec![
        UiInstruction::FinalizeMessage {
          id: self.thinking_id.clone().unwrap(),
        },
        UiInstruction::AddMessage {
          id: self.response_id.clone().unwrap(),
          role: "assistant".to_string(),
          content: self.final_response.clone(),
          timestamp: MessageStore::now_secs(),
        },
      ];
    }

    // 继续流式更新
    self.final_response.push_str(&chunk);

    // 控制更新频率
    self.update_counter += 1;
    if self.update_counter % 5 == 0 {
      vec![UiInstruction::UpdateMessage {
        id: self.response_id.clone().unwrap(),
        content: self.final_response.clone(),
      }]
    } else {
      vec![]
    }
  }

  fn update_thinking_ui(&mut self) -> Vec<UiInstruction> {
    vec![UiInstruction::UpdateMessage {
      id: self.thinking_id.clone().unwrap(),
      content: self.intermediate_steps.join("\n"),
    }]
  }

  /// 格式化思考内容，过滤掉 JSON tool calls
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

// ============================================================================
// PART 6: Agent Execution Pipeline
// ============================================================================

/// 启动 agent 执行
async fn execute_agent(
  api_messages: Vec<crate::services::ChatMessage>,
) -> tokio::sync::mpsc::UnboundedReceiver<AgentStep> {
  use tokio::sync::mpsc;

  let (step_tx, step_rx) = mpsc::unbounded_channel();
  let (abort_tx, abort_rx) = tokio::sync::broadcast::channel::<()>(1);
  tokio::spawn(async move { set_abort_sender(abort_tx).await });

  tokio::spawn(async move {
    if let Err(e) = chat_with_tools(api_messages, step_tx, abort_rx).await {
      eprintln!("[HOOKS] Agent error: {:?}", e);
    }
  });

  step_rx
}

/// 处理 agent 流并更新 UI
async fn process_agent_stream(
  mut step_rx: tokio::sync::mpsc::UnboundedReceiver<AgentStep>,
  store: &mut MessageStore,
) {
  let mut processor = StreamProcessor::new();
  processor.initialize();

  store.is_running.set(true);

  eprintln!(
    "=== STARTING AGENT: thinking={}, response={} ===",
    processor.thinking_id.as_ref().unwrap(),
    processor.response_id.as_ref().unwrap()
  );

  // 初始化思考消息
  store.handle_instruction(UiInstruction::AddMessage {
    id: processor.thinking_id.clone().unwrap(),
    role: "assistant".to_string(),
    content: String::new(),
    timestamp: MessageStore::now_secs(),
  });

  while let Some(step) = step_rx.recv().await {
    let instructions = processor.process_step(step);

    // 执行所有指令
    for instruction in instructions {
      store.handle_instruction(instruction);
    }

    // 检查是否完成
    if processor.state == ProcessingState::Complete {
      store.is_running.set(false);
      break;
    }
  }

  store.is_running.set(false);
}

/// 完整的 agent 执行流程
async fn run_agent_pipeline(
  api_messages: Vec<crate::services::ChatMessage>,
  store: &mut MessageStore,
) {
  let step_rx = execute_agent(api_messages).await;
  process_agent_stream(step_rx, store).await;
}

// ============================================================================
// PART 7: Public Coroutine Hooks
// ============================================================================

/// Hook for the chat coroutine that handles AI calls and streaming responses
pub fn use_chat_coroutine(
  messages: Signal<Vec<ChatMessage>>,
  chat_history: Signal<ChatHistoryData>,
  is_agent_running: Signal<bool>,
) -> Coroutine<String> {
  use_coroutine(move |mut rx: UnboundedReceiver<String>| {
    let mut store = MessageStore::new(messages, chat_history, is_agent_running);
    async move {
      use futures_util::stream::StreamExt;
      while let Some(text) = rx.next().await {
        store.add_user_message(text);
        let api_messages = store.build_api_messages();
        run_agent_pipeline(api_messages, &mut store).await;
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
    let mut store = MessageStore::new(messages, chat_history, is_agent_running);
    async move {
      use futures_util::stream::StreamExt;
      while let Some((message_id, new_content)) = rx.next().await {
        // 更新消息并截断历史
        store.history.write().update_message(&message_id, new_content);

        // 先保存 index，避免借用冲突
        let index = store.history.read().get_message_index(&message_id);
        if let Some(idx) = index {
          store.history.write().truncate_from_index(idx + 1);
        }

        MessageStore::sync_history(&store.history);
        store.rebuild_from_history();

        // 构建消息并运行
        let api_messages: Vec<crate::services::ChatMessage> = store
          .history
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

        run_agent_pipeline(api_messages, &mut store).await;
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
    let mut store = MessageStore::new(messages, chat_history, is_agent_running);
    async move {
      use futures_util::stream::StreamExt;
      while let Some((text, prompt)) = rx.next().await {
        let display_content = if let Some(ref p) = prompt {
          format!("{}{}", p.prefix, text)
        } else {
          text.clone()
        };

        store.add_user_message(display_content);
        let api_messages = store.build_api_messages();
        run_agent_pipeline(api_messages, &mut store).await;
      }
    }
  })
}

// ============================================================================
// PART 8: UI Interaction Hooks
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
      if session.messages.is_empty() {
        return;
      }

      let current_msgs: Vec<ChatMessage> =
        session.messages.iter().cloned().map(Into::into).collect();

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
