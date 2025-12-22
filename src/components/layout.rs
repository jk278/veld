//! Shared layout component for all pages
//! 提供页面间的统一布局和导航

use dioxus::prelude::*;
use crate::routes::Route;
use crate::theme::{use_theme};

/// Application layout with navigation
/// 包含导航栏和页面内容的共享布局
#[component]
pub fn AppLayout() -> Element {
    let (_theme_mode, theme) = use_theme();
    let current_theme = theme();

    rsx! {
        div {
            id: "app-layout",
            style: "
                display: flex;
                flex-direction: column;
                min-height: 100vh;
                background: {current_theme.bg_primary};
                color: {current_theme.text_primary};
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            ",
            // Navigation header
            nav {
                style: "
                    display: flex;
                    align-items: center;
                    padding: 12px 20px;
                    background: {current_theme.bg_secondary};
                    border-bottom: 1px solid {current_theme.border};
                    gap: 16px;
                ",

                Link {
                    to: Route::Home,
                    style: "
                        color: {current_theme.text_secondary};
                        text-decoration: none;
                        padding: 6px 12px;
                        border-radius: 6px;
                        transition: all 0.2s;
                        font-weight: 500;
                        &:hover {{
                            color: {current_theme.text_primary};
                            background: {current_theme.accent};
                            opacity: 0.1;
                        }}
                    ",
                    "Veld"
                }

                div { style: "flex: 1;" }

                Link {
                    to: Route::Settings,
                    style: "
                        color: {current_theme.text_secondary};
                        text-decoration: none;
                        padding: 6px 12px;
                        border-radius: 6px;
                        transition: all 0.2s;
                        &:hover {{
                            color: {current_theme.text_primary};
                            background: {current_theme.accent};
                            opacity: 0.1;
                        }}
                    ",
                    "Settings"
                }

                Link {
                    to: Route::About,
                    style: "
                        color: {current_theme.text_secondary};
                        text-decoration: none;
                        padding: 6px 12px;
                        border-radius: 6px;
                        transition: all 0.2s;
                        &:hover {{
                            color: {current_theme.text_primary};
                            background: {current_theme.accent};
                            opacity: 0.1;
                        }}
                    ",
                    "About"
                }
            }

            // Main content area
            main {
                style: "
                    flex: 1;
                    overflow: auto;
                    padding: 20px;
                ",
                Outlet::<Route> {}
            }
        }
    }
}
