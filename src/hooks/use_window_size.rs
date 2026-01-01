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

/// Default sidebar width in pixels
/// 默认侧边栏宽度
pub const DEFAULT_SIDEBAR_WIDTH: u32 = 280;

/// Get current window width as reactive signal
/// NOTE: Returns actual window width, not RESPONSIVE_BREAKPOINT (which is UI threshold)
pub fn use_window_size() -> Signal<f64> {
  let mut width = use_signal(|| {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      let window = dioxus_desktop::window();
      let scale = window.scale_factor();
      let physical = window.inner_size();
      physical.width as f64 / scale
    })).unwrap_or(0.0)
  });

  // Listen for window resize events
  dioxus_desktop::use_wry_event_handler(move |event, _| {
    use dioxus_desktop::tao::event::{Event, WindowEvent};
    if let Event::WindowEvent { event, .. } = event {
      if let WindowEvent::Resized(physical_size) = event {
        let window = dioxus_desktop::window();
        let scale = window.scale_factor();
        width.set(physical_size.width as f64 / scale);
      }
    }
  });

  width
}

/// Persist window size across app restarts
pub fn use_window_save() {
  use_resource(move || {
    async move {
      // Initialize last_size from config to avoid overwriting on startup
      let mut last_size = AppConfig::load()
        .ok()
        .and_then(|c| Some((c.ui.window_width?, c.ui.window_height?)));

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
/// Returns (is_resizing, start_resize_callback)
///
/// NOTE: CSS variable (--sidebar-width) is the single source of truth for width
/// - Initialized once in AppLayout to prevent race conditions
/// - Config file only used for persistence (not initial read here)
/// - No signal synchronization needed between Rust and CSS
///
/// CRITICAL: Event-driven architecture required (polling has race conditions)
/// - JavaScript eval() is async; first poll may execute before JS sets state
/// - Fast drags complete within 50ms poll interval, causing incorrect width reads
/// - CustomEvent + recv() ensures accurate final width delivery
pub fn use_sidebar_resize() -> (Signal<bool>, Callback<()>) {
  let is_resizing = use_signal(|| false);

  // Listen for drag end events from JavaScript
  let mut rx = dioxus::document::eval(
    r#"
      window.addEventListener('sidebar-resize-end', (e) => {
        dioxus.send(e.detail);
      });
    "#
  );

  // Save width when drag ends
  use_resource({
    let mut is_resizing = is_resizing.clone();
    move || async move {
      while let Ok(width) = rx.recv::<u64>().await {
        if let Ok(mut config) = AppConfig::load() {
          config.update_sidebar_width(width as u32);
        }
        is_resizing.set(false);
      }
    }
  });

  // Start drag with JavaScript global events
  let start_resize = Callback::new({
    let mut is_resizing = is_resizing.clone();
    move |()| {
      is_resizing.set(true);
      let _ = dioxus::document::eval(&format!(
        r#"
          const current = parseInt(getComputedStyle(document.documentElement).getPropertyValue('--sidebar-width')) || {0};
          const MIN_WIDTH = 200;
          const MAX_WIDTH = 400;
          const sidebar = document.querySelector('.drawer-sidebar');
          const dragHandle = document.querySelector('.sidebar-drag-handle');

          document.documentElement.style.setProperty('user-select', 'none');
          sidebar.style.transition = 'none';  // Disable for instant response
          dragHandle.classList.add('active');  // Highlight during drag

          // NOTE: Track width in mousemove (not read from CSS later)
          // WARNING: Reading CSS after drag end may get stale value
          // - setTimeout(..., 0) can execute before CSS update completes
          // - getComputedStyle() reflects previous width during fast drags
          window.__sidebarLastWidth = current;

          window.__sidebarDragHandler = (e) => {{
            const newWidth = Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, e.clientX));
            document.documentElement.style.setProperty('--sidebar-width', newWidth + 'px');
            window.__sidebarLastWidth = newWidth;  // Sync with CSS update
          }};

          window.__sidebarDragEnd = () => {{
            window.removeEventListener('mousemove', window.__sidebarDragHandler);
            window.removeEventListener('mouseup', window.__sidebarDragEnd);
            document.documentElement.style.removeProperty('user-select');
            sidebar.style.transition = '';  // Restore transition
            dragHandle.classList.remove('active');  // Remove highlight
            const finalWidth = window.__sidebarLastWidth;
            delete window.__sidebarLastWidth;
            // Send final width to Rust via CustomEvent
            window.dispatchEvent(new CustomEvent('sidebar-resize-end', {{ detail: finalWidth }}));
          }};

          window.addEventListener('mousemove', window.__sidebarDragHandler);
          window.addEventListener('mouseup', window.__sidebarDragEnd);
        "#,
        DEFAULT_SIDEBAR_WIDTH
      ));
    }
  });

  (is_resizing, start_resize)
}

/// Check if current window width is narrow screen (< RESPONSIVE_BREAKPOINT)
pub fn use_is_narrow_screen() -> bool {
  use_window_size()() < RESPONSIVE_BREAKPOINT
}

/// Create a sidebar collapse state that responds to window size
/// Returns: (collapse_signal, window_width_signal)
pub fn use_responsive_sidebar() -> (Signal<bool>, Signal<f64>) {
  let window_width = use_window_size();
  let mut collapsed = use_signal(|| window_width() < RESPONSIVE_BREAKPOINT);

  // Auto-update collapse state when window resizes
  use_effect({
    let window_width = window_width.clone();
    move || {
      let should_collapse = window_width() < RESPONSIVE_BREAKPOINT;
      if collapsed() != should_collapse {
        collapsed.set(should_collapse);
      }
    }
  });

  (collapsed, window_width)
}
