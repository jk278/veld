//! Appearance tab component
//! 外观设置标签页

use crate::config::{AppConfig, ThemeMode};
use crate::theme::{use_theme, use_zoom_level};
use dioxus::prelude::*;

/// Appearance tab content
#[component]
pub fn AppearanceTab() -> Element {
  let mut theme_mode = use_theme();
  let mut zoom_level = use_zoom_level();
  let mut preview_zoom = use_signal(|| zoom_level());
  let mut show_tooltip = use_signal(|| false);

  rsx! {
    div {
      class: "space-y-6",
      h1 {
        class: "text-2xl font-semibold text-text-primary",
        "Appearance"
      }

      section {
        class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
        h2 {
          class: "text-lg text-text-primary mb-4",
          "Theme"
        }
        div {
          class: "flex flex-wrap gap-2 items-center",
          for (mode , label , icon) in [
              (ThemeMode::Light, "Light", "☀️"),
              (ThemeMode::Dark, "Dark", "🌙"),
              (ThemeMode::System, "System", "🖥️"),
          ]
          {
            button {
              class: if theme_mode() == mode { "px-4 py-2 rounded font-mono text-sm transition-all bg-primary text-white border border-border" } else { "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border hover:bg-bg-secondary" },
              onclick: move |_| theme_mode.set(mode),
              span {
                class: "mr-2",
                "{icon}"
              }
              "{label}"
            }
          }
        }
      }

      // Zoom section
      section {
        class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
        h2 {
          class: "text-lg text-text-primary mb-4",
          "Zoom"
        }
        div {
          class: "flex items-center",
          div {
            class: "flex-1 max-w-[280px] flex items-center gap-2",
            span {
              class: "text-xs text-text-tertiary px-2 py-0.5 border border-border rounded-sm",
              "50%"
            }
            div {
              class: "relative flex-1",
              input {
                r#type: "range",
                class: "w-full h-2 bg-border rounded-lg appearance-none cursor-pointer accent-primary",
                min: "50",
                max: "200",
                step: "10",
                value: "{(preview_zoom() * 100.0) as i64}",
                onmouseenter: move |_| show_tooltip.set(true),
                onmouseleave: move |_| show_tooltip.set(false),
                oninput: move |evt| {
                  if let Ok(val) = evt.value().parse::<f64>() {
                    preview_zoom.set(val / 100.0);
                    show_tooltip.set(true);
                  }
                },
                onchange: move |evt| {
                  if let Ok(val) = evt.value().parse::<f64>() {
                    let new_zoom = val / 100.0;
                    zoom_level.set(new_zoom);
                    preview_zoom.set(new_zoom);
                    if let Ok(mut config) = AppConfig::load() {
                      config.update_zoom_level(new_zoom);
                    }
                  }
                }
              }
              // Tooltip positioned relative to slider container
              {
                let thumb_percent = (preview_zoom() * 100.0 - 50.0) / 150.0;
                let opacity_val = if show_tooltip() { 1.0 } else { 0.0 };
                let style_str = format!("left: {:.2}%; transform: translateX(-50%); opacity: {}", thumb_percent * 100.0, opacity_val);
                rsx! {
                div {
                  class: "absolute -top-10 px-3 py-1 bg-primary text-white text-sm rounded shadow-lg pointer-events-none transition-opacity duration-150",
                  style: "{style_str}",
                  "{preview_zoom() * 100.0:.0}%"
                }
              }
              }
            }
            span {
              class: "text-xs text-text-tertiary px-2 py-0.5 border border-border rounded-sm",
              "200%"
            }
          }
        }
      }
    }
  }
}
