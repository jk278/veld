//! Drawer sidebar component
//! 抽屉式侧边栏组件 - 响应式导航/会话列表

use crate::hooks::use_sidebar_resize;
use dioxus::prelude::*;

/// Drawer sidebar with responsive behavior and drag-to-resize
/// 抽屉式侧边栏 - 窄屏覆盖/宽屏内联，拖拽调整宽度
#[component]
pub fn DrawerSidebar(
  collapsed: bool,
  on_close: EventHandler<MouseEvent>,
  children: Element,
) -> Element {
  let (_is_resizing, start_resize) = use_sidebar_resize();

  rsx! {
    // Overlay backdrop (click to close, narrow screen only)
    div {
      class: if collapsed { "drawer-overlay" } else { "drawer-overlay visible" },
      style: "opacity: 0;",
      onclick: on_close,
    }

    // Drawer sidebar (responsive: overlay on narrow, inline on wide)
    div {
      class: {
          let base = "drawer-sidebar";
          if collapsed { base.to_string() } else { format!("{base} visible") }
      },
      style: "transform: translateX(-100%);",

      {children}

      // Drag handle (desktop only, visible when expanded)
      if !collapsed {
        div {
          class: "sidebar-drag-handle",
          onmousedown: move |e: MouseEvent| {
            e.stop_propagation();
            start_resize(());
          },
        }
      }
    }
  }
}
