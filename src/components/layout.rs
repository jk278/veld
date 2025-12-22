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
            class: "flex flex-col min-h-screen bg-bg-primary text-text-primary font-sans",
            style: "background: {current_theme.bg_primary}; color: {current_theme.text_primary}",

            // Navigation header (sticky at top)
            nav {
                class: "flex items-center gap-4 px-5 py-3 bg-bg-secondary border-b border-border sticky top-0 z-10",
                style: "background: {current_theme.bg_secondary}; border-bottom-color: {current_theme.border}",

                Link {
                    to: Route::Home,
                    class: "text-text-secondary hover:text-text-primary px-3 py-1.5 rounded-md transition-all duration-200 font-medium",
                    style: "color: {current_theme.text_secondary}",
                    "Veld"
                }

                div { class: "flex-1" }

                Link {
                    to: Route::Settings,
                    class: "text-text-secondary hover:text-text-primary px-3 py-1.5 rounded-md transition-all duration-200",
                    style: "color: {current_theme.text_secondary}",
                    "Settings"
                }

                Link {
                    to: Route::About,
                    class: "text-text-secondary hover:text-text-primary px-3 py-1.5 rounded-md transition-all duration-200",
                    style: "color: {current_theme.text_secondary}",
                    "About"
                }
            }

            // Main content area (allow scrolling within content only)
            div {
                class: "flex-1 overflow-auto",
                style: "padding: 1.25rem",  // p-5 equivalent

                Outlet::<Route> {}
            }
        }
    }
}
