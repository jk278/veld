//! Settings page submodule
//! 设置页面子模块

pub mod ai_providers;
pub mod mcp_servers;
pub mod appearance;
pub mod shortcuts;

// Re-export tab components
pub use ai_providers::AiProvidersTab;
pub use mcp_servers::McpServersTab;
pub use appearance::AppearanceTab;
pub use shortcuts::ShortcutsTab;
