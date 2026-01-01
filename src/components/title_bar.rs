//! Custom window title bar with drag region and window controls
//! 自定义窗口标题栏，支持拖拽区域和窗口控制按钮

use crate::components::icons::{ChatIcon, CloseIcon, MaximizeIcon, MinimizeIcon, SettingsIcon};
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
      class: "flex items-center justify-between h-10 bg-bg-secondary select-none shrink-0",
      // app-region: drag makes WebView2 treat this as non-client area (enables drag, double-click maximize, right-click system menu)
      style: "user-select: none; app-region: drag;",

      // Left: Navigation (draggable via CSS)
      div {
        class: "flex items-center gap-3 flex-1 min-w-0 pl-4",
        // Double-click to maximize
        ondoubleclick: move |_| { window_dblclick.toggle_maximized(); },

        // Navigation items (non-draggable)
        NavLink {
          route: Route::Home,
          current: current_route.clone(),
          tooltip: "Chat",
          ChatIcon { class: "w-4 h-4" },
        }
        NavLink {
          route: Route::Settings,
          current: current_route.clone(),
          tooltip: "Settings",
          SettingsIcon { class: "w-4 h-4" },
        }

        div { class: "flex-1" }  // Spacer (extends drag area)
      }

      // Right: Window controls (non-draggable)
      div {
        class: "flex items-center shrink-0",
        // no-drag restores click interaction
        style: "app-region: no-drag;",

        WindowButton {
          tooltip: "Minimize",
          close: false,
          onclick: move |_| { window_min.set_minimized(true); },
          MinimizeIcon {},
        }
        WindowButton {
          tooltip: "Maximize",
          close: false,
          onclick: move |_| { window_max_btn.toggle_maximized(); },
          MaximizeIcon {},
        }
        WindowButton {
          tooltip: "Close",
          close: true,
          onclick: move |_| { window_close.close(); },
          CloseIcon {},
        }
      }
    }
  }
}

/// Navigation link in title bar (non-draggable)
///
/// Design principles:
/// - Match window buttons: h-10 (40px) full-height square
/// - Large click area: modern, easy to use
/// - Subtle feedback: hover fills entire square
#[component]
fn NavLink(
  route: Route,
  current: Route,
  tooltip: String,
  children: Element,
) -> Element {
  let is_active = current == route;

  // Match WindowButton pattern: h-10 w-10 square
  let nav_class = if is_active {
    // Active: solid background with elevation
    "bg-bg-surface shadow-sm"
  } else {
    // Default: visible hover feedback
    "hover:bg-gray-200 dark:hover:bg-gray-700"
  };

  rsx! {
    div {
      style: "app-region: no-drag;",
      Link {
        to: route.clone(),
        class: "nav-link h-10 w-10 flex items-center justify-center transition-all duration-150 text-text-secondary hover:text-text-primary {nav_class}",
        title: "{tooltip}",
        {children}
      }
    }
  }
}

/// Window control button
#[component]
fn WindowButton(
  tooltip: String,
  #[props(default = false)] close: bool,
  onclick: EventHandler<MouseEvent>,
  children: Element,
) -> Element {
  // NOTE: Compute class outside rsx! to avoid Dioxus hot reload bugs
  let hover_class = if close {
    "hover:bg-red-600 hover:text-white".to_string()
  } else {
    "hover:bg-gray-200 hover:text-text-primary border border-transparent hover:border-border dark:hover:bg-gray-700".to_string()
  };
  let base_class = "h-10 w-10 flex items-center justify-center text-text-secondary transition-all duration-150";
  let full_class = format!("{} {}", base_class, hover_class);

  rsx! {
    button {
      class: "{full_class}",
      title: "{tooltip}",
      onclick: move |e| { onclick.call(e); },
      {children}
    }
  }
}
