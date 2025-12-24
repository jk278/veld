//! Home page - Chat interface with session history
//! 首页 - AI 聊天对话界面（带历史会话）

use dioxus::prelude::*;
use dioxus::document;
use crate::services::AiClient;
use crate::theme::use_theme;
use crate::config::AppConfig;
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use crate::components::markdown::{MarkdownContent, PlainTextContent};
use std::time::SystemTime;
use futures_util::stream::StreamExt;

/// Chat message for display
#[derive(Clone, Debug, PartialEq)]
struct ChatMessage {
    id: String,
    role: String,
    content: String,
    timestamp: u64,
}

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

/// Chat session for UI display
#[derive(Clone, Debug, PartialEq)]
struct UiSession {
    id: String,
    title: String,
    is_current: bool,
}

/// Home page component - Chat interface with sidebar
#[component]
pub fn Home() -> Element {
    let _theme_mode = use_theme();

    // Chat messages state
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut input_text = use_signal(String::new);

    // Auto-scroll state
    let scroll_container_id = "chat-messages-container";
    let mut last_message_count = use_signal(|| 0);

    // Setup scroll state tracking in JavaScript
    use_effect(move || {
        let container_id = scroll_container_id.to_string();
        document::eval(format!(
            r#"(function() {{
                const container = document.getElementById("{}");
                if (!container) return;

                // Initialize scroll state
                window.__veldScrollState = 'auto'; // 'auto' or 'manual'

                container.addEventListener('scroll', () => {{
                    const scrollTop = container.scrollTop;
                    const scrollHeight = container.scrollHeight;
                    const clientHeight = container.clientHeight;
                    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

                    // If user scrolled up more than 150px from bottom, switch to manual mode
                    if (distanceFromBottom > 150) {{
                        window.__veldScrollState = 'manual';
                    }} else if (distanceFromBottom < 50) {{
                        // User scrolled back near bottom, switch back to auto mode
                        window.__veldScrollState = 'auto';
                    }}
                }}, {{ passive: true }});
            }})()"#,
            container_id
        ).as_str());
    });

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

    // Get all providers and active provider
    let providers = use_signal(|| {
        AppConfig::load()
            .ok()
            .map(|c| c.ai.providers)
            .unwrap_or_default()
    });

    let active_provider_id = use_signal(|| {
        AppConfig::load()
            .ok()
            .and_then(|c| c.ai.active_provider)
            .unwrap_or_else(|| "claude".to_string())
    });

    // Sync messages with current session (when chat_history changes)
    use_effect(move || {
        let _ = chat_history(); // Track chat_history dependency
        if let Some(session) = chat_history().get_current_session() {
            let current_msgs: Vec<ChatMessage> = session.messages.iter().cloned().map(Into::into).collect();
            if messages() != current_msgs {
                messages.set(current_msgs);
            }
        }
    });

    // Auto-scroll to bottom when new messages arrive
    use_effect(move || {
        let current_count = messages().len();
        let prev_count = last_message_count();

        // Update last message count
        if current_count != prev_count {
            last_message_count.set(current_count);
        }

        // Only scroll if new messages were added
        if current_count > prev_count {
            let container_id = scroll_container_id;
            document::eval(format!(
                r#"(function() {{
                    const container = document.getElementById("{}");
                    if (!container) return;

                    // Check if we should auto-scroll (only if currently in auto mode)
                    if (window.__veldScrollState === 'auto') {{
                        container.scrollTo({{ top: container.scrollHeight, behavior: "smooth" }});
                    }}
                }})()"#,
                container_id
            ).as_str());
        }
    });

    // Helper function to get active provider info
    let get_active_provider_info = move || {
        let active_id = active_provider_id();
        let providers_list = providers();
        let provider = providers_list.iter().find(|p| p.id == active_id);
        let name = provider.map(|p| p.name.clone()).unwrap_or_else(|| "No Provider".to_string());
        let has_key = provider.and_then(|p| p.api_key.as_ref()).map_or(false, |k| !k.is_empty());
        (name, has_key)
    };

    // Use coroutine for async AI calls
    let tx = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        let mut messages = messages.clone();
        let mut chat_history = chat_history.clone();
        async move {
            while let Some(text) = rx.next().await {
                let text: String = text;

                // Add user message
                let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                let user_msg_id = format!("msg-{}", now_millis);
                let user_msg = ChatMessage {
                    id: user_msg_id.clone(),
                    role: "user".to_string(),
                    content: text.clone(),
                    timestamp: now_secs,
                };
                messages.push(user_msg.clone());

                // Update history
                chat_history.write().add_message(HistoryMessage {
                    id: user_msg_id.clone(),
                    role: "user".to_string(),
                    content: text.clone(),
                    timestamp: now_secs,
                });
                let history_clone = { (*chat_history.read()).clone() };
                let _ = chat_history.read().save();
                // Trigger UI update for session list
                chat_history.set(history_clone);

                // Build message history for API (exclude system errors)
                let api_messages = messages.read().iter().filter(|m| m.role != "system").map(|m| {
                    crate::services::ChatMessage {
                        role: m.role.clone(),
                        content: m.content.clone(),
                    }
                }).collect();

                // Call AI API
                match AiClient::chat_completion(api_messages).await {
                    Ok(response) => {
                        let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                        let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                        let assistant_msg_id = format!("msg-{}", now_millis);
                        let assistant_msg = ChatMessage {
                            id: assistant_msg_id.clone(),
                            role: "assistant".to_string(),
                            content: response,
                            timestamp: now_secs,
                        };
                        messages.push(assistant_msg.clone());

                        // Update history
                        chat_history.write().add_message(HistoryMessage {
                            id: assistant_msg_id.clone(),
                            role: "assistant".to_string(),
                            content: assistant_msg.content.clone(),
                            timestamp: now_secs,
                        });
                        let history_clone = { (*chat_history.read()).clone() };
                        let _ = chat_history.read().save();
                        // Trigger UI update for session list
                        chat_history.set(history_clone);
                    }
                    Err(e) => {
                        let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                        let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                        let error_msg = ChatMessage {
                            id: format!("msg-{}", now_millis),
                            role: "system".to_string(),
                            content: format!("Error: {}", e),
                            timestamp: now_secs,
                        };
                        messages.push(error_msg);
                    }
                }
            }
        }
    });

    // Switch provider handler
    let mut switch_provider = {
        let mut active_provider_id = active_provider_id.clone();
        move |provider_id: String| {
            if let Ok(mut config) = AppConfig::load() {
                config.set_active_provider(provider_id.clone());
                active_provider_id.set(provider_id);
            }
        }
    };

    // New chat handler
    let new_chat = {
        let mut chat_history = chat_history.clone();
        let mut messages = messages.clone();
        let active_provider_id = active_provider_id.clone();
        move |_| {
            // First, save current messages to current session before creating new one
            let current_msgs = messages();
            let mut history = chat_history.write();
            if let Some(session) = history.get_current_session_mut() {
                let history_msgs: Vec<HistoryMessage> = current_msgs.into_iter().map(Into::into).collect();
                session.messages = history_msgs;
            }
            let _ = history.save();

            // Then create new session
            let provider_id = active_provider_id();
            let _new_session_id = history.create_new_session(&provider_id);
            let _ = history.save();
            messages.set(vec![]);

            // Trigger UI update by cloning and dropping the borrow first
            let history_clone = (*history).clone();
            drop(history);
            chat_history.set(history_clone);
        }
    };

    // Switch session handler
    let mut switch_session = {
        let mut chat_history = chat_history.clone();
        let messages = messages.clone();
        move |session_id: String| {
            // Save current messages before switching
            let current_msgs = messages();
            let mut history = chat_history.write();
            if let Some(session) = history.get_current_session_mut() {
                let history_msgs: Vec<HistoryMessage> = current_msgs.into_iter().map(Into::into).collect();
                session.messages = history_msgs;
            }
            let _ = history.save();

            // Then switch to the target session
            history.switch_session(&session_id);
            let _ = history.save();

            // Trigger UI update by cloning and dropping the borrow first
            let history_clone = (*history).clone();
            drop(history);
            chat_history.set(history_clone);
        }
    };

    // Delete session handler
    let mut delete_session = {
        let mut chat_history = chat_history.clone();
        move |session_id: String| {
            let mut history = chat_history.write();
            history.delete_session(&session_id);
            let _ = history.save();

            // Trigger UI update by cloning and dropping the borrow first
            let history_clone = (*history).clone();
            drop(history);
            chat_history.set(history_clone);
        }
    };

    // Clear current chat handler
    let clear_chat = {
        let mut chat_history = chat_history.clone();
        let mut messages = messages.clone();
        move |_| {
            // Save current messages before clearing
            let current_msgs = messages();
            let mut history = chat_history.write();
            if let Some(session) = history.get_current_session_mut() {
                let history_msgs: Vec<HistoryMessage> = current_msgs.into_iter().map(Into::into).collect();
                session.messages = history_msgs;
            }
            let _ = history.save();

            // Then clear the session
            history.clear_current_session();
            let _ = history.save();
            messages.set(vec![]);

            // Trigger UI update by cloning and dropping the borrow first
            let history_clone = (*history).clone();
            drop(history);
            chat_history.set(history_clone);
        }
    };

    // Send message handler
    let send_message = {
        let mut input_text = input_text.clone();
        let tx_clone = tx.clone();
        move |_: MouseEvent| {
            let text = input_text().trim().to_string();
            if text.is_empty() {
                return;
            }
            input_text.set(String::new());
            tx_clone.send(text);
        }
    };

    // Filter enabled providers only
    let enabled_providers = providers().iter().filter(|p| p.enabled).cloned().collect::<Vec<_>>();

    // Get current provider info for rendering
    let (active_provider_name, has_api_key) = get_active_provider_info();

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
            class: "flex h-[calc(100vh-120px)] max-w-6xl mx-auto gap-4",

            // Sidebar - Session History
            div {
                class: "w-64 flex flex-col bg-bg-surface border border-border rounded-lg overflow-hidden",

                // Sidebar header
                div {
                    class: "p-4 border-b border-border",
                    button {
                        class: "w-full btn-primary flex items-center justify-center gap-2",
                        onclick: new_chat,
                        span { class: "text-lg", "➕" }
                        "New Chat"
                    }
                }

                // Session list
                div {
                    class: "flex-1 overflow-y-auto p-2 space-y-1",
                    if sessions_list.is_empty() {
                        p {
                            class: "text-sm text-text-muted text-center py-4",
                            "No chat history yet"
                        }
                    } else {
                        for session in sessions_list.into_iter() {
                            div {
                                class: "group relative flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer transition-colors",
                                class: if session.is_current {
                                    "bg-primary/10 border border-primary/30"
                                } else {
                                    "hover:bg-bg-primary border border-transparent"
                                },
                                onclick: {
                                    let sid = session.id.clone();
                                    move |_| {
                                        switch_session(sid.clone());
                                    }
                                },

                                span {
                                    class: "flex-1 text-sm truncate",
                                    class: if session.is_current { "text-text-primary font-medium" } else { "text-text-secondary" },
                                    {session.title.clone()}
                                }

                                // Delete button (shown on hover)
                                if !session.is_current {
                                    button {
                                        class: "opacity-0 group-hover:opacity-100 w-6 h-6 flex items-center justify-center rounded hover:bg-error/10 text-text-muted hover:text-error transition-all",
                                        onclick: {
                                            let sid = session.id.clone();
                                            move |e: MouseEvent| {
                                                e.stop_propagation();
                                                delete_session(sid.clone());
                                            }
                                        },
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Main chat area
            div {
                class: "flex-1 flex flex-col bg-bg-surface border border-border rounded-lg overflow-hidden",

                // Header
                div {
                    class: "flex items-center justify-between px-4 py-3 border-b border-border",

                    // Left side - Title and Provider Selector
                    div {
                        class: "flex items-center gap-3",
                        div {
                            class: "w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0",
                            span { class: "text-sm", "🤖" }
                        }
                        div {
                            h2 {
                                class: "text-lg font-semibold text-text-primary",
                                "{current_session_title()}"
                            }
                            // Provider selector
                            if !enabled_providers.is_empty() {
                                div {
                                    class: "flex items-center gap-1 mt-0.5",
                                    select {
                                        class: "text-xs bg-bg-surface text-text-secondary border border-border rounded px-2 py-0.5 focus:border-primary focus:outline-none cursor-pointer",
                                        value: active_provider_id(),
                                        onchange: move |e| {
                                            switch_provider(e.value());
                                        },

                                        for provider in enabled_providers.iter() {
                                            option {
                                                value: provider.id.clone(),
                                                {provider.name.clone()}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Right side - Clear button
                    button {
                        class: "px-2 py-1 text-xs bg-bg-secondary text-text-secondary rounded border border-border hover:bg-bg-primary hover:text-text-primary transition-colors",
                        onclick: clear_chat,
                        "Clear"
                    }
                }

                // Messages area (scrollable)
                div {
                    id: scroll_container_id,
                    class: "flex-1 overflow-y-auto px-4 py-4 space-y-4",

                    if messages().is_empty() {
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
                                        href: "/ai-config",
                                        "AI Config"
                                    }
                                }
                            }
                        }
                    } else {
                        for msg in messages().iter() {
                            div {
                                class: if msg.role == "user" {
                                    "flex justify-end"
                                } else if msg.role == "system" {
                                    "flex justify-center"
                                } else {
                                    "flex justify-start"
                                },

                                if msg.role == "system" {
                                    div {
                                        class: "px-4 py-2 bg-error/10 border border-error/30 rounded-lg text-sm text-error max-w-md",
                                        {msg.content.clone()}
                                    }
                                } else if msg.role == "user" {
                                    div {
                                        class: "max-w-2xl",
                                        div {
                                            class: "flex items-start gap-2 justify-end",
                                            div {
                                                class: "px-4 py-2.5 bg-primary text-white rounded-2xl rounded-tr-md",
                                                style: "max-width: 80%;",
                                                PlainTextContent {
                                                    content: msg.content.clone(),
                                                    class: "text-sm leading-relaxed".to_string()
                                                }
                                            }
                                            div {
                                                class: "w-7 h-7 rounded-full bg-primary/20 flex items-center justify-center text-xs shrink-0",
                                                "👤"
                                            }
                                        }
                                        p {
                                            class: "text-xs text-text-muted mt-1 text-right",
                                            {format_timestamp(msg.timestamp)}
                                        }
                                    }
                                } else {
                                    div {
                                        class: "max-w-2xl",
                                        div {
                                            class: "flex items-start gap-2",
                                            div {
                                                class: "w-7 h-7 rounded-full bg-bg-secondary flex items-center justify-center text-xs shrink-0",
                                                "🤖"
                                            }
                                            div {
                                                class: "px-4 py-2.5 bg-bg-surface border border-border rounded-2xl rounded-tl-md markdown-body",
                                                style: "max-width: 80%;",
                                                MarkdownContent {
                                                    content: msg.content.clone(),
                                                    class: "text-sm text-text-primary".to_string()
                                                }
                                            }
                                        }
                                        p {
                                            class: "text-xs text-text-muted mt-1",
                                            {format_timestamp(msg.timestamp)}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Input area
                div {
                    class: "border-t border-border px-4 py-3",
                    div {
                        class: "flex gap-2 items-end",

                        div {
                            class: "flex-1 relative",
                            textarea {
                                class: "w-full p-3 pr-10 bg-bg-primary text-text-primary border border-border rounded-lg resize-none focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all font-mono text-sm",
                                rows: 1,
                                placeholder: if !has_api_key {
                                    "Configure API key first..."
                                } else {
                                    "Type your message... (Enter to send)"
                                },
                                value: input_text(),
                                disabled: !has_api_key,
                                oninput: move |e| input_text.set(e.value()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter && has_api_key {
                                        e.prevent_default();
                                        let text = input_text().trim().to_string();
                                        if !text.is_empty() {
                                            input_text.set(String::new());
                                            tx.send(text);
                                        }
                                    }
                                },
                            }
                            // Character count
                            if !input_text().is_empty() {
                                span {
                                    class: "absolute bottom-2 right-2 text-xs text-text-muted",
                                    {format!("{}c", input_text().chars().count())}
                                }
                            }
                        }

                        button {
                            class: "px-4 py-2.5 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm font-medium",
                            disabled: !has_api_key || input_text().trim().is_empty(),
                            onclick: send_message,
                            span { "📤" }
                            "Send"
                        }
                    }

                    // Helper text
                    p {
                        class: "text-xs text-text-muted text-center mt-2",
                        if has_api_key {
                            "Using {active_provider_name} · History saved automatically"
                        } else {
                            "Configure API key to start"
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
