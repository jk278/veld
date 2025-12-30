//! Custom window title bar with drag region and window controls
//! 自定义窗口标题栏，支持拖拽区域和窗口控制按钮

use crate::routes::Route;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

/// Custom window title bar with drag region and window controls
#[component]
pub fn TitleBar() -> Element {
  let window = dioxus_desktop::use_window();
  let _navigator = use_navigator();
  let current_route = use_route::<Route>();

  // Clone window for use in closures
  let window_dblclick = window.clone();
  let window_max_btn = window.clone();
  let window_min = window.clone();
  let window_close = window.clone();

  rsx! {
    div {
      class: "flex items-center justify-between h-8 bg-bg-secondary select-none shrink-0",
      // app-region: drag makes WebView2 treat this as non-client area (enables drag, double-click maximize, right-click system menu)
      style: "user-select: none; app-region: drag;",

      // Left: Navigation (draggable via CSS)
      div {
        class: "flex items-center gap-1 flex-1 min-w-0",
        // Double-click to maximize
        ondoubleclick: move |_| { window_dblclick.toggle_maximized(); },

        // Navigation items (non-draggable)
        NavLink {
          route: Route::Home,
          current: current_route.clone(),
          label: "Chat",
        }
        NavLink {
          route: Route::Settings,
          current: current_route.clone(),
          label: "Settings",
        }
        NavLink {
          route: Route::About,
          current: current_route.clone(),
          label: "About",
        }

        div { class: "flex-1" }  // Spacer (extends drag area)
      }

      // Right: Window controls (non-draggable)
      div {
        class: "flex items-center shrink-0",
        // no-drag restores click interaction
        style: "app-region: no-drag;",

        WindowButton {
          icon: "−",
          tooltip: "Minimize",
          close: false,
          onclick: move |_| { window_min.set_minimized(true); },
        }
        WindowButton {
          icon: "□",
          tooltip: "Maximize",
          close: false,
          onclick: move |_| { window_max_btn.toggle_maximized(); },
        }
        WindowButton {
          icon: r"×",
          tooltip: "Close",
          close: true,
          onclick: move |_| { window_close.close(); },
        }
      }
    }
  }
}

/// Navigation link in title bar (non-draggable)
#[component]
fn NavLink(
  route: Route,
  current: Route,
  label: String,
) -> Element {
  let is_active = current == route;

  rsx! {
    div {
      // no-drag allows link clicks
      style: "app-region: no-drag;",
      Link {
        to: route.clone(),
        class: format!(
          "px-3 py-1 text-sm rounded-md transition-all duration-200 {}",
          if is_active {
            "text-text-primary bg-bg-tertiary/50"
          } else {
            "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary/30"
          }
        ),
        "{label}"
      }
    }
  }
}

/// Window control button
#[component]
fn WindowButton(
  icon: &'static str,
  tooltip: String,
  #[props(default = false)] close: bool,
  onclick: EventHandler<MouseEvent>,
) -> Element {
  rsx! {
    button {
      class: format!(
        "h-8 w-11 flex items-center justify-center text-text-secondary \
        transition-all duration-150 {}",
        if close {
          "hover:bg-red-600 hover:text-white"
        } else {
          "hover:bg-bg-tertiary hover:text-text-primary"
        }
      ),
      title: "{tooltip}",
      onclick: move |e| { onclick.call(e); },
      span { class: "text-base leading-none", "{icon}" }
    }
  }
}
