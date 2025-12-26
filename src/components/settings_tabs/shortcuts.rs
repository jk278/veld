//! Shortcuts tab component
//! 快捷键设置标签页

use dioxus::prelude::*;

/// Shortcuts tab content
#[component]
pub fn ShortcutsTab() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            h1 {
                class: "text-2xl font-semibold text-text-primary",
                "Keyboard Shortcuts"
            }

            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
                h2 {
                    class: "text-lg text-text-primary mb-4",
                    "Global Shortcuts"
                }
                div {
                    class: "space-y-3",
                    ShortcutItem {
                        action: "Show Floating Input",
                        shortcut: "Ctrl+Shift+Space",
                    }
                }
            }
        }
    }
}

/// Shortcut display item
#[component]
fn ShortcutItem(
    action: String,
    shortcut: String,
) -> Element {
    rsx! {
        div {
            class: "flex justify-between items-center py-2 border-b border-border last:border-b-0",
            span {
                class: "text-text-secondary",
                "{action}"
            }
            code {
                class: "px-3 py-1 bg-bg-primary text-primary rounded font-mono text-sm",
                "{shortcut}"
            }
        }
    }
}
