use dioxus::prelude::*;
use dioxus_desktop::{
    use_global_shortcut,
    use_tray_icon_event_handler,
    use_tray_menu_event_handler,
    trayicon::TrayIconEvent,
};
use crate::components::floating_input::FloatingInput;
use crate::shortcuts::ShortcutManager;
use crate::theme::{init_theme};
use crate::routes::Route;
use std::sync::{Arc, Mutex, OnceLock};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
static SHOW_FLOATING_INPUT: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

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

    // 隐藏顶部菜单栏，保留右键菜单（可以右键 → Inspect Element 打开开发者工具）
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus::desktop::Config::new().with_menu(None))
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut show_floating_input = use_signal(|| false);

    // Initialize theme and provide context
    let theme_context = init_theme();
    provide_context(theme_context.clone());

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

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: TAILWIND_CSS }
        // 旧的样式表已不再需要，TailwindCSS已包含所有样式
        // document::Link { rel: "stylesheet", href: GLOBAL_STYLES }

        // Router with layout attribute automatically wraps all routes
        Router::<Route> {}

        // Global floating input overlay (triggered by hotkey or tray, not default)
        if show_floating_input() {
            FloatingInput {
                is_visible: show_floating_input(),
                on_close: Callback::new(move |_| show_floating_input.set(false)),
                on_submit: Callback::new(|text: String| {
                    println!("Tool selected and submitted: {}", text);
                    // TODO: Implement AI tool handling
                }),
            }
        }
    }
}

pub mod components;
pub mod tray;
pub mod shortcuts;
pub mod window_manager;
pub mod theme;
pub mod config;
pub mod routes;
pub mod services;
pub mod chat_history;

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        assert_eq!(2 + 2, 4);
    }
}
