//! Result viewer page
//! 显示AI处理结果的页面

use dioxus::prelude::*;
use crate::theme::{use_theme};

/// Result viewer component
/// 显示特定会话的结果
#[component]
pub fn ResultViewer(session_id: String) -> Element {
    let (_, theme) = use_theme();

    rsx! {
        div {
            class: "flex flex-col items-center justify-center min-h-[calc(100vh-80px)] text-center space-y-6",

            h1 {
                class: "text-2xl font-light text-text-primary mb-4",
                style: "color: {theme().text_primary}",
                "Result Viewer"
            }

            p {
                class: "text-text-secondary font-mono",
                style: "color: {theme().text_secondary}",
                "Session ID: {session_id}"
            }

            div {
                class: "bg-bg-surface border border-border rounded-lg p-5 max-w-4xl",
                style: "background: {theme().bg_surface}; border-color: {theme().border}",
                p {
                    class: "text-text-secondary font-mono",
                    style: "margin: 0; color: {theme().text_secondary}",
                    "📊 Results will appear here..."
                }
            }
        }
    }
}
