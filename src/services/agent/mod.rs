//! MCP Agent Service
//! MCP 代理服务，负责工具调用与 AI 交互循环

mod agent;
mod executor;
mod parser;
mod stream;
mod types;

pub use agent::chat_with_tools;
pub use types::{AgentError, AgentStep, Result};
