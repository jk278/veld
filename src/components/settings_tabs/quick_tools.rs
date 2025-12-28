//! Quick Tools tab component
//! 快速工具设置标签页

use crate::config::{AppConfig, QuickPrompt};
use dioxus::prelude::*;

/// Quick Tools tab content
#[component]
pub fn QuickToolsTab() -> Element {
  let prompts = use_signal(|| {
    AppConfig::load()
      .map(|c| c.quick_tools.prompts)
      .unwrap_or_default()
  });

  let mut editing_prompt = use_signal(|| Option::<QuickPrompt>::None);
  let mut show_add_dialog = use_signal(|| false);

  // Build prompt items
  let prompt_items = prompts()
    .iter()
    .map(|prompt| {
      let prompt_clone = prompt.clone();
      rsx! {
        PromptItem {
          prompt: prompt_clone.clone(),
          on_edit: Callback::new({
              let mut editing = editing_prompt.clone();
              let mut show = show_add_dialog.clone();
              move |_| {
                  editing.set(Some(prompt_clone.clone()));
                  show.set(true);
              }
          }),
          on_delete: Callback::new({
              let mut prompts = prompts.clone();
              let id = prompt.id.clone();
              move |_| {
                  let mut updated = prompts();
                  updated.retain(|p| p.id != id);
                  let updated_clone = updated.clone();
                  prompts.set(updated);
                  if let Ok(mut config) = AppConfig::load() {
                      config
                          .update_quick_tools(crate::config::QuickToolsConfig {
                              prompts: updated_clone,
                          });
                  }
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
        "Quick Tools"
      }

      p {
        class: "text-text-secondary mb-4",
        "Configure quick prompt presets for fast AI actions. Type / in the chat to access these tools."
      }

      section {
        class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

        div {
          class: "flex justify-end mb-4",
          button {
            class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all text-sm font-medium",
            onclick: move |_| {
                show_add_dialog.set(true);
                editing_prompt
                    .set(
                        Some(QuickPrompt {
                            id: format!("{}", uuid::Uuid::new_v4()),
                            name: "New quick action".to_string(),
                            keyword: "action".to_string(),
                            prefix: String::new(),
                            placeholder: None,
                        }),
                    );
            },
            "+ Add New Tool"
          }
        }

        div {
          class: "space-y-3",
          {prompt_items.into_iter()}
        }
      }

      // Add/Edit dialog
      if show_add_dialog() {
        PromptEditDialog {
          prompt: editing_prompt(),
          on_save: Callback::new({
              let mut prompts = prompts.clone();
              let mut show_dialog = show_add_dialog.clone();
              move |prompt: QuickPrompt| {
                  let mut updated = prompts();
                  if let Some(pos) = updated.iter().position(|p| p.id == prompt.id) {
                      updated[pos] = prompt.clone();
                  } else {
                      updated.push(prompt.clone());
                  }
                  let updated_clone = updated.clone();
                  prompts.set(updated);
                  if let Ok(mut config) = AppConfig::load() {
                      config
                          .update_quick_tools(crate::config::QuickToolsConfig {
                              prompts: updated_clone,
                          });
                  }
                  show_dialog.set(false);
                  editing_prompt.set(None);
              }
          }),
          on_cancel: Callback::new(move |_| {
              show_add_dialog.set(false);
              editing_prompt.set(None);
          }),
        }
      }
    }
  }
}

/// Single prompt item display
#[component]
fn PromptItem(prompt: QuickPrompt, on_edit: Callback<()>, on_delete: Callback<()>) -> Element {
  rsx! {
    div {
      class: "flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 bg-bg-surface border border-border rounded-md hover:border-primary transition-colors",
      div {
        class: "flex-1 min-w-0",
        div {
          class: "flex flex-wrap items-center gap-2",
          span {
            class: "font-mono text-sm font-medium text-primary",
            "/{prompt.keyword}"
          }
          span {
            class: "text-text-primary font-medium",
            "{prompt.name}"
          }
        }
      }
      div {
        class: "flex flex-wrap items-center gap-2 sm:flex-nowrap",
        button {
          class: "px-3 py-1.5 text-sm text-text-secondary hover:text-primary transition-colors",
          onclick: move |_| on_edit.call(()),
          "Edit"
        }
        button {
          class: "px-3 py-1.5 text-sm text-text-secondary hover:text-error transition-colors",
          onclick: move |_| on_delete.call(()),
          "Delete"
        }
      }
    }
  }
}

/// Dialog for adding/editing a prompt
#[component]
fn PromptEditDialog(
  prompt: Option<QuickPrompt>,
  on_save: Callback<QuickPrompt>,
  on_cancel: Callback<()>,
) -> Element {
  let mut name = use_signal(|| prompt.as_ref().map(|p| p.name.clone()).unwrap_or_default());
  let mut keyword = use_signal(|| {
    prompt
      .as_ref()
      .map(|p| p.keyword.clone())
      .unwrap_or_default()
  });
  let mut prefix = use_signal(|| {
    prompt
      .as_ref()
      .map(|p| p.prefix.clone())
      .unwrap_or_default()
  });
  let mut placeholder = use_signal(|| {
    prompt
      .as_ref()
      .and_then(|p| p.placeholder.clone())
      .unwrap_or_default()
  });

  let is_edit = prompt.is_some();
  let title = if is_edit {
    "Edit Quick Tool"
  } else {
    "Add Quick Tool"
  };

  rsx! {
    div {
      class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
      onclick: move |_| on_cancel.call(()),
      div {
        class: "bg-bg-surface border border-border rounded-lg p-6 w-full max-w-lg",
        onclick: move |e| e.stop_propagation(),

        h2 {
          class: "text-xl font-semibold text-text-primary mb-4",
          "{title}"
        }

        div {
          class: "space-y-4",
          // Command (first - it's the primary identifier)
          div {

            label {
              class: "block text-sm font-medium text-text-secondary mb-1",
              "Command"
            }
            div {
              class: "flex items-center gap-2",
              span {
                class: "text-text-secondary font-mono",
                "/"
              }
              input {
                class: "flex-1 px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none font-mono",
                r#type: "text",
                value: keyword(),
                oninput: move |e| keyword.set(e.value()),
                placeholder: "e.g., summarize",
              }
            }
          }

          // Description
          div {

            label {
              class: "block text-sm font-medium text-text-secondary mb-1",
              "Description"
            }
            input {
              class: "w-full px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none",
              r#type: "text",
              value: name(),
              oninput: move |e| name.set(e.value()),
              placeholder: "e.g., Summarize content",
            }
          }

          // Prefix (System Prompt)
          div {

            label {
              class: "block text-sm font-medium text-text-secondary mb-1",
              "System Prompt"
            }
            textarea {
              class: "w-full px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none font-mono text-sm resize-y",
              rows: 5,
              value: prefix(),
              oninput: move |e| prefix.set(e.value()),
              placeholder: "The prompt that will be prepended to user input...",
            }
            p {
              class: "text-xs text-text-secondary mt-1",
              "This will be added before the user's content when using this tool."
            }
          }

          // Input Placeholder (optional)
          div {

            label {
              class: "block text-sm font-medium text-text-secondary mb-1",
              "Input Placeholder (optional)"
            }
            input {
              class: "w-full px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none",
              r#type: "text",
              value: placeholder(),
              oninput: move |e| placeholder.set(e.value()),
              placeholder: "Auto-generated if empty",
            }
            p {
              class: "text-xs text-text-secondary mt-1",
              "Leave empty to auto-generate from description."
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
            disabled: name().trim().is_empty() || keyword().trim().is_empty(),
            onclick: move |_| {
                let placeholder_text = placeholder();
                let placeholder_val = if placeholder_text.trim().is_empty() {
                    None
                } else {
                    Some(placeholder_text)
                };
                let prompt = QuickPrompt {
                    id: prompt
                        .as_ref()
                        .map(|p| p.id.clone())
                        .unwrap_or_else(|| format!("{}", uuid::Uuid::new_v4())),
                    name: name(),
                    keyword: keyword(),
                    prefix: prefix(),
                    placeholder: placeholder_val,
                };
                on_save.call(prompt);
            },
            "Save"
          }
        }
      }
    }
  }
}
