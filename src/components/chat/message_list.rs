//! Message list component - Simplified rendering
//! 消息列表组件 - 简化渲染逻辑

use crate::components::icons::ChatIcon;
use crate::components::markdown::{MarkdownContent, PlainTextContent};
use crate::services::agent::Step;
use crate::theme::use_is_dark;
use dioxus::prelude::*;
use serde_json;

/// Chat message for display
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
  pub id: String,
  pub role: String,
  pub content: String,
  pub timestamp: u64,
}

/// Edit state for messages
#[derive(Clone, Debug, PartialEq)]
pub struct MessageEdit {
  pub message_id: String,
  pub is_editing: bool,
  pub edit_content: String,
}

/// Message list container
#[component]
pub fn MessageList(
  messages: Vec<ChatMessage>,
  has_api_key: bool,
  #[props(default)] scroll_container_id: String,
  #[props(default)] edit_state: Option<MessageEdit>,
  #[props(default)] on_edit: Option<EventHandler<(String, String)>>,
  #[props(default)] on_regenerate: Option<EventHandler<String>>,
  #[props(default)] is_agent_running: bool,
) -> Element {
  // Find last assistant message (non-step) for action buttons
  let last_assistant_msg_id: Option<String> = messages
    .iter()
    .rposition(|m| m.role == "assistant" && !m.id.starts_with("tool-") && !m.id.starts_with("conn-"))
    .and_then(|i| messages.get(i).map(|m| m.id.clone()));

  // Pre-compute show_actions for each message
  // Only the last assistant message shows actions, and only when not generating
  let messages_with_actions: Vec<(ChatMessage, bool)> = messages.into_iter()
    .map(|msg| {
      let is_last_assistant = last_assistant_msg_id.as_ref() == Some(&msg.id);
      let show = is_last_assistant && !is_agent_running;
      (msg, show)
    })
    .collect();

  rsx! {
    div {
      id: scroll_container_id,
      class: "flex-1 overflow-y-auto px-4 py-4 space-y-4",

      if messages_with_actions.is_empty() {
        EmptyState {
          has_api_key,
        }
      } else {
        for (msg, show_actions) in messages_with_actions.into_iter() {
          MessageBubble {
            message: msg,
            edit_state: edit_state.clone(),
            on_edit: on_edit.clone(),
            on_regenerate: on_regenerate.clone(),
            is_agent_running,
            show_actions,
          }
        }
      }
    }
  }
}

/// Empty state when no messages
#[component]
pub fn EmptyState(has_api_key: bool) -> Element {
  rsx! {
    div {
      class: "flex flex-col items-center justify-center h-full text-center gap-4 opacity-50",
      div {
        class: "text-6xl text-text-secondary",
        ChatIcon { class: "w-16 h-16" }
      }
      p {
        class: "text-lg text-text-secondary",
        "Start a conversation"
      }
      if !has_api_key {
        p {
          class: "text-sm text-text-muted",
          "Configure your API key in "
          a {
            class: "text-primary hover:underline",
            href: "/settings",
            "Settings"
          }
        }
      }
    }
  }
}

/// Individual message bubble
#[component]
fn MessageBubble(
  message: ChatMessage,
  #[props(default)] edit_state: Option<MessageEdit>,
  #[props(default)] on_edit: Option<EventHandler<(String, String)>>,
  #[props(default)] on_regenerate: Option<EventHandler<String>>,
  #[props(default)] is_agent_running: bool,
  #[props(default)] show_actions: bool,
) -> Element {
  // Check if this is an intermediate step (tool call or conn-info)
  let is_step = message.id.starts_with("tool-") || message.id.starts_with("conn-");

  // Check if this message is being edited
  let is_editing = edit_state
    .as_ref()
    .map(|e| e.is_editing && e.message_id == message.id)
    .unwrap_or(false);
  let edit_content = edit_state.as_ref().map(|e| e.edit_content.clone()).unwrap_or_default();

  rsx! {
    div {
      class: if message.role == "user" { "flex justify-end" } else if message.role == "system" { "flex justify-center" } else { "flex justify-start" },

      if message.role == "system" {
        div {
          class: "px-4 py-2 bg-error/10 border border-error/30 rounded-lg text-sm text-error max-w-md",
          {message.content.clone()}
        }
      } else if message.role == "user" {
        UserMessageBubble {
          content: message.content.clone(),
          timestamp: message.timestamp,
          is_editing,
          edit_content: edit_content.clone(),
          on_edit: on_edit.clone(),
          message_id: message.id.clone(),
        }
      } else {
        AssistantMessageBubble {
          content: message.content.clone(),
          timestamp: message.timestamp,
          is_step,
          message_id: message.id.clone(),
          on_regenerate: on_regenerate.clone(),
          is_agent_running,
          show_actions,
        }
      }
    }
  }
}

