//! Command Palette Component - Quick prompt selection via /command
//! 命令面板组件 - 通过 /命令快速选择预设提示词

use crate::config::QuickPrompt;
use dioxus::prelude::*;

/// Command palette for quick prompt selection
/// Triggered by typing "/" in the input area
#[component]
pub fn CommandPalette(
  prompts: Vec<QuickPrompt>,
  initial_filter: String,
  visible: bool,
  on_select: Callback<QuickPrompt>,
  on_close: Callback<()>,
) -> Element {
  // Internal filter state (starts with initial_filter from textarea)
  let mut filter = use_signal(|| initial_filter.trim_start_matches('/').to_string());
  let mut selected_index = use_signal(|| 0);

  // Filter prompts based on filter text
  let filtered_prompts = use_memo(move || {
    let search = filter().to_lowercase();
    if search.is_empty() {
      return prompts.clone();
    }
    prompts
      .iter()
      .filter(|p| {
        p.keyword.to_lowercase().contains(&search) || p.name.to_lowercase().contains(&search)
      })
      .cloned()
      .collect::<Vec<_>>()
  });

  if !visible {
    return rsx! {};
  }

  let filtered = filtered_prompts();
  let has_results = !filtered.is_empty();

  // Build list items
  let list_items = filtered
    .iter()
    .enumerate()
    .map(|(idx, prompt)| {
      let is_selected = idx == selected_index();
      let prompt_clone = prompt.clone();
      rsx! {
        div {
          class: if is_selected { "command-palette-item selected" } else { "command-palette-item" },
          onclick: move |_| {
              on_select.call(prompt_clone.clone());
          },

          span {
            class: "command-palette-keyword",
            "/{prompt_clone.keyword}"
          }
          span {
            class: "command-palette-name",
            "{prompt_clone.name}"
          }
        }
      }
    })
    .collect::<Vec<_>>();

  rsx! {
    div {
      class: "command-palette-overlay",
      onclick: move |_| on_close.call(()),

      div {
        class: "command-palette",
        onclick: move |e| e.stop_propagation(),

        // Filter input (styled like a search bar)
        div {
          class: "command-palette-search",
          input {
            class: "command-palette-input",
            r#type: "text",
            value: "{filter()}",
            placeholder: "Search tools...",
            oninput: move |e| {
                filter.set(e.value());
                selected_index.set(0);
            },
            onkeydown: move |e| {
                let filtered_clone = filtered.clone();
                match e.key() {
                    Key::ArrowDown => {
                        e.stop_propagation();
                        e.prevent_default();
                        let count = filtered_clone.len();
                        if count > 0 {
                            selected_index.set((selected_index() + 1) % count);
                        }
                    }
                    Key::ArrowUp => {
                        e.stop_propagation();
                        e.prevent_default();
                        let count = filtered_clone.len();
                        if count > 0 {
                            selected_index
                                .set(
                                    if selected_index() == 0 {
                                        count - 1
                                    } else {
                                        selected_index() - 1
                                    },
                                );
                        }
                    }
                    Key::Enter => {
                        e.stop_propagation();
                        e.prevent_default();
                        if has_results {
                            let prompt = filtered_clone[selected_index()].clone();
                            on_select.call(prompt);
                        }
                    }
                    Key::Escape => {
                        e.stop_propagation();
                        e.prevent_default();
                        on_close.call(());
                    }
                    _ => {}
                }
            },
          }
        }

        div {
          class: "command-palette-header",
          span {
            class: "command-palette-title",
            "Quick Tools"
          }
          span {
            class: "command-palette-hint",
            "↑↓ navigate • Enter select • Esc close"
          }
        }

        if has_results {
          div {
            class: "command-palette-list",
            {list_items.into_iter()}
          }
        } else {
          div {
            class: "command-palette-empty",
            "No matching quick tools"
          }
        }
      }
    }
  }
}
