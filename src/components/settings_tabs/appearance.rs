//! Appearance tab component
//! 外观设置标签页

use dioxus::prelude::*;
use crate::config::ThemeMode;
use crate::theme::use_theme;

/// Appearance tab content
#[component]
pub fn AppearanceTab() -> Element {
    let mut theme_mode = use_theme();

    rsx! {
        div {
            class: "space-y-6",
            h1 {
                class: "text-2xl font-semibold text-text-primary",
                "Appearance"
            }

            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
                h2 {
                    class: "text-lg text-text-primary mb-4",
                    "Theme"
                }
                div {
                    class: "flex flex-wrap gap-2 items-center",
                    for (mode, label, icon) in [
                        (ThemeMode::Light, "Light", "☀️"),
                        (ThemeMode::Dark, "Dark", "🌙"),
                        (ThemeMode::System, "System", "🖥️"),
                    ] {
                        button {
                            class: if theme_mode() == mode {
                                "px-4 py-2 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                            } else {
                                "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border hover:bg-bg-secondary"
                            },
                            onclick: move |_| theme_mode.set(mode),
                            span { class: "mr-2", "{icon}" }
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
