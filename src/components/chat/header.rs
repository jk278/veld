//! Chat header component
//! 聊天头部组件 - 标题、提供商选择器、MCP 状态

use dioxus::prelude::*;

/// Chat header with title, provider selector, and MCP badges
#[component]
pub fn ChatHeader(
    current_session_title: String,
    active_provider_id: String,
    enabled_providers: Vec<crate::config::ProviderConfig>,
    enabled_mcp_servers: Vec<crate::config::McpServerConfig>,
    sidebar_collapsed: bool,
    on_toggle_sidebar: EventHandler<MouseEvent>,
    on_new_chat: EventHandler<MouseEvent>,
    on_switch_provider: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between px-4 py-3 border-b border-border relative z-10 shadow-custom",

            // Left side - Collapse button, Title and Provider Selector
            div {
                class: "flex items-center gap-3",
                // Collapse toggle button
                button {
                    class: "w-8 h-8 flex items-center justify-center rounded hover:bg-bg-primary text-text-muted hover:text-text-primary transition-colors",
                    onclick: on_toggle_sidebar,
                    span { class: "text-lg", if sidebar_collapsed { "☰" } else { "«" } }
                }
                div {
                    class: "w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0",
                    span { class: "text-sm", "🤖" }
                }
                div {
                    h2 {
                        class: "text-lg font-semibold text-text-primary",
                        "{current_session_title}"
                    }
                    // Provider selector and MCP badges (horizontal)
                    if !enabled_providers.is_empty() || !enabled_mcp_servers.is_empty() {
                        div {
                            class: "flex items-center gap-2 mt-0.5",
                            // Provider selector
                            if !enabled_providers.is_empty() {
                                select {
                                    class: "text-xs bg-bg-surface text-text-secondary border border-border rounded px-2 py-0.5 focus:border-primary focus:outline-none cursor-pointer",
                                    value: active_provider_id,
                                    onchange: move |e| on_switch_provider(e.value()),

                                    for provider in enabled_providers.iter() {
                                        option {
                                            value: provider.id.clone(),
                                            {provider.name.clone()}
                                        }
                                    }
                                }
                            }

                            // MCP status badges
                            if !enabled_mcp_servers.is_empty() {
                                div {
                                    class: "flex items-center gap-1",
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
                }
            }

            // Right side - New Chat button
            button {
                class: "w-8 h-8 flex items-center justify-center rounded-lg bg-bg-surface hover:bg-bg-secondary text-text-secondary hover:text-text-primary transition-colors",
                onclick: on_new_chat,
                "＋"
            }
        }
    }
}
