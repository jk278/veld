use clipboard::{ClipboardContext, ClipboardProvider};
use dioxus::prelude::*;
use crate::theme::Theme;

#[component]
pub fn FloatingInput(
    is_visible: bool,
    on_close: Callback<()>,
    on_submit: Callback<String>,
    theme: Theme,
) -> Element {
    let mut input_text = use_signal(String::new);
    let mut selected_tool = use_signal(|| "explain".to_string());

    let tools = vec!["explain", "summarize", "translate", "code_gen", "refactor"];

    rsx! {
        if is_visible {
            div {
                style: "position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 100; backdrop-filter: blur(4px);",
                onclick: move |_| on_close.call(()),

                div {
                    style: "background: {theme.bg_secondary}; border: 1px solid {theme.border}; padding: 24px; border-radius: 8px; width: 600px; max-width: 90vw; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25); box-sizing: border-box;",
                    onclick: move |e| e.stop_propagation(),

                    select {
                        style: "width: 100%; padding: 8px; background: {theme.bg_surface}; color: {theme.text_primary}; border: 1px solid {theme.border}; border-radius: 4px; margin-bottom: 16px; font-family: monospace;",
                        value: selected_tool(),
                        oninput: move |e| selected_tool.set(e.value()),

                        for tool in tools {
                            option {
                                value: tool,
                                style: "font-family: monospace;",
                                {tool.replace("_", " ")}
                            }
                        }
                    }

                    input {
                        style: "width: 100%; padding: 12px; background: {theme.bg_surface}; color: {theme.text_primary}; border: 1px solid {theme.border}; border-radius: 4px; margin-bottom: 16px; font-family: monospace; font-size: 14px; outline: none; box-sizing: border-box;",
                        r#type: "text",
                        placeholder: "Type your prompt...",
                        value: input_text(),
                        oninput: move |e| input_text.set(e.value()),
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

                    div { style: "display: flex; gap: 8px; margin-bottom: 16px;",
                        button {
                            style: "flex: 1; padding: 10px; background: {theme.bg_surface}; color: {theme.text_secondary}; border: 1px solid {theme.border}; border-radius: 4px; cursor: pointer; font-family: monospace; font-size: 14px; box-sizing: border-box;",
                            onclick: move |_| {
                                if let Ok(mut ctx) = ClipboardContext::new() {
                                    if let Ok(contents) = ctx.get_contents() {
                                        input_text.set(contents);
                                    }
                                }
                            },
                            title: "Paste from clipboard",
                            "📋 Paste"
                        }
                        button {
                            style: "flex: 1; padding: 10px; background: {theme.bg_surface}; color: {theme.text_secondary}; border: 1px solid {theme.border}; border-radius: 4px; cursor: pointer; font-family: monospace; font-size: 14px; box-sizing: border-box;",
                            onclick: move |_| {
                                if !input_text().trim().is_empty() {
                                    if let Ok(mut ctx) = ClipboardContext::new() {
                                        let _ = ctx.set_contents(input_text());
                                    }
                                }
                            },
                            title: "Copy to clipboard",
                            "📄 Copy"
                        }
                    }

                    div { style: "display: flex; gap: 12px; justify-content: flex-end;",
                        button {
                            style: "padding: 10px 20px; color: {theme.text_secondary}; background: transparent; border: 1px solid {theme.border}; border-radius: 4px; cursor: pointer; font-family: monospace;",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            style: "padding: 10px 20px; background: {theme.accent}; color: white; border: none; border-radius: 4px; cursor: pointer; font-family: monospace; font-weight: 500;",
                            onclick: move |_| {
                                if !input_text().trim().is_empty() {
                                    on_submit.call(input_text());
                                    input_text.set(String::new());
                                }
                            },
                            "Send"
                        }
                    }
                }
            }
        }
    }
}
