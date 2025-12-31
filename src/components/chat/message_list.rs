//! Message list component
//! 消息列表组件 - 显示聊天消息

use crate::components::markdown::{MarkdownContent, PlainTextContent};
use crate::theme::use_is_dark;
use dioxus::prelude::*;

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
) -> Element {
  rsx! {
    div {
      id: scroll_container_id,
      class: "flex-1 overflow-y-auto px-4 py-4 space-y-4",

      if messages.is_empty() {
        EmptyState {
          has_api_key,
        }
      } else {
        for msg in messages.into_iter() {
          MessageBubble {
            message: msg,
            edit_state: edit_state.clone(),
            on_edit: on_edit.clone(),
            on_regenerate: on_regenerate.clone(),
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
      span {
        class: "text-5xl",
        "💬"
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
) -> Element {
  // Check if this is an intermediate step (contains bullet point marker)
  let is_intermediate = message.content.contains("• ");

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
          is_intermediate,
          message_id: message.id.clone(),
          on_regenerate: on_regenerate.clone(),
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

  // Clone values for closures
  let message_id_key = message_id.clone();
  let message_id_confirm = message_id.clone();
  let message_id_cancel = message_id.clone();
  let message_id_edit = message_id.clone();
  let content_clone = content.clone();

  rsx! {
    div {
      class: "w-[80%] max-w-2xl flex justify-end flex-col items-end gap-2",

      // 编辑模式：独立编辑区域
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
                  handler((message_id_confirm.clone(), local_edit()));
                }
              },
              "保存 (Ctrl+Enter)"
            }
          }
        }
      } else {
        // TODO: 选中用户消息复制时末尾多一个换行符（浏览器块级元素复制行为）
        // 已优化：p→span（PlainTextContent）
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

/// Parse a thinking step line, returning (short, optional_detail)
fn parse_thinking_step(line: &str) -> Option<(String, Option<String>)> {
  line.strip_prefix("• ").map(|step_text| {
    if let Some(idx) = step_text.find('|') {
      let short = step_text[..idx].to_string();
      let detail = step_text[idx + 1..].to_string();
      (short, Some(detail))
    } else {
      (step_text.to_string(), None)
    }
  })
}

/// Assistant message bubble
#[component]
fn AssistantMessageBubble(
  content: String,
  timestamp: u64,
  is_intermediate: bool,
  #[props(default)] message_id: String,
  #[props(default)] on_regenerate: Option<EventHandler<String>>,
) -> Element {
  let is_dark = use_is_dark();
  let mut copy_status = use_signal(|| false);

  // Use dioxus's spawn for async tasks in the UI context
  use dioxus::prelude::spawn;

  // Count steps for the summary
  let step_count = content.matches("• ").count();

  // Copy handler using arboard (native clipboard)
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

  // Create regenerate handler outside rsx - needs to own the handler
  let show_regenerate = on_regenerate.is_some();
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
      if is_intermediate {
        // Intermediate steps: collapsible panel
        div {
          class: "thinking-process",
          details {

            summary {
              class: "thinking-summary cursor-pointer",
              "思考过程"
              if step_count > 0 {
                span {
                  class: "ml-2 text-text-muted",
                  "({step_count}步)"
                }
              }
            }
            div {
              class: "thinking-content",
              // Parse and render each step (split by bullet point, not newline)
              // Filter out empty items before rendering
              for part in content
                  .split("• ")
                  .skip(1)
                  .filter_map(|part| {
                      parse_thinking_step(&format!("• {}", part))
                          .filter(|(short, _)| !short.trim().is_empty())
                  })
              {
                div {
                  class: "thinking-item",
                  div {
                    class: "thinking-item-short",
                    MarkdownContent {
                      content: part.0.clone(),
                      class: String::new(),
                      dark: is_dark(),
                    }
                  }
                  if let Some(detail_text) = &part.1 {
                    div {
                      class: "thinking-item-detail",
                      MarkdownContent {
                        content: detail_text.clone(),
                        class: String::new(),
                        dark: is_dark(),
                      }
                    }
                  }
                }
              }
            }
          }
        }
      } else {
        // Final answer: markdown rendering with action buttons at bottom
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

/// Format timestamp for display
fn format_timestamp(timestamp: u64) -> String {
  use chrono::{DateTime, Local, Utc};
  let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0);
  if let Some(utc) = dt {
    let local: DateTime<Local> = utc.into();
    local.format("%H:%M").to_string()
  } else {
    "??:??".to_string()
  }
}
