//! Shared layout component for all pages
//! 提供页面间的统一布局和导航

use crate::components::home::ACTIVATE_INPUT_TRIGGER;
use crate::components::title_bar::TitleBar;
use crate::routes::Route;
use crate::theme::use_theme;
use dioxus::prelude::*;
use dioxus_router::hooks::use_route;

/// Global navigation trigger (shared with main.rs)
pub static NAVIGATE_HOME_TRIGGER: std::sync::OnceLock<std::sync::Arc<std::sync::Mutex<u64>>> =
  std::sync::OnceLock::new();

/// Application layout with navigation
/// 包含导航栏和页面内容的共享布局
#[component]
pub fn AppLayout() -> Element {
  let _theme_mode = use_theme();
  let navigator = use_navigator();

  // Track if we need to activate input after navigation
  let mut pending_activation = use_signal(|| false);

  // Listen for navigation trigger from main.rs
  let mut last_nav_count = use_signal(|| 0u64);
  use_resource(move || {
    async move {
      loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Some(trigger) = NAVIGATE_HOME_TRIGGER.get() {
          if let Ok(count) = trigger.lock() {
            let current = *count;
            if current > last_nav_count() {
              last_nav_count.set(current);
              // Set pending activation flag and navigate
              pending_activation.set(true);
              navigator.push(Route::Home {});
            }
          }
        }
      }
    }
  });

  // After route changes to Home, trigger input activation
  let current_route = use_route::<Route>();
  use_effect(move || {
    if pending_activation() {
      match current_route {
        Route::Home {} => {
          // Route is now Home, trigger input activation
          pending_activation.set(false);
          if let Some(input_trigger) = ACTIVATE_INPUT_TRIGGER.get() {
            if let Ok(mut count) = input_trigger.lock() {
              *count += 1;
            }
          }
        }
        _ => {}
      }
    }
  });

  rsx! {
    div {
      id: "app-layout",
      class: "flex flex-col h-screen bg-bg-primary text-text-primary font-sans overflow-hidden",

      // Custom title bar with drag region and window controls
      TitleBar {}

      // Main content area (allow scrolling within content only)
      div {
        class: "flex-1 flex-col overflow-hidden",
        Outlet::<Route> {

        }
      }
    }
  }
}
