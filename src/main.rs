use crate::components::home::ACTIVATE_INPUT_TRIGGER;
use crate::components::layout::NAVIGATE_HOME_TRIGGER;
use crate::config::AppConfig;
use crate::routes::Route;
use crate::shortcuts::ShortcutManager;
use crate::theme::init_theme;
use dioxus::prelude::*;
use dioxus_desktop::{
  tao::window::{Icon, WindowBuilder}, trayicon::TrayIconEvent,
  use_global_shortcut, use_muda_event_handler, use_tray_icon_event_handler,
};
use std::sync::{Arc, Mutex};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn load_window_icon() -> Option<Icon> {
  let bytes = include_bytes!("../assets/favicon.ico");
  let img = image::load_from_memory(bytes).ok()?;
  let rgba = img.to_rgba8();
  let (width, height) = rgba.dimensions();
  Icon::from_rgba(rgba.into_raw(), width, height).ok()
}

fn main() {
  // Initialize triggers
  ACTIVATE_INPUT_TRIGGER.set(Arc::new(Mutex::new(0))).unwrap();
  NAVIGATE_HOME_TRIGGER.set(Arc::new(Mutex::new(0))).unwrap();

  let tray = match crate::tray::SystemTray::new() {
    Ok(tray) => {
      println!("System tray initialized successfully");
      Some(tray)
    }
    Err(e) => {
      println!("Failed to initialize system tray: {:?}", e);
      None
    }
  };

  if let Some(tray) = tray {
    std::mem::forget(tray);
  }

  // Load persisted window size from config
  let (window_width, window_height) = AppConfig::load()
    .ok()
    .and_then(|c| Some((c.ui.window_width?, c.ui.window_height?)))
    .unwrap_or(crate::hooks::DEFAULT_WINDOW_SIZE);

  // 配置窗口：隐藏原生标题栏，使用自定义标题栏
  // NOTE: decorations(false) 导致 resize 区域仅 1-2px (tao 限制，Electron 有 titleBarStyle: "custom")
  let mut window = WindowBuilder::new()
    .with_title("Veld - AI Toolkit")
    .with_decorations(false)  // Hide native titlebar for custom title bar
    .with_resizable(true)
    .with_min_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(
      crate::hooks::MIN_WINDOW_SIZE.0 as f64,
      crate::hooks::MIN_WINDOW_SIZE.1 as f64,
    ))
    .with_window_icon(load_window_icon());

  // Restore window size if available
  if let Some(inner_size) = dioxus_desktop::tao::dpi::LogicalSize::new(
    window_width as f64,
    window_height as f64,
  ).into()
  {
    window = window.with_inner_size(inner_size);
  }

  dioxus::LaunchBuilder::new()
    .with_cfg(
      dioxus::desktop::Config::new()
        .with_window(window)
        .with_menu(None),
    )
    .launch(App);
}

#[component]
fn App() -> Element {
  // Initialize theme and provide context
  let theme_context = init_theme();
  provide_context(theme_context.clone());

  // Enable window size save/restore
  crate::hooks::use_window_save();

  // Load shortcut from config
  let activate_shortcut = use_signal(|| {
    crate::config::AppConfig::load()
      .map(|c| {
        c.shortcuts
          .command_palette
          .unwrap_or_else(|| "Ctrl+Shift+Space".to_string())
      })
      .unwrap_or_else(|_| "Ctrl+Shift+Space".to_string())
  });

  // Global hotkey handler
  let _shortcut_handle = use_global_shortcut(activate_shortcut().as_str(), move |state| {
    if state == dioxus_desktop::HotKeyState::Pressed {
      println!("[App] Global hotkey triggered!");

      // Restore window
      let window = dioxus::desktop::window();
      window.set_minimized(false);

      // Trigger navigation (AppLayout will trigger input activation after navigation)
      if let Some(nav_trigger) = NAVIGATE_HOME_TRIGGER.get() {
        if let Ok(mut count) = nav_trigger.lock() {
          *count += 1;
        }
      }
    }
  });

  // Tray icon click handler
  use_tray_icon_event_handler(move |event| {
    match event {
      TrayIconEvent::Click { button, .. } => {
        if *button == dioxus_desktop::trayicon::MouseButton::Left {
          println!("[App] Tray icon clicked!");

          let window = dioxus::desktop::window();
          window.set_minimized(false);

          // Trigger navigation (AppLayout will trigger input activation after navigation)
          if let Some(nav_trigger) = NAVIGATE_HOME_TRIGGER.get() {
            if let Ok(mut count) = nav_trigger.lock() {
              *count += 1;
            }
          }
        }
      }
      _ => {}
    }
  });

  // Tray menu handler - use_muda_event_handler is the correct API in 0.7.2
  use_muda_event_handler(move |event: &dioxus_desktop::muda::MenuEvent| {
    println!("[App] Menu event: id={:?}", event.id);
    match event.id.as_ref() {
      "show" => {
        println!("[App] Tray menu 'Show' clicked!");

        let window = dioxus::desktop::window();
        window.set_minimized(false);

        // Trigger navigation (AppLayout will trigger input activation after navigation)
        if let Some(nav_trigger) = NAVIGATE_HOME_TRIGGER.get() {
          if let Ok(mut count) = nav_trigger.lock() {
            *count += 1;
          }
        }
      }
      "quit" => std::process::exit(0),
      _ => {}
    }
  });

  use_effect(move || match ShortcutManager::new() {
    Ok(_) => {
      println!("Global shortcuts initialized");
      println!("Press {} to activate chat input", activate_shortcut());
    }
    Err(e) => eprintln!("Failed to initialize shortcuts: {:?}", e),
  });

  rsx! {
    document::Link {
      rel: "icon",
      href: FAVICON,
    }
    // NOTE: FOUC (Flash of Unstyled Content) on startup
    //
    // Known limitations causing FOUC:
    // 1. Tailwind CSS v4 scanner has limited support for Rust `class: "..."` syntax
    //    - Can detect most static class names, generates ~64KB CSS (256 selectors)
    //    - Verified: Used classes like .max-w-4xl, .bg-bg-surface, .flex are present
    //    - See: tailwind.config.js for details
    //
    // 2. document::Stylesheet loads asynchronously in WebView
    //    - No onmounted/onload event support (Dioxus limitation)
    //    - See: https://github.com/DioxusLabs/dioxus/issues/3758
    //
    // 3. General CSS loading timing issue in Dioxus Desktop
    //    - See: https://github.com/DioxusLabs/dioxus/issues/2847
    //
    // Current approach: Accept brief FOUC as trade-off
    // - Adding delays (500ms) is unacceptable for development UX
    // - Sidebar components use inline styles to prevent most visible FOUC
    //
    // Future directions to monitor:
    // - Official Dioxus CSS loading improvements
    // - Tailwind v4 @source directive for better Rust support
    // - Alternative: safelist, critical CSS extraction, or Tailwind v3
    document::Stylesheet {
      href: TAILWIND_CSS,
    }

    Router::<Route> {

    }
  }
}

pub mod chat_history;
pub mod components;
pub mod config;
pub mod hooks;
pub mod routes;
pub mod services;
pub mod shortcuts;
pub mod theme;
pub mod tray;

#[cfg(test)]
mod tests {
  #[test]
  fn test_main() {
    assert_eq!(2 + 2, 4);
  }
}
