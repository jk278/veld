//! Home page - Floating input interface
//! 首页组件，集成浮动输入窗口功能

use dioxus::prelude::*;
use crate::components::floating_input::FloatingInput;
use crate::theme::{use_theme};
use crate::routes::Route;

/// Home page component
/// 显示浮动输入界面的首页
#[component]
pub fn Home() -> Element {
    let (_, theme) = use_theme();
    let mut is_visible = use_signal(|| true);
    let mut last_result = use_signal(|| Option::<String>::None);

    // Handle form submission - use cloned signals to avoid borrow issues
    let mut is_visible_clone = is_visible.clone();
    let mut last_result_clone = last_result.clone();
    let on_submit = move |text: String| {
        // TODO: Process with AI API
        // For now, just echo the input
        last_result_clone.set(Some(format!("Processed: {}", text)));
        is_visible_clone.set(false);
    };

    // Handle close
    let on_close = move |_| {
        is_visible.set(false);
    };

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

            // Welcome message
            h1 {
                style: "
                    font-size: 2.5rem;
                    font-weight: 300;
                    margin-bottom: 1rem;
                    color: {theme().text_primary};
                ",
                "Veld AI Toolkit"
            }

            p {
                style: "
                    font-size: 1.1rem;
                    color: {theme().text_secondary};
                    margin-bottom: 2rem;
                    max-width: 600px;
                ",
                "Press Ctrl+Shift+Space or use the navigation above to access AI tools"
            }

            // Quick action buttons
            div {
                style: "
                    display: flex;
                    gap: 12px;
                    margin-bottom: 2rem;
                    flex-wrap: wrap;
                    justify-content: center;
                ",

                button {
                    style: "
                        padding: 10px 20px;
                        background: {theme().accent};
                        color: white;
                        border: none;
                        border-radius: 6px;
                        cursor: pointer;
                        font-family: monospace;
                        transition: all 0.2s;
                        &:hover {{
                            opacity: 0.9;
                        }}
                    ",
                    onclick: move |_| is_visible.set(true),
                    "Open Input Window"
                }

                Link {
                    to: Route::Settings,
                    style: "
                        padding: 10px 20px;
                        background: {theme().bg_surface};
                        color: {theme().text_primary};
                        border: 1px solid {theme().border};
                        border-radius: 6px;
                        text-decoration: none;
                        font-family: monospace;
                        transition: all 0.2s;
                        display: inline-block;
                        &:hover {{
                            background: {theme().accent};
                            color: white;
                        }}
                    ",
                    "Settings"
                }
            }

            // Show last result if available
            if let Some(result) = last_result() {
                div {
                    style: "
                        background: {theme().bg_surface};
                        border: 1px solid {theme().border};
                        border-radius: 8px;
                        padding: 20px;
                        max-width: 800px;
                        margin-top: 2rem;
                    ",
                    h3 {
                        style: "margin: 0 0 12px 0; color: {theme().accent};",
                        "Last Result:"
                    }
                    p {
                        style: "
                            margin: 0;
                            color: {theme().text_secondary};
                            font-family: monospace;
                            white-space: pre-wrap;
                        ",
                        {result}
                    }
                }
            }
        }

        // Floating input window
        FloatingInput {
            is_visible: is_visible(),
            on_close: on_close,
            on_submit: on_submit,
            theme: theme(),
        }
    }
}
