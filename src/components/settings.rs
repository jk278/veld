//! Settings page component
//! 设置页面组件

use dioxus::prelude::*;
use crate::theme::{use_theme, ThemeMode};

/// Settings page
/// 应用设置页面
#[component]
pub fn Settings() -> Element {
    let (mut theme_mode, theme) = use_theme();
    let current_theme = theme();

    let mut api_key = use_signal(|| "".to_string());
    let mut selected_model = use_signal(|| "gpt-4".to_string());
    let mut auto_start = use_signal(|| false);

    rsx! {
        div {
            style: "
                max-width: 800px;
                margin: 0 auto;
            ",

            h1 {
                style: "
                    font-size: 2rem;
                    font-weight: 300;
                    margin-bottom: 2rem;
                    color: {current_theme.text_primary};
                ",
                "Settings"
            }

            // Theme selector
            section {
                style: "
                    background: {current_theme.bg_surface};
                    border: 1px solid {current_theme.border};
                    border-radius: 8px;
                    padding: 24px;
                    margin-bottom: 24px;
                ",

                h2 {
                    style: "
                        font-size: 1.3rem;
                        margin: 0 0 16px 0;
                        color: {current_theme.text_primary};
                    ",
                    "Appearance"
                }

                div { style: "display: flex; gap: 8px; align-items: center;",
                    label { style: "color: {current_theme.text_secondary}; font-size: 14px; margin-right: 8px;",
                        "Theme:"
                    }
                    button {
                        style: format!(
                            "padding: 6px 12px; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-family: monospace; cursor: pointer;",
                            current_theme.border,
                            if theme_mode() == ThemeMode::Light { current_theme.accent } else { current_theme.bg_surface },
                            if theme_mode() == ThemeMode::Light { "white" } else { current_theme.text_primary }
                        ),
                        onclick: move |_| theme_mode.set(ThemeMode::Light),
                        "☀️ Light"
                    }
                    button {
                        style: format!(
                            "padding: 6px 12px; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-family: monospace; cursor: pointer;",
                            current_theme.border,
                            if theme_mode() == ThemeMode::Dark { current_theme.accent } else { current_theme.bg_surface },
                            if theme_mode() == ThemeMode::Dark { "white" } else { current_theme.text_primary }
                        ),
                        onclick: move |_| theme_mode.set(ThemeMode::Dark),
                        "🌙 Dark"
                    }
                    button {
                        style: format!(
                            "padding: 6px 12px; border-radius: 4px; border: 1px solid {}; background: {}; color: {}; font-family: monospace; cursor: pointer;",
                            current_theme.border,
                            if theme_mode() == ThemeMode::System { current_theme.accent } else { current_theme.bg_surface },
                            if theme_mode() == ThemeMode::System { "white" } else { current_theme.text_primary }
                        ),
                        onclick: move |_| theme_mode.set(ThemeMode::System),
                        "🖥️ System"
                    }
                }
            }

            // AI Configuration section
            section {
                style: "
                    background: {current_theme.bg_surface};
                    border: 1px solid {current_theme.border};
                    border-radius: 8px;
                    padding: 24px;
                    margin-bottom: 24px;
                ",

                h2 {
                    style: "
                        font-size: 1.3rem;
                        margin: 0 0 16px 0;
                        color: {current_theme.text_primary};
                    ",
                    "AI Configuration"
                }

                div {
                    style: "margin-bottom: 20px;",
                    label {
                        style: "
                            display: block;
                            margin-bottom: 8px;
                            color: {current_theme.text_secondary};
                            font-size: 0.9rem;
                        ",
                        "API Provider"
                    }
                    select {
                        style: "
                            width: 100%;
                            padding: 10px;
                            background: {current_theme.bg_surface};
                            color: {current_theme.text_primary};
                            border: 1px solid {current_theme.border};
                            border-radius: 6px;
                            font-family: monospace;
                        ",
                        value: selected_model(),
                        oninput: move |e| selected_model.set(e.value()),

                        option { value: "gpt-4", "OpenAI GPT-4" }
                        option { value: "gpt-3.5-turbo", "OpenAI GPT-3.5 Turbo" }
                        option { value: "claude-3", "Anthropic Claude 3" }
                    }
                }

                div {
                    style: "margin-bottom: 20px;",
                    label {
                        style: "
                            display: block;
                            margin-bottom: 8px;
                            color: {current_theme.text_secondary};
                            font-size: 0.9rem;
                        ",
                        "API Key"
                    }
                    input {
                        style: "
                            width: 100%;
                            padding: 10px;
                            background: {current_theme.bg_surface};
                            color: {current_theme.text_primary};
                            border: 1px solid {current_theme.border};
                            border-radius: 6px;
                            font-family: monospace;
                            box-sizing: border-box;
                        ",
                        r#type: "password",
                        placeholder: "Enter your API key...",
                        value: api_key(),
                        oninput: move |e| api_key.set(e.value()),
                    }
                }
            }

            // Keyboard Shortcuts section
            section {
                style: "
                    background: {current_theme.bg_surface};
                    border: 1px solid {current_theme.border};
                    border-radius: 8px;
                    padding: 24px;
                    margin-bottom: 24px;
                ",

                h2 {
                    style: "
                        font-size: 1.3rem;
                        margin: 0 0 16px 0;
                        color: {current_theme.text_primary};
                    ",
                    "Keyboard Shortcuts"
                }

                div {
                    style: "display: flex; flex-direction: column; gap: 12px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        span {
                            style: "color: {current_theme.text_secondary};",
                            "Show Floating Input"
                        }
                        code {
                            style: "
                                background: {current_theme.bg_primary};
                                padding: 6px 12px;
                                border-radius: 4px;
                                color: {current_theme.accent};
                                font-family: monospace;
                            ",
                            "Ctrl+Shift+Space"
                        }
                    }
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        span {
                            style: "color: {current_theme.text_secondary};",
                            "Quick Summarize"
                        }
                        code {
                            style: "
                                background: {current_theme.bg_primary};
                                padding: 6px 12px;
                                border-radius: 4px;
                                color: {current_theme.accent};
                                font-family: monospace;
                            ",
                            "Ctrl+Shift+S"
                        }
                    }
                }
            }

            // Application Preferences section
            section {
                style: "
                    background: {current_theme.bg_surface};
                    border: 1px solid {current_theme.border};
                    border-radius: 8px;
                    padding: 24px;
                    margin-bottom: 24px;
                ",

                h2 {
                    style: "
                        font-size: 1.3rem;
                        margin: 0 0 16px 0;
                        color: {current_theme.text_primary};
                    ",
                    "Application Preferences"
                }

                label {
                    style: "
                        display: flex;
                        align-items: center;
                        gap: 12px;
                        cursor: pointer;
                        color: {current_theme.text_secondary};
                    ",

                    input {
                        r#type: "checkbox",
                        checked: auto_start(),
                        oninput: move |e| auto_start.set(e.checked()),
                    }

                    span {
                        "Start with system"
                    }
                }
            }

            // Save button
            div {
                style: "display: flex; justify-content: flex-end; gap: 12px;",
                button {
                    style: "
                        padding: 10px 24px;
                        background: {current_theme.accent};
                        color: white;
                        border: none;
                        border-radius: 6px;
                        cursor: pointer;
                        font-family: monospace;
                        font-weight: 500;
                        transition: all 0.2s;
                        &:hover {{
                            opacity: 0.9;
                        }}
                    ",
                    onclick: move |_| {
                        // TODO: Save settings to config file
                        println!("Settings saved!");
                    },
                    "Save Settings"
                }
            }
        }
    }
}