/// User message bubble
#[component]
fn UserMessageBubble(
  content: String,
  timestamp: u64,
  #[props(default)] is_editing: bool,
  #[props(default)] edit_content: String,
  #[props(default)] on_edit: Option<EventHandler<(String, String)>>,
  #[props(default)] message_id: String,
) -> Element {
  let mut local_edit = use_signal(|| edit_content.clone());

  // Update local edit signal when edit_content changes from parent
  use_effect(move || {
    if is_editing && local_edit() != edit_content {
      local_edit.set(edit_content.clone());
    }
  });

  // Clone for closures
  let message_id_key = message_id.clone();
  let message_id_cancel = message_id.clone();
  let message_id_save = message_id.clone();
  let message_id_edit = message_id.clone();
  let content_clone = content.clone();

  rsx! {
    div {
      class: "w-[80%] max-w-2xl flex justify-end flex-col items-end gap-2",

      // Edit mode
      if is_editing {
        div {
          class: "w-full flex flex-col gap-2",
          textarea {
            class: "w-full px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg resize-none outline-none focus:border-primary focus:ring-2 focus:ring-primary/20 transition-all font-mono text-sm",
            rows: 3,
            value: local_edit(),
            oninput: move |e: FormEvent| local_edit.set(e.value()),
            onkeydown: move |e: KeyboardEvent| {
              if e.key() == Key::Enter && e.modifiers().contains(dioxus::prelude::Modifiers::CONTROL) {
                if let Some(ref handler) = on_edit {
                  handler((message_id_key.clone(), local_edit()));
                }
              }
            },
          }
          div {
            class: "flex gap-2 justify-end",
            button {
              class: "px-3 py-1 text-sm bg-bg-secondary border border-border rounded hover:border-text-muted transition-colors",
              onclick: move |_| {
                if let Some(ref handler) = on_edit {
                  handler((message_id_cancel.clone(), String::new()));
                }
              },
              "取消"
            }
            button {
              class: "px-3 py-1 text-sm bg-primary text-white rounded hover:bg-primary/90 transition-colors",
              onclick: move |_| {
                if let Some(ref handler) = on_edit {
                  handler((message_id_save.clone(), local_edit()));
                }
              },
              "保存 (Ctrl+Enter)"
            }
          }
        }
      } else {
        // Normal display
        div {
          class: "px-4 py-2.5 bg-bg-secondary text-text-primary rounded-2xl rounded-tr-md",
          PlainTextContent {
            content: content.clone(),
            class: "text-sm leading-relaxed".to_string(),
          }
        }

        if on_edit.is_some() {
          button {
            class: "text-xs text-text-muted hover:text-text-primary transition-colors",
            onclick: move |_| {
              local_edit.set(content_clone.clone());
              if let Some(ref handler) = on_edit {
                handler((message_id_edit.clone(), content_clone.clone()));
              }
            },
            "编辑"
          }
        }
      }
    }
  }
}

/// Parse step from JSON content
fn parse_step(content: &str) -> Option<Step> {
  serde_json::from_str(content).ok()
}

