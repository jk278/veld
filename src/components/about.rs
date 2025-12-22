//! About page
//! 关于页面

use dioxus::prelude::*;
use crate::theme::{use_theme};

/// About component
/// 显示应用信息
#[component]
pub fn About() -> Element {
    let (_, theme) = use_theme();

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",

            h1 {
                class: "text-3xl font-light text-text-primary mb-8",
                style: "color: {theme().text_primary}",
                "About Veld"
            }

            div {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
                style: "background: {theme().bg_surface}; border-color: {theme().border}",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    style: "color: {theme().text_primary}",
                    "🦀 Veld - AI Toolkit for Developers"
                }

                p {
                    class: "text-text-secondary leading-relaxed",
                    style: "color: {theme().text_secondary}; line-height: 1.6; margin-bottom: 16px",
                    "Veld is a cross-platform system tray tool that provides quick access to AI assistant functionality through keyboard shortcuts. Built with Dioxus 0.7."
                }

                p {
                    class: "text-text-secondary leading-relaxed",
                    style: "color: {theme().text_secondary}; line-height: 1.6",
                    "Features include floating input windows, pre-configured prompts, context menus, and real-time AI interaction to improve developer productivity."
                }
            }

            div {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
                style: "background: {theme().bg_surface}; border-color: {theme().border}",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    style: "color: {theme().text_primary}",
                    "🛠️ Technology Stack"
                }

                ul {
                    class: "space-y-2",
                    li {
                        class: "flex items-center gap-3 text-text-secondary font-mono text-sm",
                        style: "color: {theme().text_secondary}; padding: 8px 0",
                        span { class: "text-primary", "▸" }
                        "Dioxus 0.7 - UI framework"
                    }
                    li {
                        class: "flex items-center gap-3 text-text-secondary font-mono text-sm",
                        style: "color: {theme().text_secondary}; padding: 8px 0",
                        span { class: "text-primary", "▸" }
                        "Rust - Programming language"
                    }
                    li {
                        class: "flex items-center gap-3 text-text-secondary font-mono text-sm",
                        style: "color: {theme().text_secondary}; padding: 8px 0",
                        span { class: "text-primary", "▸" }
                        "TailwindCSS - Styling framework"
                    }
                    li {
                        class: "flex items-center gap-3 text-text-secondary font-mono text-sm",
                        style: "color: {theme().text_secondary}; padding: 8px 0",
                        span { class: "text-primary", "▸" }
                        "Tao/Wry - Window management"
                    }
                    li {
                        class: "flex items-center gap-3 text-text-secondary font-mono text-sm",
                        style: "color: {theme().text_secondary}; padding: 8px 0",
                        span { class: "text-primary", "▸" }
                        "System Tray API - Cross-platform tray support"
                    }
                }
            }
        }
    }
}
