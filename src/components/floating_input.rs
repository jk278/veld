//! Floating input window component
//! Provides a quick-access input interface for AI tools

use dioxus::prelude::*;

const FLOATING_INPUT_CSS: Asset = asset!("/assets/floating_input.css");

/// Floating input component
#[component]
pub fn FloatingInput(
    is_visible: bool,
    on_close: Callback<()>,
    on_submit: Callback<String>,
) -> Element {
    let mut input_text = use_signal(String::new);
    let mut selected_tool = use_signal(|| "explain".to_string());

    // Available tools
    let tools = vec!["explain", "summarize", "translate", "code_gen", "refactor"];

    rsx! {
        document::Link { rel: "stylesheet", href: FLOATING_INPUT_CSS }
        if is_visible {
            div {
                id: "floating-input-overlay",
                onclick: move |_| {
                    // Close if clicking outside the input box
                    on_close.call(());
                },
                div {
                    id: "floating-input-container",
                    onclick: move |e| e.stop_propagation(),
                    div {
                        id: "tool-selector",
                        select {
                            value: selected_tool(),
                            oninput: move |e| {
                                selected_tool.set(e.value());
                            },
                            for tool in tools {
                                option {
                                    value: tool,
                                    {tool}
                                }
                            }
                        }
                    },
                    div {
                        id: "input-wrapper",
                        input {
                            id: "floating-input",
                            r#type: "text",
                            placeholder: "Type your prompt or press / for help...",
                            value: input_text(),
                            oninput: move |e| {
                                input_text.set(e.value());
                            },
                            onkeydown: move |e| {
                                if e.key() == Key::Escape {
                                    on_close.call(());
                                } else if e.key() == Key::Enter {
                                    if !input_text().trim().is_empty() {
                                        on_submit.call(input_text());
                                        input_text.set(String::new());
                                    }
                                }
                            },
                        }
                        button {
                            id: "submit-btn",
                            onclick: move |_| {
                                if !input_text().trim().is_empty() {
                                    on_submit.call(input_text());
                                    input_text.set(String::new());
                                }
                            },
                            "Send"
                        }
                    },
                    div {
                        id: "help-hint",
                        "Press Ctrl+Shift+Space to activate • Esc to close"
                    }
                }
            }
        }
    }
}
