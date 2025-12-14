use dioxus::prelude::*;
use crate::components::floating_input::FloatingInput;
use std::sync::{Arc, Mutex};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

// Global state for system tray and window management
static SHOW_FLOATING_INPUT: once_cell::sync::OnceCell<Arc<Mutex<bool>>> = once_cell::sync::OnceCell::new();

fn main() {
    // Initialize global state for window control
    SHOW_FLOATING_INPUT.set(Arc::new(Mutex::new(false))).unwrap();

    // Initialize system tray before launching the app
    let tray = match crate::tray::SystemTray::new() {
        Ok(tray) => {
            println!("System tray initialized successfully");
            Some(tray)
        }
        Err(e) => {
            println!("Failed to initialize system tray: {:?}", e);
            None
        }
    };

    // Keep the tray alive for the duration of the program
    if let Some(tray) = tray {
        std::mem::forget(tray);
    }

    // Launch the application
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_floating_input = use_signal(|| false);

    // Sync with global state
    use_effect(move || {
        if let Some(global_state) = SHOW_FLOATING_INPUT.get() {
            if let Ok(mut visible) = global_state.lock() {
                *visible = show_floating_input();
            }
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            id: "app",
            h1 { "Veld - AI Toolkit for Developers" }
            p { "Cross-platform system tray application with AI assistant capabilities" }
            div { id: "features",
                h2 { "Features" }
                ul {
                    li { "System tray integration ✓" }
                    li { "Global keyboard shortcuts" }
                    li { "AI-powered tools" }
                    li { "Context-aware operations" }
                }
            }
            div { id: "controls",
                h2 { "Controls" }
                button {
                    onclick: move |_| {
                        show_floating_input.set(true);
                    },
                    "Show Floating Input (Ctrl+Shift+Space)"
                }
                p {
                    class: "hint",
                    "Click the button or press Ctrl+Shift+Space to open the floating input"
                }
            }
            div { id: "status",
                p { "Status: Running - Check your system tray for the Veld icon!" }
            }
        }

        // Floating input component
        FloatingInput {
            is_visible: show_floating_input(),
            on_close: move |_| {
                show_floating_input.set(false);
                if let Some(global_state) = SHOW_FLOATING_INPUT.get() {
                    if let Ok(mut visible) = global_state.lock() {
                        *visible = false;
                    }
                }
            },
            on_submit: move |text: String| {
                println!("Prompt submitted: {}", text);
                show_floating_input.set(false);
                if let Some(global_state) = SHOW_FLOATING_INPUT.get() {
                    if let Ok(mut visible) = global_state.lock() {
                        *visible = false;
                    }
                }
                // TODO: Send to AI
            },
        }
    }
}

// Re-export components
pub mod components;
pub mod tray;
pub mod shortcuts;
pub mod window_manager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        assert_eq!(2 + 2, 4);
    }
}


