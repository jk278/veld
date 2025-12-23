use arboard::Clipboard;
use dioxus::prelude::*;

#[component]
pub fn FloatingInput(
    is_visible: bool,
    on_close: Callback<()>,
    on_submit: Callback<String>,
) -> Element {
    let mut input_text = use_signal(String::new);
    let mut selected_tool = use_signal(|| "explain".to_string());

    let tools = vec!["explain", "summarize", "translate", "code_gen", "refactor"];

    // Use CSS classes for visibility control (no conditional rendering)
    // This prevents component remounting and improves activation speed
    let overlay_class = if is_visible {
        "floating-overlay visible"
    } else {
        "floating-overlay"
    };

    let content_class = if is_visible {
        "floating-content visible"
    } else {
        "floating-content"
    };

    rsx! {
        div {
            class: "{overlay_class}",
            onclick: move |_| on_close.call(()),

            div {
                class: "{content_class}",
                onclick: move |e| e.stop_propagation(),

                    select {
                        class: "input-field mb-4",
                        value: selected_tool(),
                        oninput: move |e| selected_tool.set(e.value()),

                        for tool in tools {
                            option {
                                value: tool,
                                class: "font-mono",
                                {tool.replace("_", " ")}
                            }
                        }
                    }

                    input {
                        class: "input-field mb-4",
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
                            class: "btn-secondary flex-1 p-2.5 text-sm",
                            onclick: move |_| {
                                if let Ok(mut clipboard) = Clipboard::new() {
                                    if let Ok(text) = clipboard.get_text() {
                                        input_text.set(text);
                                    }
                                }
                            },
                            title: "Paste from clipboard",
                            "📋 Paste"
                        }
                        button {
                            class: "btn-secondary flex-1 p-2.5 text-sm",
                            onclick: move |_| {
                                if !input_text().trim().is_empty() {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        let _ = clipboard.set_text(input_text());
                                    }
                                }
                            },
                            title: "Copy to clipboard",
                            "📄 Copy"
                        }
                    }

                    div { class: "flex gap-3 justify-end",
                        button {
                            class: "btn-cancel",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "btn-primary",
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
