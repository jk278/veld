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
                class: "fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50",
                style: "backdrop-filter: blur(4px)",
                onclick: move |_| on_close.call(()),

                div {
                    class: "w-[600px] max-w-[90vw] bg-bg-secondary border border-border rounded-lg p-6 shadow-2xl",
                    style: "background: {theme.bg_secondary}; border-color: {theme.border}; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25); box-sizing: border-box;",
                    onclick: move |e| e.stop_propagation(),

                    select {
                        class: "w-full p-2 bg-bg-surface text-text-primary border border-border rounded mb-4 font-mono focus:border-primary focus:outline-none",
                        style: "background: {theme.bg_surface}; color: {theme.text_primary}; border-color: {theme.border}",
                        value: selected_tool(),
                        oninput: move |e| selected_tool.set(e.value()),

                        for tool in tools {
                            option {
                                value: tool,
                                class: "font-mono",
                                style: "font-family: monospace",
                                {tool.replace("_", " ")}
                            }
                        }
                    }

                    input {
                        class: "w-full p-3 bg-bg-surface text-text-primary border border-border rounded mb-4 font-mono text-sm outline-none transition-all focus:border-primary",
                        style: "background: {theme.bg_surface}; color: {theme.text_primary}; border-color: {theme.border}; box-sizing: border-box",
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

                    div { class: "flex gap-2 mb-4",
                        button {
                            class: "flex-1 p-2.5 bg-bg-surface text-text-secondary border border-border rounded cursor-pointer font-mono text-sm transition-all hover:bg-primary hover:text-white hover:border-primary",
                            style: "background: {theme.bg_surface}; color: {theme.text_secondary}; border-color: {theme.border}; box-sizing: border-box",
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
                            class: "flex-1 p-2.5 bg-bg-surface text-text-secondary border border-border rounded cursor-pointer font-mono text-sm transition-all hover:bg-primary hover:text-white hover:border-primary",
                            style: "background: {theme.bg_surface}; color: {theme.text_secondary}; border-color: {theme.border}; box-sizing: border-box",
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

                    div { class: "flex gap-3 justify-end",
                        button {
                            class: "px-5 py-2.5 text-text-secondary bg-transparent border border-border rounded cursor-pointer font-mono transition-all hover:bg-bg-surface",
                            style: "color: {theme.text_secondary}; border-color: {theme.border}",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "px-5 py-2.5 bg-primary text-white border-none rounded cursor-pointer font-mono font-medium transition-all hover:opacity-90",
                            style: "background: {theme.accent}",
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
