//! Window size hook for responsive behavior
//! 窗口尺寸 Hook - 用于响应式布局

use dioxus::prelude::*;

/// Responsive breakpoint: narrow screen threshold in pixels
/// 响应式断点：窄屏阈值（像素）
pub const RESPONSIVE_BREAKPOINT: f64 = 768.0;

/// Window size state - tracks logical window width
pub fn use_window_size() -> Signal<f64> {
  let window_width = use_signal(|| RESPONSIVE_BREAKPOINT);
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
