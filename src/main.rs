use dioxus::prelude::*;
use dioxus_desktop::{
    use_global_shortcut,
    use_tray_icon_event_handler,
    use_tray_menu_event_handler,
    trayicon::TrayIconEvent,
};
use crate::shortcuts::ShortcutManager;
use crate::theme::{use_theme, ThemeMode, ThemeSelector};
use std::sync::{Arc, Mutex};

const FAVICON: Asset = asset!("/assets/favicon.ico");
static SHOW_FLOATING_INPUT: once_cell::sync::OnceCell<Arc<Mutex<bool>>> = once_cell::sync::OnceCell::new();

fn main() {
    SHOW_FLOATING_INPUT.set(Arc::new(Mutex::new(false))).unwrap();

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

    if let Some(tray) = tray {
        std::mem::forget(tray);
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut show_floating_input = use_signal(|| false);
    let (mut theme_mode, theme) = use_theme();

    let _shortcut_handle = use_global_shortcut(
        "Ctrl+Shift+Space",
        move |state| {
            if state == dioxus_desktop::HotKeyState::Pressed {
                println!("[App] Global hotkey triggered!");
                show_floating_input.set(true);
            }
        },
    );

    use_tray_icon_event_handler(move |event| {
        match event {
            TrayIconEvent::Click { button, .. } => {
                if *button == dioxus_desktop::trayicon::MouseButton::Left {
                    show_floating_input.set(true);
                }
            }
            _ => {}
        }
    });

    use_tray_menu_event_handler(move |event: &dioxus_desktop::trayicon::menu::MenuEvent| {
        match event.id.as_ref() {
            "show" => show_floating_input.set(true),
            "quit" => std::process::exit(0),
            _ => {}
        }
    });

    use_effect(|| {
        match ShortcutManager::new() {
            Ok(_) => {
                println!("Global shortcuts initialized");
                println!("Press Ctrl+Shift+Space to show floating input");
            }
            Err(e) => eprintln!("Failed to initialize shortcuts: {:?}", e),
        }
    });

    use_effect(move || {
        if let Some(global_state) = SHOW_FLOATING_INPUT.get() {
            if let Ok(mut visible) = global_state.lock() {
                *visible = show_floating_input();
            }
        }
    });

    let current_theme = theme();

    rsx! {
        document::Link { rel: "icon", href: FAVICON }

        div {
            style: "font-family: Inter, system-ui, sans-serif; padding: 40px; min-height: 100vh; background: {current_theme.bg_primary}; color: {current_theme.text_secondary};",
            ondoubleclick: move |_| {
                let new_mode = match theme_mode() {
                    ThemeMode::Light => ThemeMode::Dark,
                    ThemeMode::Dark => ThemeMode::System,
                    ThemeMode::System => ThemeMode::Light,
                };
                theme_mode.set(new_mode);
            },

            h1 { style: "font-size: 32px; font-weight: 600; margin-bottom: 16px; color: {current_theme.text_primary};",
                "Veld - AI Toolkit"
            }

            ThemeSelector { theme_mode, current_theme }

            p { style: "margin-bottom: 24px; color: {current_theme.text_secondary};",
                "System tray application ready"
            }

            div { style: "background: {current_theme.bg_secondary}; border: 1px solid {current_theme.border}; padding: 24px; border-radius: 8px; margin-bottom: 24px;",
                h2 { style: "font-size: 20px; font-weight: 600; margin-bottom: 16px; color: {current_theme.text_primary};",
                    "Features"
                }

                ul { style: "list-style: none;",
                    li { style: "font-family: monospace; font-size: 14px; color: {current_theme.text_muted}; margin-bottom: 8px; padding: 4px 0;",
                        "✓ System tray integration"
                    }
                    li { style: "font-family: monospace; font-size: 14px; color: {current_theme.text_muted}; margin-bottom: 8px; padding: 4px 0;",
                        "✓ Global shortcuts (Ctrl+Shift+Space)"
                    }
                    li { style: "font-family: monospace; font-size: 14px; color: {current_theme.text_muted}; margin-bottom: 8px; padding: 4px 0;",
                        "✓ Floating input window"
                    }
                    li { style: "font-family: monospace; font-size: 14px; color: {current_theme.text_muted}; margin-bottom: 8px; padding: 4px 0;",
                        "✓ AI-powered tools"
                    }
                }
            }

            button {
                style: "background: {current_theme.accent}; color: white; padding: 12px 24px; border: none; border-radius: 4px; cursor: pointer; font-weight: 500; font-family: monospace;",
                onclick: move |_| show_floating_input.set(true),
                "🚀 Open Floating Input"
            }

            div { style: "background: {current_theme.bg_secondary}; border-left: 2px solid {current_theme.accent}; padding: 16px; margin-top: 32px; border-radius: 4px;",
                p { style: "font-family: monospace; font-size: 14px; color: {current_theme.text_muted};",
                    "Status: Running - Check system tray!"
                }
            }
        }

        if show_floating_input() {
            div {
                style: "position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 100; backdrop-filter: blur(4px);",
                onclick: move |_| show_floating_input.set(false),

                div {
                    style: "background: {current_theme.bg_secondary}; border: 1px solid {current_theme.border}; padding: 24px; border-radius: 8px; width: 600px; max-width: 90vw; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.25);",
                    onclick: move |e| e.stop_propagation(),

                    select {
                        style: "width: 100%; padding: 8px; background: {current_theme.bg_surface}; color: {current_theme.text_primary}; border: 1px solid {current_theme.border}; border-radius: 4px; margin-bottom: 16px; font-family: monospace;",
                        option { value: "explain", "Explain code" }
                        option { value: "summarize", "Summarize text" }
                        option { value: "translate", "Translate" }
                        option { value: "code_gen", "Generate code" }
                        option { value: "refactor", "Refactor code" }
                    }

                    input {
                        style: "width: 100%; padding: 12px; background: {current_theme.bg_surface}; color: {current_theme.text_primary}; border: 1px solid {current_theme.border}; border-radius: 4px; margin-bottom: 16px; font-family: monospace; font-size: 14px; outline: none;",
                        placeholder: "Type your prompt..."
                    }

                    div { style: "display: flex; gap: 12px; justify-content: flex-end;",
                        button {
                            style: "padding: 10px 20px; color: {current_theme.text_secondary}; background: transparent; border: 1px solid {current_theme.border}; border-radius: 4px; cursor: pointer; font-family: monospace;",
                            onclick: move |_| show_floating_input.set(false),
                            "Cancel"
                        }
                        button {
                            style: "padding: 10px 20px; background: {current_theme.accent}; color: white; border: none; border-radius: 4px; cursor: pointer; font-family: monospace; font-weight: 500;",
                            "Send"
                        }
                    }
                }
            }
        }
    }
}

pub mod components;
pub mod tray;
pub mod shortcuts;
pub mod window_manager;
pub mod theme;

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        assert_eq!(2 + 2, 4);
    }
}
