//! Message list component
//! 消息列表组件 - 显示聊天消息

use dioxus::prelude::*;
use crate::components::markdown::{MarkdownContent, PlainTextContent};

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
            class: "max-w-2xl",
            div {
                class: "flex items-start gap-2 justify-end",
                div {
                    class: "px-4 py-2.5 bg-primary text-white rounded-2xl rounded-tr-md",
                    style: "max-width: 80%;",
                    PlainTextContent {
                        content: content.clone(),
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
                {format_timestamp(timestamp)}
            }
        }
    }
}

/// Assistant message bubble
#[component]
fn AssistantMessageBubble(content: String, timestamp: u64) -> Element {
    rsx! {
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
                        content: content.clone(),
                        class: "text-sm text-text-primary".to_string()
                    }
                }
            }
            p {
                class: "text-xs text-text-muted mt-1",
                {format_timestamp(timestamp)}
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
