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
    let last_result = use_signal(|| Option::<String>::None);

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
            class: "flex flex-col items-center justify-center text-center gap-6 py-8",

            // Welcome message
            h1 {
                class: "text-4xl font-light text-text-primary mb-4",
                style: "color: {theme().text_primary}",
                "Veld AI Toolkit"
            }

            p {
                class: "text-lg text-text-secondary max-w-2xl leading-relaxed mb-8",
                style: "color: {theme().text_secondary}",
                "Press Ctrl+Shift+Space or use the navigation above to access AI tools"
            }

            // Quick action buttons
            div {
                class: "flex flex-wrap gap-3 justify-center mb-8",

                button {
                    class: "px-5 py-2.5 bg-primary text-white border-none rounded-md cursor-pointer font-mono transition-all hover:opacity-90",
                    style: "background: {theme().accent}",
                    onclick: move |_| is_visible.set(true),
                    "Open Input Window"
                }

                Link {
                    to: Route::Settings,
                    class: "px-5 py-2.5 bg-bg-surface text-text-primary border border-border rounded-md text-decoration-none font-mono transition-all hover:bg-primary hover:text-white inline-block",
                    style: "background: {theme().bg_surface}; color: {theme().text_primary}; border-color: {theme().border}",
                    "Settings"
                }
            }

            // Show last result if available
            if let Some(result) = last_result() {
                div {
                    class: "bg-bg-surface border border-border rounded-lg p-5 max-w-4xl mt-8",
                    style: "background: {theme().bg_surface}; border-color: {theme().border}",
                    h3 {
                        class: "text-primary font-mono text-sm mb-3 uppercase tracking-wide",
                        style: "margin: 0 0 12px 0; color: {theme().accent}",
                        "Last Result:"
                    }
                    p {
                        class: "text-text-secondary font-mono whitespace-pre-wrap",
                        style: "margin: 0; color: {theme().text_secondary}",
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
