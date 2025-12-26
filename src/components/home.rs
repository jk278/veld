//! Home page - Chat interface with session history
//! 首页 - AI 聊天对话界面（带历史会话）

use dioxus::prelude::*;
use crate::theme::use_theme;
use crate::config::AppConfig;
use crate::chat_history::ChatHistoryData;
use crate::components::chat::*;
use crate::components::chat::message_list::ChatMessage;

// Re-export for use in other modules
pub use crate::components::chat::UiSession;

/// Home page component - Chat interface with sidebar
#[component]
pub fn Home() -> Element {
    let _theme_mode = use_theme();

    // Chat messages state
    let messages = use_signal(Vec::<ChatMessage>::new);
    let input_text = use_signal(String::new);

    // Auto-scroll state
    let scroll_container_id = "chat-messages-container";
    let last_message_count = use_signal(|| 0);

    // Sidebar collapse state (persisted to config file)
    let mut sidebar_collapsed = use_signal(|| {
        AppConfig::load()
            .map(|c| c.ui.sidebar_collapsed)
            .unwrap_or(false)
    });

    // Persist sidebar state to config when changed
    use_effect(move || {
        let collapsed = sidebar_collapsed();
        if let Ok(mut config) = AppConfig::load() {
            config.update_sidebar_collapsed(collapsed);
        }
    });

    // Initialize scroll state tracking
    use_scroll_state_init(scroll_container_id.to_string());

    // Session history state
    let chat_history = use_signal(|| {
        ChatHistoryData::load().unwrap_or_default()
    });

    // Session list for sidebar (derived from history) - use_memo for auto-update
    let sessions = use_memo(move || {
        let history = chat_history();
        history.sessions.iter().map(|s| UiSession {
            id: s.id.clone(),
            title: s.title.clone(),
            is_current: history.current_session_id.as_ref() == Some(&s.id),
        }).collect::<Vec<_>>()
    });

    // Active provider and MCP server (cached, updated on switch)
    let active_provider_id = use_signal(|| {
        AppConfig::load()
            .ok()
            .and_then(|c| c.ai.active_provider)
            .unwrap_or_else(|| "claude".to_string())
    });

    // Sync messages with current session
    use_message_sync(messages.clone(), chat_history.clone());

    // Auto-scroll to bottom when new messages arrive
    use_auto_scroll(messages.clone(), last_message_count.clone(), scroll_container_id.to_string());

    // Chat coroutine for AI calls
    let tx = use_chat_coroutine(messages.clone(), chat_history.clone());

    // Create handlers
    let new_chat_handler = use_new_chat_handler(
        chat_history.clone(),
        messages.clone(),
        active_provider_id.clone(),
    );

    let switch_session = use_switch_session_handler(
        chat_history.clone(),
        messages.clone(),
    );

    let delete_session = use_delete_session_handler(chat_history.clone());

    let switch_provider = use_switch_provider_handler(active_provider_id.clone());

    let send_message_handler = use_send_message_handler(input_text.clone(), tx.clone());

    // Wrapper handlers for EventHandler compatibility (create closures that clone the handler)
    let new_chat_for_sidebar = {
        let mut handler = new_chat_handler.clone();
        move |_: MouseEvent| handler()
    };
    let new_chat_for_header = {
        let mut handler = new_chat_handler.clone();
        move |_: MouseEvent| handler()
    };
    let send_message = {
        let mut handler = send_message_handler.clone();
        move |_: MouseEvent| handler()
    };

    // Filter enabled providers and MCP servers
    let config = AppConfig::load().ok();
    let enabled_providers = config.as_ref()
        .map(|c| c.ai.providers.iter().filter(|p| p.enabled).cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let enabled_mcp_servers = config.as_ref()
        .map(|c| c.mcp.servers.iter().filter(|s| s.enabled).cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    // Get current provider info for rendering
    let (_active_provider_name, has_api_key) = get_active_provider_info();

    // Get current session title - use_memo for auto-update when session changes
    let current_session_title = use_memo(move || {
        chat_history().get_current_session()
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "New Chat".to_string())
    });

    // Get sessions list for rendering (clone to owned Vec to fix lifetime issues)
    let sessions_list = sessions().clone();

    rsx! {
        div {
            class: if sidebar_collapsed() {
                "flex flex-1 gap-0 overflow-hidden h-full"
            } else {
                "flex flex-1 max-w-6xl mx-auto gap-4 overflow-hidden h-full"
            },

            // Sidebar - Session History (collapsible)
            ChatSidebar {
                sessions: sessions_list,
                sidebar_collapsed: sidebar_collapsed(),
                on_new_chat: new_chat_for_sidebar,
                on_switch_session: switch_session,
                on_delete_session: delete_session,
            }

            // Main chat area
            div {
                class: "flex-1 flex flex-col bg-bg-primary border border-border rounded-lg overflow-hidden",

                // Header
                ChatHeader {
                    current_session_title: current_session_title(),
                    active_provider_id: active_provider_id(),
                    enabled_providers: enabled_providers.clone(),
                    enabled_mcp_servers: enabled_mcp_servers.clone(),
                    sidebar_collapsed: sidebar_collapsed(),
                    on_toggle_sidebar: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
                    on_new_chat: new_chat_for_header,
                    on_switch_provider: switch_provider,
                }

                // Messages area (scrollable)
                MessageList {
                    messages: messages.read().clone(),
                    has_api_key,
                    scroll_container_id: scroll_container_id.to_string(),
                }

                // Input area
                InputArea {
                    input_text: input_text.clone(),
                    has_api_key,
                    on_send: send_message,
                    tx: tx.clone(),
                }
            }
        }
    }
}

/// Helper function to get active provider info
///
/// IMPORTANT: A provider is only usable if it's enabled AND has a non-empty API key
fn get_active_provider_info() -> (String, bool) {
    let config_result = AppConfig::load();
    let (name, has_key) = match config_result {
        Ok(config) => {
            // Use get_usable_provider which checks: exists + enabled + has API key
            match config.get_usable_provider() {
                Some(provider) => (provider.name.clone(), true),
                None => {
                    // Active provider is not usable - show warning
                    let active_id = config.ai.active_provider.as_deref().unwrap_or("none");
                    eprintln!("[WARN] Active provider '{}' is not usable (missing, disabled, or no API key)", active_id);
                    ("No Usable Provider".to_string(), false)
                }
            }
        }
        Err(_) => ("No Provider".to_string(), false),
    };
    println!("[DEBUG] Provider state: name={}, has_key={}", name, has_key);
    (name, has_key)
}
