//! Home page - Chat interface with session history
//! 首页 - AI 聊天对话界面（带历史会话）

use dioxus::prelude::*;
use dioxus::document;
use crate::services::{chat_with_tools, AgentStep};
use crate::theme::use_theme;
use crate::config::AppConfig;
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use crate::components::markdown::{MarkdownContent, PlainTextContent};
use std::time::SystemTime;
use futures_util::stream::StreamExt;
use tokio::sync::mpsc;

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

    // Active provider and MCP server (cached, updated on switch)
    let active_provider_id = use_signal(|| {
        AppConfig::load()
            .ok()
            .and_then(|c| c.ai.active_provider)
            .unwrap_or_else(|| "claude".to_string())
    });

    // Sync messages with current session (when chat_history changes)
    //
    // ALERT: CRITICAL - Agent step detection logic must stay in sync with step formatting below!
    // If you modify the step format (emojis/prefixes), you MUST update BOTH places:
    // 1. The placeholder detection here (has_unsaved_placeholder check)
    // 2. The step formatting in the use_coroutine block (around line 290-310)
    //
    // Otherwise, use_effect will overwrite in-progress agent updates, causing steps to flicker/disappear.
    use_effect(move || {
        let _ = chat_history(); // Track chat_history dependency
        if let Some(session) = chat_history().get_current_session() {
            let current_msgs: Vec<ChatMessage> = session.messages.iter().cloned().map(Into::into).collect();

            // Check if messages has an unsaved placeholder (agent in progress)
            // Format: "- 🔌", "- 🤔", "- 🔧", "- ✅"
            let has_unsaved_placeholder = messages().iter().any(|m| {
                m.content == "思考中..." ||
                m.content.contains("- 🔌") ||
                m.content.contains("- 🤔") ||
                m.content.contains("- 🔧") ||
                m.content.contains("- ✅")
            });

            // Only sync if there's no in-progress agent
            if !has_unsaved_placeholder && messages() != current_msgs {
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
    // IMPORTANT: A provider is only usable if it's enabled AND has a non-empty API key
    let get_active_provider_info = move || {
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
    };

    // Use coroutine for async AI calls
    let tx = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        let mut messages = messages.clone();
        let mut chat_history = chat_history.clone();
        let mut msg_counter: u64 = 0;  // Message counter for unique IDs
        async move {
            while let Some(text) = rx.next().await {
                let text: String = text;

                // Add user message
                let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                msg_counter += 1;
                let user_msg_id = format!("msg-{}-{}", now_millis, msg_counter);
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
                let api_messages: Vec<crate::services::ChatMessage> = messages.read().iter().filter(|m| m.role != "system").map(|m| {
                    crate::services::ChatMessage {
                        role: m.role.clone(),
                        content: m.content.clone(),
                    }
                }).collect();

                // Create channel for streaming AgentStep updates
                let (step_tx, mut step_rx) = mpsc::unbounded_channel::<AgentStep>();

                // Create temporary assistant message ID for streaming updates
                let now_millis = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                let now_secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                msg_counter += 1;
                let assistant_msg_id = format!("msg-{}-{}", now_millis, msg_counter);

                // Add initial placeholder message
                messages.push(ChatMessage {
                    id: assistant_msg_id.clone(),
                    role: "assistant".to_string(),
                    content: "思考中...".to_string(),
                    timestamp: now_secs,
                });

                // Track intermediate steps and final answer
                // IMPORTANT: Don't update chat_history during agent execution to avoid
                // triggering the use_effect sync that overwrites our message updates
                let mut intermediate_steps = Vec::new();
                let mut final_response = String::new();

                eprintln!("=== STARTING AGENT TASK ===");
                let api_messages_clone = api_messages.clone();

                // Spawn agent in background (but process steps in this coroutine context)
                tokio::spawn(async move {
                    let _ = chat_with_tools(api_messages_clone, step_tx).await;
                });

                // Process steps as they arrive
                eprintln!("=== ENTERING STEP LOOP ===");
                let mut step_count = 0;

                while let Some(step) = step_rx.recv().await {
                    step_count += 1;
                    eprintln!("=== PROCESSING STEP {} ===", step_count);

                    // ALERT: STEP FORMAT - If you modify these formats, update the placeholder detection
                    // in use_effect above (line ~150) to match! Otherwise steps will flicker/disappear.
                    match step {
                        AgentStep::Connecting(msg) => {
                            intermediate_steps.push(format!("- 🔌 {}", msg));
                        }
                        AgentStep::Thinking { short, content } => {
                            // Use collapsible details if there's content, otherwise just show short text
                            if let Some(thought_content) = content {
                                intermediate_steps.push(format!("- 🤔 {}\n<details><summary>查看思考内容</summary>\n{}\n</details>", short, thought_content));
                            } else {
                                intermediate_steps.push(format!("- 🤔 {}", short));
                            }
                        }
                        AgentStep::ToolCall { name, .. } => {
                            intermediate_steps.push(format!("- 🔧 调用: {}", name));
                        }
                        AgentStep::ToolResult { name, .. } => {
                            intermediate_steps.push(format!("- ✅ 完成: {}", name));
                        }
                        AgentStep::Final(text) => {
                            final_response = text.clone();
                            // NOTE: Don't update chat_history yet - do it after the loop
                        }
                    }

                    // When final response arrives, create a separate message bubble for it
                    if !final_response.is_empty() {
                        eprintln!("=== FINAL RESPONSE RECEIVED, CREATING NEW MESSAGE ===");

                        // First, update the steps message to show completion
                        let current_msgs = messages.read().clone();
                        if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
                            let mut updated = current_msgs;
                            updated[pos].content = intermediate_steps.join("\n");
                            messages.set(updated);
                        }

                        // Then, create a new message for the final response
                        let now_millis_final = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
                        let now_secs_final = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                        msg_counter += 1;
                        let final_msg_id = format!("msg-{}-{}", now_millis_final, msg_counter);

                        messages.push(ChatMessage {
                            id: final_msg_id.clone(),
                            role: "assistant".to_string(),
                            content: final_response.clone(),
                            timestamp: now_secs_final,
                        });

                        // Save both messages to history
                        chat_history.write().add_message(HistoryMessage {
                            id: assistant_msg_id.clone(),
                            role: "assistant".to_string(),
                            content: intermediate_steps.join("\n"),
                            timestamp: now_secs,
                        });
                        chat_history.write().add_message(HistoryMessage {
                            id: final_msg_id.clone(),
                            role: "assistant".to_string(),
                            content: final_response.clone(),
                            timestamp: now_secs_final,
                        });
                        let history_clone = { (*chat_history.read()).clone() };
                        let _ = chat_history.read().save();
                        chat_history.set(history_clone);

                        // Break out of the loop since we're done
                        break;
                    }

                    // Update the steps message (still in progress)
                    let display_content = intermediate_steps.join("\n");
                    eprintln!("=== UPDATING SIGNAL (step {}, content length: {}) ===", step_count, display_content.len());

                    let current_msgs = messages.read().clone();
                    if let Some(pos) = current_msgs.iter().position(|m| m.id == assistant_msg_id) {
                        let mut updated = current_msgs;
                        updated[pos].content = display_content;
                        updated[pos].timestamp = now_secs;
                        messages.set(updated);
                        eprintln!("=== MESSAGE UPDATED (via set) ===");
                    } else {
                        eprintln!("=== WARNING: MESSAGE NOT FOUND IN LIST ===");
                        drop(current_msgs);
                        messages.push(ChatMessage {
                            id: assistant_msg_id.clone(),
                            role: "assistant".to_string(),
                            content: display_content,
                            timestamp: now_secs,
                        });
                    }
                }
                eprintln!("=== STEP LOOP DONE ===");
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
            // Don't create new session if current one is empty
            if messages().is_empty() {
                return;
            }

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
        let mut messages = messages.clone();
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

            // Load target session's messages directly (bypass use_effect placeholder check)
            if let Some(session) = history.get_current_session() {
                let session_msgs: Vec<ChatMessage> = session.messages.iter().cloned().map(Into::into).collect();
                messages.set(session_msgs);
            }

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

    // Filter enabled providers and MCP servers
    let config = AppConfig::load().ok();
    let enabled_providers = config.as_ref()
        .map(|c| c.ai.providers.iter().filter(|p| p.enabled).cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let enabled_mcp_servers = config.as_ref()
        .map(|c| c.mcp.servers.iter().filter(|s| s.enabled).cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    // Get current provider info for rendering
    let (active_provider_name, has_api_key) = get_active_provider_info();

    // Debug: Log final state
    println!("[DEBUG] Rendering with: provider={}, has_api_key={}", active_provider_name, has_api_key);

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
                "flex flex-1 gap-0 overflow-hidden"
            } else {
                "flex flex-1 max-w-6xl mx-auto gap-4 overflow-hidden"
            },

            // Sidebar - Session History (collapsible)
            div {
                class: if sidebar_collapsed() {
                    "w-0 min-w-0 flex flex-col bg-bg-surface border border-border rounded-lg overflow-hidden opacity-0"
                } else {
                    "w-64 min-w-0 flex flex-col bg-bg-surface border border-border rounded-lg overflow-hidden opacity-100"
                },
                // Always apply transition for both collapse and expand
                class: "transition-all duration-300 ease-in-out",


                // Sidebar header
                div {
                    class: "p-4",
                    button {
                        class: "w-full btn-primary flex items-center justify-center gap-2 whitespace-nowrap",
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
                    class: "flex items-center justify-between px-4 py-3",

                    // Left side - Collapse button, Title and Provider Selector
                    div {
                        class: "flex items-center gap-3",
                        // Collapse toggle button
                        button {
                            class: "w-8 h-8 flex items-center justify-center rounded hover:bg-bg-primary text-text-muted hover:text-text-primary transition-colors",
                            onclick: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
                            span { class: "text-lg", if sidebar_collapsed() { "☰" } else { "«" } }
                        }
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

                            // MCP status badges (enabled servers available to AI)
                            if !enabled_mcp_servers.is_empty() {
                                div {
                                    class: "flex items-center gap-1 mt-0.5",
                                    span {
                                        class: "text-xs text-text-muted",
                                        "MCP:"
                                    }
                                    for server in enabled_mcp_servers.iter() {
                                        span {
                                            class: "text-xs bg-success/10 text-success border border-success/30 rounded px-1.5 py-0.5 font-mono",
                                            {server.name.clone()}
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Right side - New Chat button
                    button {
                        class: "px-3 py-1 text-xs bg-primary text-white rounded border border-border hover:bg-primary/90 transition-colors flex items-center gap-1",
                        onclick: new_chat,
                        span { class: "text-sm", "＋" }
                        "New Chat"
                    }
                }

                // Messages area (scrollable)
                div {
                    id: scroll_container_id,
                    class: "flex-1 overflow-y-auto px-4 py-4 space-y-4",

                    if messages.read().is_empty() {
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
                        for msg in messages.read().iter() {
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
                // NOTE: textarea must be direct child of flex (no wrapper div) to avoid 6px ghost height issue
                div {
                    class: "px-4 py-3",
                    div {
                        class: "flex gap-2",

                        textarea {
                            class: "flex-1 px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg resize-none focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all font-mono text-sm",
                            rows: 1,
                            placeholder: if !has_api_key {
                                "Configure API key first..."
                            } else {
                                "Type your message..."
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

                        button {
                            class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm font-medium",
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
