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
