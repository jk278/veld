//! Chat header component - minimal, title only
//! 聊天头部组件 - 极简设计，仅显示标题

use dioxus::prelude::*;

/// Chat header with title only (minimal)
/// 所有交互元素已移至底部输入区域
#[component]
pub fn ChatHeader(
  current_session_title: String,
  sidebar_collapsed: bool,
  on_toggle_sidebar: EventHandler<MouseEvent>,
  on_new_chat: EventHandler<MouseEvent>,
) -> Element {
  rsx! {
    div {
      class: "flex items-center justify-between px-4 py-3 border-b border-border relative z-10 shadow-custom",

      // Left side - Collapse button and Title
      div {
        class: "flex items-center gap-3 flex-1 min-w-0",
        // Collapse toggle button
        button {
          class: "w-8 h-8 flex items-center justify-center rounded-full hover:bg-bg-surface text-text-secondary transition-colors flex-shrink-0",
          onclick: on_toggle_sidebar,
          "☰"
        }
        h2 {
          class: "text-base font-medium text-text-primary truncate",
          "{current_session_title}"
        }
      }

      // Right side - New Chat button
      button {
        class: "w-8 h-8 flex items-center justify-center rounded-full bg-bg-surface hover:bg-bg-secondary text-text-secondary hover:text-text-primary transition-colors",
        onclick: on_new_chat,
        "＋"
      }
    }
  }
}
