//! Services module
//! 服务模块

pub mod ai_client;

pub use ai_client::{AiClient, AiError, ChatMessage, user_message, system_message, assistant_message};
