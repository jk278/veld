//! Chat input area component
//! 聊天输入区域组件

use dioxus::prelude::*;

/// Input area with text field and send button
///
/// # IMPORTANT NOTE
/// textarea must be direct child of flex (no wrapper div) to avoid 6px ghost height issue
#[component]
pub fn InputArea(
    input_text: Signal<String>,
    has_api_key: bool,
    on_send: EventHandler<MouseEvent>,
    tx: Coroutine<String>,
) -> Element {
    rsx! {
        div {
            class: "px-4 py-3 border-t border-border relative z-10 shadow-custom",
            div {
                class: "flex gap-2",

                textarea {
                    class: "flex-1 px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg resize-none focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all font-mono text-sm",
                    rows: 1,
                    placeholder: if !has_api_key {
                        "Configure API key first..."
                    } else {
                        "Type your message..."
                    },
                    value: input_text(),
                    disabled: !has_api_key,
                    oninput: move |e| input_text.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter && has_api_key {
                            e.prevent_default();
                            let text = input_text().trim().to_string();
                            if !text.is_empty() {
                                input_text.set(String::new());
                                tx.send(text);
                            }
                        }
                    },
                }

                button {
                    class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm font-medium",
                    disabled: !has_api_key || input_text().trim().is_empty(),
                    onclick: on_send,
                    span { "📤" }
                    "Send"
                }
            }
        }
    }
}

/// Chat input wrapper with proper signal handling
#[component]
pub fn ChatInput(
    input_text: Signal<String>,
    has_api_key: bool,
    on_send: EventHandler<MouseEvent>,
    tx: Coroutine<String>,
) -> Element {
    rsx! {
        InputArea {
            input_text: input_text.clone(),
            has_api_key,
            on_send,
            tx,
        }
    }
}
