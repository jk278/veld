//! Window size hook for responsive behavior
//! 窗口尺寸 Hook - 用于响应式布局

use crate::config::AppConfig;
use dioxus::prelude::*;

/// Responsive breakpoint: narrow screen threshold in pixels
/// 响应式断点：窄屏阈值（像素）
pub const RESPONSIVE_BREAKPOINT: f64 = 768.0;

/// Minimum window size (width, height) in pixels
/// 最小窗口尺寸（用于硬约束和保存过滤）
pub const MIN_WINDOW_SIZE: (u32, u32) = (400, 500);

/// Default window size (width, height) in pixels
/// 默认窗口尺寸（无配置时使用）
pub const DEFAULT_WINDOW_SIZE: (u32, u32) = (1200, 800);

/// Get current window width as reactive signal
/// NOTE: Returns actual window width, not RESPONSIVE_BREAKPOINT (which is UI threshold)
pub fn use_window_size() -> Signal<f64> {
  let window_width = use_signal(|| 0.0);  // Placeholder, updated immediately by use_effect
  let mut width = window_width.clone();

  // Get initial window size
  use_effect(move || {
    if let Ok(size) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      let window = dioxus_desktop::window();
      let scale = window.scale_factor();
      let physical = window.inner_size();
      physical.width as f64 / scale
    })) {
      width.set(size);
    }
  });

  // Listen for window resize events
  dioxus_desktop::use_wry_event_handler(move |event, _| {
    use dioxus_desktop::tao::event::{Event, WindowEvent};
    if let Event::WindowEvent { event, .. } = event {
      if let WindowEvent::Resized(physical_size) = event {
        let window = dioxus_desktop::window();
        let scale = window.scale_factor();
        let logical_width = physical_size.width as f64 / scale;
        width.set(logical_width);
      }
    }
  });

  window_width
}

/// Persist window size across app restarts
pub fn use_window_save() {
  use_resource(move || {
    async move {
      let mut last_size = None;
      loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let size = dioxus_desktop::window().inner_size();
        let current = (size.width, size.height);

        if last_size != Some(current) && size.width >= MIN_WINDOW_SIZE.0 && size.height >= MIN_WINDOW_SIZE.1 {
          if let Ok(mut config) = AppConfig::load() {
            config.update_window_size(size.width, size.height);
            last_size = Some(current);
          }
        }
      }
    }
  });
}

/// Sidebar resize with drag handle
/// Returns (width_signal, is_resizing, start_resize_callback)
///
/// NOTE: Uses JavaScript global events instead of `use_wry_event_handler` because:
/// - Dioxus 0.7 lacks official gesture/drag support
/// - Desktop API has limited mouse event capture (mousemove/mouseup issues)
/// - Third-party `dioxus-use-gesture` is unmaintained
/// - JS provides reliable cross-platform event handling
pub fn use_sidebar_resize() -> (Signal<u32>, Signal<bool>, Callback<()>) {
  let sidebar_width = use_signal(|| {
    AppConfig::load()
      .map(|c| c.ui.sidebar_width)
      .unwrap_or(280)
  });

  let is_resizing = use_signal(|| false);

  // Apply width to CSS variable when changed
  use_effect(move || {
    let width = sidebar_width();
    let _ = dioxus::document::eval(&format!(
      "document.documentElement.style.setProperty('--sidebar-width', '{width}px');"
    ));
  });

  // Start drag with JavaScript global events
  // Range: 180-500px (button min width ~140px + padding; max to avoid squeezing main chat area)
  let start_resize = Callback::new({
    let mut is_resizing = is_resizing.clone();
    let current_width = sidebar_width.clone();
    move |()| {
      is_resizing.set(true);
      let min_width = (current_width() - 120).max(180);
      let max_width = (current_width() + 120).min(500);
      let _ = dioxus::document::eval(&format!(
        r#"
          const minWidth = {min_width};
          const maxWidth = {max_width};
          document.documentElement.style.setProperty('user-select', 'none');
          window.__sidebarDragHandler = (e) => {{
            const newWidth = Math.max(minWidth, Math.min(maxWidth, e.clientX));
            document.documentElement.style.setProperty('--sidebar-width', newWidth + 'px');
            window.__sidebarDragWidth = newWidth;
          }};
          window.__sidebarDragEnd = () => {{
            window.removeEventListener('mousemove', window.__sidebarDragHandler);
            window.removeEventListener('mouseup', window.__sidebarDragEnd);
            document.documentElement.style.removeProperty('user-select');
          }};
          window.addEventListener('mousemove', window.__sidebarDragHandler);
          window.addEventListener('mouseup', window.__sidebarDragEnd);
        "#,
      ));
    }
  });

  // Poll for width changes during drag and save on end
  use_resource({
    let mut sidebar_width = sidebar_width.clone();
    move || {
      async move {
        let mut last_saved = None;
        loop {
          tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
          if is_resizing() {
            if let Ok(width) = dioxus::document::eval("window.__sidebarDragWidth || null").await {
              if let Some(w) = width.as_u64() {
                sidebar_width.set(w as u32);
                last_saved = Some(w as u32);
              }
            }
          } else if let Some(to_save) = last_saved.take() {
            if let Ok(mut config) = AppConfig::load() {
              config.update_sidebar_width(to_save);
            }
          }
        }
      }
    }
  });

  (sidebar_width, is_resizing, start_resize)
}

/// Check if current window width is narrow screen (< RESPONSIVE_BREAKPOINT)
pub fn use_is_narrow_screen() -> bool {
  use_window_size()() < RESPONSIVE_BREAKPOINT
}

/// Create a sidebar collapse state that responds to window size
/// Returns: (collapse_signal, window_width_signal)
pub fn use_responsive_sidebar() -> (Signal<bool>, Signal<f64>) {
  let window_width = use_window_size();

  let collapsed = use_signal(|| {
    // Initial state: collapsed on narrow, expanded on wide
    window_width() < RESPONSIVE_BREAKPOINT
  });

  // Auto-update collapse state when window resizes
  let mut collapsed_clone = collapsed.clone();
  dioxus_desktop::use_wry_event_handler(move |event, _| {
    use dioxus_desktop::tao::event::{Event, WindowEvent};
    if let Event::WindowEvent { event, .. } = event {
      if let WindowEvent::Resized(physical_size) = event {
        let window = dioxus_desktop::window();
        let scale = window.scale_factor();
        let logical_width = physical_size.width as f64 / scale;
        let should_collapse = logical_width < RESPONSIVE_BREAKPOINT;
        if should_collapse != collapsed_clone() {
          collapsed_clone.set(should_collapse);
        }
      }
    }
  });

  (collapsed, window_width)
}
