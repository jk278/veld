//! Shortcuts tab component
//! 快捷键设置标签页

use crate::config::{AppConfig, ShortcutConfig};
use dioxus::prelude::*;

/// Shortcut item structure
#[derive(Debug, Clone, PartialEq)]
pub struct ShortcutItem {
  pub id: String,
  pub action: String,
  pub description: String,
  pub config_key: ConfigKey,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigKey {
  CommandPalette,
  // Reserved for future shortcuts
  // QuickSummarize,
  // QuickTranslate,
  // QuickExplain,
}

/// Shortcuts tab content
#[component]
pub fn ShortcutsTab() -> Element {
  let shortcuts = use_signal(|| {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    vec![ShortcutItem {
      id: "command_palette".to_string(),
      action: config
        .shortcuts
        .command_palette
        .clone()
        .unwrap_or_else(|| "Ctrl+Shift+Space".to_string()),
      description: "Open command palette".to_string(),
      config_key: ConfigKey::CommandPalette,
    }]
  });

  let mut editing_shortcut = use_signal(|| Option::<(usize, ShortcutItem)>::None);
  let mut show_edit_dialog = use_signal(|| false);

  // Build shortcut items
  let shortcut_items = shortcuts()
    .iter()
    .enumerate()
    .map(|(idx, shortcut)| {
      let shortcut_clone = shortcut.clone();
      let idx_clone = idx;
      rsx! {
        ShortcutDisplayItem {
          shortcut: shortcut_clone.clone(),
          on_edit: Callback::new({
              let mut editing = editing_shortcut.clone();
              let mut show = show_edit_dialog.clone();
              move |_| {
                  editing.set(Some((idx_clone, shortcut_clone.clone())));
                  show.set(true);
              }
          }),
        }
      }
    })
    .collect::<Vec<_>>();

  rsx! {
    div {
      class: "space-y-6",
      h1 {
        class: "text-2xl font-semibold text-text-primary",
        "Keyboard Shortcuts"
      }

      p {
        class: "text-text-secondary mb-4",
        "Configure global keyboard shortcuts. Changes take effect after restart."
      }

      section {
        class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
        h2 {
          class: "text-lg text-text-primary mb-4",
          "Global Shortcuts"
        }
        div {
          class: "space-y-3",
          {shortcut_items.into_iter()}
        }
      }

      // Edit dialog
      if show_edit_dialog() {
        ShortcutEditDialog {
          shortcut: editing_shortcut(),
          on_save: Callback::new({
              let mut shortcuts = shortcuts.clone();
              let mut show_dialog = show_edit_dialog.clone();
              move |(idx, shortcut): (usize, ShortcutItem)| {
                  let mut updated = shortcuts();
                  updated[idx] = shortcut.clone();
                  let updated_clone = updated.clone();
                  shortcuts.set(updated);
                  if let Ok(mut config) = AppConfig::load() {
                      let shortcut_config = ShortcutConfig {
                          command_palette: updated_clone
                              .iter()
                              .find(|s| s.config_key == ConfigKey::CommandPalette)
                              .map(|s| s.action.clone()),
                      };
                      config.update_shortcuts(shortcut_config);
                  }
                  show_dialog.set(false);
                  editing_shortcut.set(None);
              }
          }),
          on_cancel: Callback::new(move |_| {
              show_edit_dialog.set(false);
              editing_shortcut.set(None);
          }),
        }
      }
    }
  }
}

/// Single shortcut display item
#[component]
fn ShortcutDisplayItem(shortcut: ShortcutItem, on_edit: Callback<()>) -> Element {
  rsx! {
    div {
      class: "flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 bg-bg-surface border border-border rounded-md hover:border-primary transition-colors",
      div {
        class: "flex-1 min-w-0",
        span {
          class: "text-text-primary font-medium",
          "{shortcut.description}"
        }
      }
      div {
        class: "flex flex-wrap items-center gap-2 sm:flex-nowrap",
        code {
          class: "px-3 py-1.5 bg-bg-surface text-primary rounded font-mono text-sm border border-border",
          "{shortcut.action}"
        }
        button {
          class: "px-3 py-1.5 text-sm text-text-secondary hover:text-primary transition-colors",
          onclick: move |_| on_edit.call(()),
          "Edit"
        }
      }
    }
  }
}

/// Dialog for editing a shortcut
#[component]
fn ShortcutEditDialog(
  shortcut: Option<(usize, ShortcutItem)>,
  on_save: Callback<(usize, ShortcutItem)>,
  on_cancel: Callback<()>,
) -> Element {
  let mut action = use_signal(|| {
    shortcut
      .as_ref()
      .map(|s| s.1.action.clone())
      .unwrap_or_default()
  });

  let (idx, original) = shortcut.unwrap();

  rsx! {
    div {
      class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
      onclick: move |_| on_cancel.call(()),
      div {
        class: "bg-bg-surface border border-border rounded-lg p-6 w-full max-w-md",
        onclick: move |e| e.stop_propagation(),

        h2 {
          class: "text-xl font-semibold text-text-primary mb-4",
          "Edit Shortcut"
        }

        div {
          class: "space-y-4",
          div {

            label {
              class: "block text-sm font-medium text-text-secondary mb-1",
              "Action"
            }
            input {
              class: "w-full px-3 py-2 bg-bg-primary text-text-muted border border-border rounded-lg cursor-not-allowed",
              r#type: "text",
              value: original.description.clone(),
              readonly: true,
            }
          }

          div {

            label {
              class: "block text-sm font-medium text-text-secondary mb-1",
              "Keyboard Shortcut"
            }
            input {
              class: "w-full px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none font-mono",
              r#type: "text",
              value: action(),
              oninput: move |e| action.set(e.value()),
              placeholder: "e.g., Ctrl+Shift+Space",
            }
            p {
              class: "text-xs text-text-secondary mt-1",
              "Format: Ctrl+Shift+Key, Alt+Key, etc."
            }
          }
        }

        div {
          class: "flex justify-end gap-3 mt-6",
          button {
            class: "px-4 py-2 text-text-secondary hover:text-text-primary transition-all",
            onclick: move |_| on_cancel.call(()),
            "Cancel"
          }
          button {
            class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed",
            disabled: action().trim().is_empty(),
            onclick: move |_| {
                let updated = ShortcutItem {
                    action: action(),
                    ..original.clone()
                };
                on_save.call((idx, updated));
            },
            "Save"
          }
        }
      }
    }
  }
}
