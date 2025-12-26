//! Chat interface components
//! 聊天界面组件模块

pub mod sidebar;
pub mod message_list;
pub mod input_area;
pub mod header;
pub mod hooks;
pub mod handlers;

// Re-export commonly used components
pub use sidebar::ChatSidebar;
pub use message_list::{MessageList, EmptyState};
pub use input_area::{ChatInput, InputArea};
pub use header::ChatHeader;

// Re-export hooks
pub use hooks::{use_chat_coroutine, use_message_sync, use_auto_scroll, use_scroll_state_init};

// Re-export handlers
pub use handlers::{
    use_new_chat_handler,
    use_switch_session_handler,
    use_delete_session_handler,
    use_switch_provider_handler,
    use_send_message_handler,
};

// Shared types
/// Chat session for UI display
#[derive(Clone, Debug, PartialEq)]
pub struct UiSession {
    pub id: String,
    pub title: String,
    pub is_current: bool,
}
