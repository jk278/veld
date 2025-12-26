//! Shared layout component for all pages
//! 提供页面间的统一布局和导航

use dioxus::prelude::*;
use crate::routes::Route;
use crate::theme::use_theme;

/// Application layout with navigation
/// 包含导航栏和页面内容的共享布局
#[component]
pub fn AppLayout() -> Element {
    let _theme_mode = use_theme();

    rsx! {
        div {
            id: "app-layout",
            class: "flex flex-col h-screen bg-bg-primary text-text-primary font-sans overflow-hidden",

            // Navigation header (fixed at top)
            nav {
                class: "flex items-center gap-4 px-5 py-1.5 bg-bg-secondary border-b border-border shrink-0",

                Link {
                    to: Route::Home,
                    class: "text-text-secondary hover:text-text-primary px-3 py-0.5 rounded-md transition-all duration-200 font-medium",
                    "Chat"
                }

                div { class: "flex-1" }

                Link {
                    to: Route::Settings,
                    class: "text-text-secondary hover:text-text-primary px-3 py-0.5 rounded-md transition-all duration-200",
                    "Settings"
                }

                Link {
                    to: Route::About,
                    class: "text-text-secondary hover:text-text-primary px-3 py-0.5 rounded-md transition-all duration-200",
                    "About"
                }
            }

            // Main content area (allow scrolling within content only)
            div {
                class: "flex-1 flex-col overflow-hidden p-3",

                Outlet::<Route> {}
            }
        }
    }
}
