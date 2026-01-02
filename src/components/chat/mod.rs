//! Chat interface components
//! 聊天界面组件模块

pub mod handlers;
pub mod header;
pub mod hooks;
pub mod input_area;
pub mod message_list;
pub mod sidebar;

// Re-export commonly used components
pub use header::ChatHeader;
pub use input_area::{ChatInput, InputArea};
pub use message_list::{EmptyState, MessageList, MessageEdit};
pub use sidebar::ChatSidebar;

// Re-export hooks
pub use hooks::{
  abort_streaming, use_auto_scroll, use_chat_coroutine, use_chat_coroutine_with_prefix,
  use_message_sync, use_regenerate_coroutine, use_scroll_state_init, use_streaming_scroll,
};

// Re-export handlers
pub use handlers::{
  use_delete_session_handler, use_new_chat_handler, use_send_message_handler,
  use_switch_provider_handler, use_switch_session_handler,
};

// Shared types
/// Chat session for UI display
#[derive(Clone, Debug, PartialEq)]
pub struct UiSession {
  pub id: String,
  pub title: String,
  pub is_current: bool,
}
