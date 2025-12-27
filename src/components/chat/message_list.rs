//! Message list component
//! 消息列表组件 - 显示聊天消息

use dioxus::prelude::*;
use crate::components::markdown::{MarkdownContent, PlainTextContent};
use crate::theme::use_is_dark;

/// Chat message for display
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

/// Message list container
#[component]
pub fn MessageList(
    messages: Vec<ChatMessage>,
    has_api_key: bool,
    #[props(default)] scroll_container_id: String,
) -> Element {
    rsx! {
        div {
            id: scroll_container_id,
            class: "flex-1 overflow-y-auto px-4 py-4 space-y-4",

            if messages.is_empty() {
                EmptyState { has_api_key }
            } else {
                for msg in messages.into_iter() {
                    MessageBubble { message: msg }
                }
            }
        }
    }
}

/// Empty state when no messages
#[component]
pub fn EmptyState(has_api_key: bool) -> Element {
    rsx! {
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
                        href: "/settings",
                        "Settings"
                    }
                }
            }
        }
    }
}

/// Individual message bubble
#[component]
fn MessageBubble(message: ChatMessage) -> Element {
    // Check if this is an intermediate step (contains bullet point marker)
    let is_intermediate = message.content.contains("• ");

    rsx! {
        div {
            class: if message.role == "user" {
                "flex justify-end"
            } else if message.role == "system" {
                "flex justify-center"
            } else {
                "flex justify-start"
            },

            if message.role == "system" {
                div {
                    class: "px-4 py-2 bg-error/10 border border-error/30 rounded-lg text-sm text-error max-w-md",
                    {message.content.clone()}
                }
            } else if message.role == "user" {
                UserMessageBubble {
                    content: message.content.clone(),
                    timestamp: message.timestamp,
                }
            } else {
                AssistantMessageBubble {
                    content: message.content.clone(),
                    timestamp: message.timestamp,
                    is_intermediate,
                }
            }
        }
    }
}

/// User message bubble
#[component]
fn UserMessageBubble(content: String, timestamp: u64) -> Element {
    rsx! {
        div {
            class: "max-w-2xl max-w-[80%] flex justify-end",
            div {
                class: "px-4 py-2.5 bg-primary text-white rounded-2xl rounded-tr-md",
                PlainTextContent {
                    content: content.clone(),
                    class: "text-sm leading-relaxed".to_string()
                }
            }
        }
    }
}

/// Parse a thinking step line, returning (short, optional_detail)
fn parse_thinking_step(line: &str) -> Option<(String, Option<String>)> {
    line.strip_prefix("• ").map(|step_text| {
        if let Some(idx) = step_text.find('|') {
            let short = step_text[..idx].to_string();
            let detail = step_text[idx + 1..].to_string();
            (short, Some(detail))
        } else {
            (step_text.to_string(), None)
        }
    })
}

/// Assistant message bubble
#[component]
fn AssistantMessageBubble(content: String, timestamp: u64, is_intermediate: bool) -> Element {
    let is_dark = use_is_dark();

    // Count steps for the summary
    let step_count = content.matches("• ").count();

    rsx! {
        div {
            class: "max-w-2xl w-full",
            if is_intermediate {
                // Intermediate steps: collapsible panel
                div {
                    class: "thinking-process",
                    details {
                        summary {
                            class: "thinking-summary cursor-pointer",
                            "思考过程"
                            if step_count > 0 {
                                span { class: "ml-2 text-text-muted", "({step_count}步)" }
                            }
                        }
                        div {
                            class: "thinking-content",
                            // Parse and render each step (split by bullet point, not newline)
                            // Filter out empty items before rendering
                            for part in content.split("• ").skip(1).filter_map(|part| {
                                parse_thinking_step(&format!("• {}", part))
                                    .filter(|(short, _)| !short.trim().is_empty())
                            }) {
                                div {
                                    class: "thinking-item",
                                    div {
                                        class: "thinking-item-short",
                                        MarkdownContent {
                                            content: part.0.clone(),
                                            class: String::new(),
                                            dark: is_dark()
                                        }
                                    }
                                    if let Some(detail_text) = &part.1 {
                                        div {
                                            class: "thinking-item-detail",
                                            MarkdownContent {
                                                content: detail_text.clone(),
                                                class: String::new(),
                                                dark: is_dark()
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Final answer: normal markdown rendering
                div {
                    class: "markdown-body w-full text-sm text-text-primary",
                    MarkdownContent {
                        content: content.clone(),
                        class: String::new(),
                        dark: is_dark()
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
