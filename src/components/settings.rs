//! Settings page component
//! 设置页面组件

use dioxus::prelude::*;
use crate::theme::use_theme;
use crate::config::ThemeMode;

/// Settings page
/// 应用设置页面
#[component]
pub fn Settings() -> Element {
    let mut theme_mode = use_theme();

    let mut api_key = use_signal(|| "".to_string());
    let mut selected_model = use_signal(|| "gpt-4".to_string());
    let mut auto_start = use_signal(|| false);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",

            h1 {
                class: "text-3xl font-light text-text-primary mb-8",
                "Settings"
            }

            // Theme selector
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    "🎨 Appearance"
                }

                div { class: "flex flex-wrap gap-2 items-center",
                    label {
                        class: "text-text-secondary text-sm font-medium mr-2",
                        "Theme:"
                    }
                    button {
                        class: if theme_mode() == ThemeMode::Light {
                            "px-3 py-1.5 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                        } else {
                            "px-3 py-1.5 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border"
                        },
                        onclick: move |_| theme_mode.set(ThemeMode::Light),
                        "☀️ Light"
                    }
                    button {
                        class: if theme_mode() == ThemeMode::Dark {
                            "px-3 py-1.5 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                        } else {
                            "px-3 py-1.5 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border"
                        },
                        onclick: move |_| theme_mode.set(ThemeMode::Dark),
                        "🌙 Dark"
                    }
                    button {
                        class: if theme_mode() == ThemeMode::System {
                            "px-3 py-1.5 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                        } else {
                            "px-3 py-1.5 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border"
                        },
                        onclick: move |_| theme_mode.set(ThemeMode::System),
                        "🖥️ System"
                    }
                }
            }

            // AI Configuration section
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    "🤖 AI Configuration"
                }

                div {
                    class: "space-y-2",
                    label {
                        class: "block text-text-secondary text-sm font-medium",
                        "API Provider"
                    }
                    select {
                        class: "w-full p-2.5 bg-bg-surface text-text-primary border border-border rounded-md font-mono focus:border-primary focus:outline-none",
                        value: selected_model(),
                        oninput: move |e| selected_model.set(e.value()),

                        option { value: "gpt-4", "OpenAI GPT-4" }
                        option { value: "gpt-3.5-turbo", "OpenAI GPT-3.5 Turbo" }
                        option { value: "claude-3", "Anthropic Claude 3" }
                    }
                }

                div {
                    class: "space-y-2",
                    label {
                        class: "block text-text-secondary text-sm font-medium",
                        "API Key"
                    }
                    input {
                        class: "w-full p-2.5 bg-bg-surface text-text-primary border border-border rounded-md font-mono outline-none transition-all focus:border-primary",
                        r#type: "password",
                        placeholder: "Enter your API key...",
                        value: api_key(),
                        oninput: move |e| api_key.set(e.value()),
                    }
                }
            }

            // Keyboard Shortcuts section
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    "⌨️ Keyboard Shortcuts"
                }

                div {
                    class: "space-y-3",
                    div {
                        class: "flex justify-between items-center py-2 border-b border-border last:border-b-0",
                        span {
                            class: "text-text-secondary",
                            "Show Floating Input"
                        }
                        code {
                            class: "px-3 py-1 bg-bg-primary text-primary rounded font-mono text-sm",
                            "Ctrl+Shift+Space"
                        }
                    }
                    div {
                        class: "flex justify-between items-center py-2 border-b border-border last:border-b-0",
                        span {
                            class: "text-text-secondary",
                            "Quick Summarize"
                        }
                        code {
                            class: "px-3 py-1 bg-bg-primary text-primary rounded font-mono text-sm",
                            "Ctrl+Shift+S"
                        }
                    }
                }
            }

            // Application Preferences section
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    "⚙️ Application Preferences"
                }

                label {
                    class: "flex items-center gap-3 cursor-pointer text-text-secondary hover:text-text-primary transition-colors",

                    input {
                        r#type: "checkbox",
                        checked: auto_start(),
                        oninput: move |e| auto_start.set(e.checked()),
                        class: "w-4 h-4 text-primary bg-bg-surface border-border rounded focus:ring-primary focus:ring-2",
                    }

                    span {
                        "Start with system"
                    }
                }
            }

            // Save button
            div {
                class: "flex justify-end gap-3 pt-4",
                button {
                    class: "btn-primary",
                    onclick: move |_| {
                        // TODO: Save settings to config file
                        println!("Settings saved!");
                    },
                    "💾 Save Settings"
                }
            }
        }
    }
}
