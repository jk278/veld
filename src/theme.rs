//! Theme system using TailwindCSS dark mode
//! 主题系统使用 TailwindCSS 暗色模式

use dioxus::prelude::*;
use dioxus_desktop::tao::{
    event::{Event as WryEvent, WindowEvent},
    window::Theme as SystemTheme,
};
use crate::config::{AppConfig, ThemeMode};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ThemeContext {
    pub theme_mode: Signal<ThemeMode>,
    pub config: Arc<Mutex<AppConfig>>,
}

/// Initialize theme system and provide context
/// 初始化主题系统并提供上下文
pub fn init_theme() -> ThemeContext {
    // Load saved config
    let config = Arc::new(Mutex::new(
        AppConfig::load().unwrap_or_else(|e| {
            eprintln!("[Theme] Failed to load config: {}, using defaults", e);
            AppConfig::default()
        })
    ));

    let initial_mode = config.lock().unwrap().theme.mode;
    let theme_mode = use_signal(move || initial_mode);
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
    dioxus_desktop::use_wry_event_handler(move |event, _| {
        if let WryEvent::WindowEvent {
            event: WindowEvent::ThemeChanged(new_theme),
            ..
        } = event {
            println!("[Theme] System theme changed to: {:?}", new_theme);
            system_theme_signal.set(*new_theme);
            apply_theme_class(theme_mode(), *new_theme);
        }
    });

    // Apply theme on mode change
    use_effect(move || {
        let mode = theme_mode();
        let system_theme = system_theme_signal();
        apply_theme_class(mode, system_theme);
    });

    // Auto-save theme mode when it changes
    use_effect(move || {
        let mode = theme_mode();
        let config = config_clone.clone();
        let system_theme = system_theme_signal();
        apply_theme_class(mode, system_theme);
        std::thread::spawn(move || {
            if let Ok(mut config) = config.lock() {
                config.update_theme(mode);
                println!("[Theme] Theme mode saved: {:?}", mode);
            }
        });
    });

    ThemeContext {
        theme_mode,
        config,
    }
}

/// Apply dark class to document element based on theme mode
/// 根据主题模式应用 dark class 到文档元素
fn apply_theme_class(mode: ThemeMode, system_theme: SystemTheme) {
    let is_dark = match mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::System => matches!(system_theme, SystemTheme::Dark),
    };

    // Use dioxus desktop eval to apply dark class
    let script = if is_dark {
        "document.documentElement.classList.add('dark')"
    } else {
        "document.documentElement.classList.remove('dark')"
    };

    dioxus::document::eval(script);
}

/// Access theme mode from any component
/// 从任何组件访问主题模式
pub fn use_theme() -> Signal<ThemeMode> {
    use_context::<ThemeContext>().theme_mode
}
