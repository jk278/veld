//! Chat sidebar component
//! 聊天侧边栏组件 - 会话列表

use dioxus::prelude::*;
use super::UiSession;

/// Chat sidebar with session list (drawer overlay pattern)
#[component]
pub fn ChatSidebar(
    sessions: Vec<UiSession>,
    sidebar_collapsed: bool,
    on_new_chat: EventHandler<MouseEvent>,
    on_switch_session: EventHandler<String>,
    on_delete_session: EventHandler<String>,
    on_close: EventHandler<MouseEvent>,
    #[props(default)] on_auto_collapse: EventHandler<()>,
) -> Element {
    rsx! {
        // Overlay backdrop (click to close, narrow screen only)
        div {
            class: if sidebar_collapsed { "drawer-overlay" } else { "drawer-overlay visible" },
            onclick: on_close,
        }

        // Drawer sidebar (responsive: overlay on narrow, inline on wide)
        div {
            class: {
                let base = "drawer-sidebar";
                if sidebar_collapsed {
                    // Narrow: hidden, Wide: collapsed
                    format!("{base} collapsed")
                } else {
                    // Narrow: visible, Wide: shown
                    format!("{base} visible")
                }
            },

            // Sidebar header
            div {
                class: "p-4",
                button {
                    class: "w-full btn-primary flex items-center justify-center gap-2 whitespace-nowrap",
                    onclick: on_new_chat,
                    span { class: "text-lg", "➕" }
                    "New Chat"
                }
            }

            // Session list
            div {
                class: "flex-1 overflow-y-auto p-2 space-y-1",
                if sessions.is_empty() {
                    p {
                        class: "text-sm text-text-muted text-center py-4",
                        "No chat history yet"
                    }
                } else {
                    for session in sessions.into_iter() {
                        SessionItem {
                            session: session.clone(),
                            on_switch: on_switch_session,
                            on_delete: on_delete_session,
                            on_auto_collapse: on_auto_collapse,
                        }
                    }
                }
            }
        }
    }
}

/// Individual session item
#[component]
pub fn SessionItem(
    session: UiSession,
    #[props(default)] on_switch: EventHandler<String>,
    #[props(default)] on_delete: EventHandler<String>,
    #[props(default)] on_auto_collapse: EventHandler<()>,
) -> Element {
    rsx! {
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
                    on_switch.call(sid.clone());
                    on_auto_collapse.call(());
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
                            on_delete.call(sid.clone());
                        }
                    },
                    "×"
                }
            }
        }
    }
}
