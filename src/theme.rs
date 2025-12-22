use dioxus::prelude::*;
use dioxus_desktop::tao::{
    event::{Event as WryEvent, WindowEvent},
    window::Theme as SystemTheme,
};
use crate::config::{AppConfig, ThemeMode};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
pub struct Theme {
    pub bg_primary: &'static str,
    pub bg_secondary: &'static str,
    pub bg_surface: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    pub accent: &'static str,
    pub border: &'static str,
}

pub const LIGHT_THEME: Theme = Theme {
    bg_primary: "#ffffff",
    bg_secondary: "#f8f9fa",
    bg_surface: "#e9ecef",
    text_primary: "#212529",
    text_secondary: "#495057",
    text_muted: "#6c757d",
    accent: "#1194a3",
    border: "#dee2e6",
};

pub const DARK_THEME: Theme = Theme {
    bg_primary: "#050505",
    bg_secondary: "#111827",
    bg_surface: "#1a1a1a",
    text_primary: "#e8eaed",
    text_secondary: "#9aa0a6",
    text_muted: "#5f6368",
    accent: "#1194a3",
    border: "#333",
};

impl Default for Theme {
    fn default() -> Self {
        DARK_THEME
    }
}

#[derive(Clone)]
pub struct ThemeContext {
    pub theme_mode: Signal<ThemeMode>,
    pub theme: Signal<Theme>,
    pub config: Arc<Mutex<AppConfig>>,
}

/// Initialize theme system and provide context - call this in root component
pub fn init_theme() -> ThemeContext {
    // Load saved config
    let config = match AppConfig::load() {
        Ok(config) => {
            println!("[Theme] Configuration loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("[Theme] Failed to load config: {}, using defaults", e);
            AppConfig::default()
        }
    };

    let config = Arc::new(Mutex::new(config));
    let initial_mode = config.lock().unwrap().theme.mode;

    let theme_mode = use_signal(move || initial_mode);
    let mut theme = use_signal(|| DARK_THEME);
    let mut system_theme_signal = use_signal(|| SystemTheme::Dark);
    let config_clone = config.clone();

    // Initialize system theme from window on first run
    use_effect(move || {
        let window = dioxus_desktop::window();
        let initial_theme = window.theme();
        println!("[Theme] Initial system theme detected: {:?}", initial_theme);
        system_theme_signal.set(initial_theme);
    });

    // Listen for system theme changes
    use_effect(move || {
        dioxus_desktop::use_wry_event_handler(move |event, _| {
            match event {
                WryEvent::WindowEvent {
                    event: WindowEvent::ThemeChanged(new_theme),
                    ..
                } => {
                    println!("[Theme] System theme changed to: {:?}", new_theme);
                    system_theme_signal.set(*new_theme);
                }
                _ => {}
            }
        });
    });

    use_effect(move || {
        let mode = theme_mode();

        let current_theme = match mode {
            ThemeMode::Light => LIGHT_THEME,
            ThemeMode::Dark => DARK_THEME,
            ThemeMode::System => {
                match system_theme_signal() {
                    SystemTheme::Dark => DARK_THEME,
                    SystemTheme::Light => LIGHT_THEME,
                    _ => DARK_THEME,
                }
            }
        };
        theme.set(current_theme);
    });

    // Auto-save theme mode when it changes (via config module)
    use_effect(move || {
        let mode = theme_mode();
        let config = config_clone.clone();
        // Use config module's update method which handles saving in background
        std::thread::spawn(move || {
            if let Ok(mut config) = config.lock() {
                config.update_theme(mode);
                println!("[Theme] Theme mode saved: {:?}", mode);
            }
        });
    });

    ThemeContext {
        theme_mode,
        theme,
        config,
    }
}

/// Access theme context from any component
pub fn use_theme() -> (Signal<ThemeMode>, Signal<Theme>) {
    let ctx = use_context::<ThemeContext>();
    (ctx.theme_mode, ctx.theme)
}
