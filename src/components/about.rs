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
            style: "
                max-width: 800px;
                margin: 0 auto;
            ",

            h1 {
                style: "
                    font-size: 2rem;
                    font-weight: 300;
                    margin-bottom: 2rem;
                    color: {theme().text_primary};
                ",
                "About Veld"
            }

            div {
                style: "
                    background: {theme().bg_surface};
                    border: 1px solid {theme().border};
                    border-radius: 8px;
                    padding: 24px;
                    margin-bottom: 24px;
                ",

                h2 {
                    style: "
                        font-size: 1.3rem;
                        margin: 0 0 16px 0;
                        color: {theme().text_primary};
                    ",
                    "Veld - AI Toolkit for Developers"
                }

                p {
                    style: "
                        color: {theme().text_secondary};
                        line-height: 1.6;
                        margin-bottom: 16px;
                    ",
                    "Veld is a cross-platform system tray tool that provides quick access to AI assistant functionality through keyboard shortcuts. Built with Dioxus 0.7."
                }

                p {
                    style: "
                        color: {theme().text_secondary};
                        line-height: 1.6;
                    ",
                    "Features include floating input windows, pre-configured prompts, context menus, and real-time AI interaction to improve developer productivity."
                }
            }

            div {
                style: "
                    background: {theme().bg_surface};
                    border: 1px solid {theme().border};
                    border-radius: 8px;
                    padding: 24px;
                ",

                h2 {
                    style: "
                        font-size: 1.3rem;
                        margin: 0 0 16px 0;
                        color: {theme().text_primary};
                    ",
                    "Technology Stack"
                }

                ul {
                    style: "
                        list-style: none;
                        padding: 0;
                    ",
                    li {
                        style: "
                            padding: 8px 0;
                            color: {theme().text_secondary};
                            font-family: monospace;
                        ",
                        "• Dioxus 0.7 - UI framework"
                    }
                    li {
                        style: "
                            padding: 8px 0;
                            color: {theme().text_secondary};
                            font-family: monospace;
                        ",
                        "• Rust - Programming language"
                    }
                    li {
                        style: "
                            padding: 8px 0;
                            color: {theme().text_secondary};
                            font-family: monospace;
                        ",
                        "• Tao/Wry - Window management"
                    }
                    li {
                        style: "
                            padding: 8px 0;
                            color: {theme().text_secondary};
                            font-family: monospace;
                        ",
                        "• System Tray API - Cross-platform tray support"
                    }
                }
            }
        }
    }
}
