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
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                min-height: calc(100vh - 80px);
                text-align: center;
            ",

            h1 {
                style: "
                    font-size: 2rem;
                    font-weight: 300;
                    margin-bottom: 1rem;
                    color: {theme().text_primary};
                ",
                "Result Viewer"
            }

            p {
                style: "
                    font-size: 1rem;
                    color: {theme().text_secondary};
                    font-family: monospace;
                ",
                "Session ID: {session_id}"
            }

            div {
                style: "
                    margin-top: 2rem;
                    padding: 20px;
                    background: {theme().bg_surface};
                    border: 1px solid {theme().border};
                    border-radius: 8px;
                    max-width: 800px;
                ",
                p {
                    style: "
                        margin: 0;
                        color: {theme().text_secondary};
                        font-family: monospace;
                    ",
                    "Results will appear here..."
                }
            }
        }
    }
}