/// Assistant message bubble
#[component]
fn AssistantMessageBubble(
  content: String,
  timestamp: u64,
  is_step: bool,
  #[props(default)] message_id: String,
  #[props(default)] on_regenerate: Option<EventHandler<String>>,
  #[props(default)] is_agent_running: bool,
  #[props(default)] show_actions: bool,
) -> Element {
  let is_dark = use_is_dark();
  let mut copy_status = use_signal(|| false);

  use dioxus::prelude::spawn;

  // Copy handler
  let copy_message = {
    let content = content.clone();
    move |_| {
      if arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(&content))
        .is_ok()
      {
        copy_status.set(true);
        let mut copy_status = copy_status.clone();
        spawn(async move {
          tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
          copy_status.set(false);
        });
      }
    }
  };

  // Regenerate handler
  let show_regenerate = on_regenerate.is_some() && !is_agent_running;
  let regenerate_handler = {
    let msg_id = message_id.clone();
    move |_| {
      if let Some(ref handler) = on_regenerate {
        handler(msg_id.clone());
      }
    }
  };

  rsx! {
    div {
      class: "max-w-2xl w-full",

      if is_step {
        // Render step (tool call or info)
        if let Some(step) = parse_step(&content) {
          StepRenderer {
            step,
            is_dark: is_dark(),
          }
        }
      } else {
        // Final answer: markdown rendering with action buttons
        div {
          div {
            class: "markdown-body w-full text-sm text-text-primary",
            MarkdownContent {
              content: content.clone(),
              class: String::new(),
              dark: is_dark(),
            }
          }
          // Action buttons at bottom
          if show_actions {
            div {
              class: "flex gap-2 mt-3 flex-wrap",
              button {
                class: format!("px-3 py-1.5 bg-bg-secondary border border-border rounded text-xs text-text-secondary hover:text-text-primary hover:border-text-muted transition-colors {}", if copy_status() { "bg-success/10 border-success/30 text-success" } else { "" }),
                onclick: copy_message,
                disabled: copy_status(),
                if copy_status() { "已复制!" } else { "复制" }
              }
              if show_regenerate {
                button {
                  class: "px-3 py-1.5 bg-bg-secondary border border-border rounded text-xs text-text-secondary hover:text-text-primary hover:border-text-muted transition-colors",
                  onclick: regenerate_handler,
                  "重新生成"
                }
              }
            }
          }
        }
      }
    }
  }
}

/// Step renderer - display tool calls (info steps are hidden)
#[component]
fn StepRenderer(step: Step, is_dark: bool) -> Element {
  match step {
    Step::Tool { name, args, result, status, .. } => {
      use crate::services::agent::ToolStatus;

      let (icon, icon_class, status_text) = match status {
        ToolStatus::Pending => ("⏳", "text-text-muted", "等待中..."),
        ToolStatus::Running => ("🔄", "text-primary", "执行中..."),
        ToolStatus::Success => ("✓", "text-success", "完成"),
        ToolStatus::Error => ("✗", "text-error", "失败"),
      };

      rsx! {
        details {
          class: "px-4 py-3 bg-bg-secondary border border-border rounded-lg",
          summary {
            class: "cursor-pointer flex items-center gap-2 text-sm select-none",
            span {
              class: "text-primary",
              "🔧"
            }
            span {
              class: "font-medium text-text-primary",
              "调用 {name}"
            }
            span {
              class: format!("text-xs {}", icon_class),
              {icon}
              {status_text}
            }
          }
          div {
            class: "mt-3 text-xs text-text-muted space-y-1",
            div {
              strong { "参数: " }
              code {
                class: "bg-bg-primary px-2 py-1 rounded text-xs",
                {format!("{:?}", args)}
              }
            }
            if let Some(res) = result {
              div {
                strong { "结果: " }
                div {
                  class: "mt-2 text-xs text-text-secondary max-h-96 overflow-y-auto",
                  {
                    // The result may be double-escaped (JSON string inside JSON string)
                    // Try to parse it, and if it fails or contains escape sequences, unescape it first
                    let display_content = if res.contains("\\\"") {
                      // Double-escaped JSON - need to parse twice
                      if let Ok(inner_json) = serde_json::from_str::<String>(&res) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&inner_json) {
                          json["content"]
                            .as_array()
                            .and_then(|arr| arr.first())
                            .and_then(|obj| obj["text"].as_str())
                            .unwrap_or(&inner_json)
                            .to_string()
                        } else {
                          inner_json
                        }
                      } else {
                        res.clone()
                      }
                    } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(&res) {
                      // Single-layer JSON
                      json["content"]
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|obj| obj["text"].as_str())
                        .unwrap_or(&res)
                        .to_string()
                    } else {
                      // Not JSON, use as-is
                      res.clone()
                    };
                    rsx! {
                      MarkdownContent {
                        content: display_content,
                        class: String::new(),
                        dark: is_dark,
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
    Step::Info { text, .. } => {
      rsx! {
        div {
          class: "px-4 py-2 bg-bg-secondary/50 border border-border/50 rounded text-sm text-text-muted",
          {text}
        }
      }
    }
    Step::Answer { .. } => {
      // Should not render Answer as step, but handle gracefully
      rsx! {
        div {
          class: "px-4 py-2 bg-bg-secondary rounded-lg text-sm text-text-muted",
          "..."
        }
      }
    }
  }
}
