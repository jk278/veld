use dioxus::prelude::*;
use dioxus_desktop::tao::{
    event::{Event as WryEvent, WindowEvent},
    window::Theme as SystemTheme,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

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

pub fn use_theme() -> (Signal<ThemeMode>, Signal<Theme>) {
    let theme_mode = use_signal(|| ThemeMode::Dark);
    let mut theme = use_signal(|| DARK_THEME);
    let mut system_theme_signal = use_signal(|| SystemTheme::Dark);

    // Listen for system theme changes
    use_effect(move || {
        dioxus_desktop::use_wry_event_handler(move |event, _| {
            if let WryEvent::WindowEvent {
                event: WindowEvent::ThemeChanged(new_theme),
                ..
            } = event
            {
                system_theme_signal.set(*new_theme);
            }
        });
    });

    use_effect(move || {
        let mode = theme_mode();
        let _window = dioxus_desktop::window();

        let current_theme = match mode {
            ThemeMode::Light => LIGHT_THEME,
            ThemeMode::Dark => DARK_THEME,
            ThemeMode::System => {
                // Use the system theme (either from signal if changed, or current)
                match system_theme_signal() {
                    SystemTheme::Dark => DARK_THEME,
                    SystemTheme::Light => LIGHT_THEME,
                    _ => DARK_THEME,
                }
            }
        };
        theme.set(current_theme);
    });

    (theme_mode, theme)
}

pub fn get_theme_selector_style(theme_mode: ThemeMode, current_theme: Theme, target_mode: ThemeMode) -> String {
    let is_active = theme_mode == target_mode;
    format!(
        "padding: 6px 12px; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-family: monospace; cursor: pointer;",
        current_theme.border,
        if is_active { current_theme.accent } else { current_theme.bg_surface },
        if is_active { "white" } else { current_theme.text_primary }
    )
}

#[component]
pub fn ThemeSelector(theme_mode: Signal<ThemeMode>, current_theme: Theme) -> Element {
    rsx! {
        div { style: "display: flex; gap: 8px; align-items: center;",
            label { style: "color: {current_theme.text_secondary}; font-size: 14px; margin-right: 8px;",
                "Theme:"
            }
            button {
                style: "{get_theme_selector_style(theme_mode(), current_theme, ThemeMode::Light)}",
                onclick: move |_| theme_mode.set(ThemeMode::Light),
                "☀️ Light"
            }
            button {
                style: "{get_theme_selector_style(theme_mode(), current_theme, ThemeMode::Dark)}",
                onclick: move |_| theme_mode.set(ThemeMode::Dark),
                "🌙 Dark"
            }
            button {
                style: "{get_theme_selector_style(theme_mode(), current_theme, ThemeMode::System)}",
                onclick: move |_| theme_mode.set(ThemeMode::System),
                "🖥️ System"
            }
        }
    }
}
