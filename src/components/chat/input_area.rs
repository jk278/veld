//! Chat input area component with /command support
//! 聊天输入区域组件 - 支持 /命令快速工具

use crate::components::command_palette::CommandPalette;
use crate::config::{AppConfig, QuickPrompt};
use dioxus::prelude::*;

/// Enhanced InputArea with /command support
/// Command is shown as a styled block: [📝 Summarize] content here
/// Backspace deletes the entire command block
#[component]
pub fn InputArea(
  mut input_text: Signal<String>,
  has_api_key: bool,
  on_send: EventHandler<MouseEvent>,
  tx: Coroutine<String>,
  tx_with_prefix: Coroutine<(String, Option<QuickPrompt>)>,
) -> Element {
  // Command palette state
  let mut show_palette = use_signal(|| false);
  let mut palette_filter = use_signal(String::new);

  // Active command state
  let mut active_command = use_signal(|| Option::<QuickPrompt>::None);
  let mut user_content = use_signal(String::new);
  let mut textarea_rows = use_signal(|| 1u32);

  // Load quick prompts from config
  let quick_prompts = use_signal(|| {
    AppConfig::load()
      .map(|c| c.quick_tools.prompts)
      .unwrap_or_default()
  });

  // Current placeholder text
  let placeholder = use_memo(move || {
    if let Some(ref cmd) = active_command() {
      cmd.get_placeholder()
    } else if !has_api_key {
      "Configure API key first...".to_string()
    } else {
      "Type ... ( / for commands )".to_string()
    }
  });

  // Sync input_text with our internal state
  let mut update_input_text = move || {
    if let Some(ref cmd) = active_command() {
      input_text.set(format!("/{} {}", cmd.keyword, user_content()));
    } else {
      input_text.set(user_content());
    }
  };

  // Handle input changes (only for user content, command is separate)
  let handle_input = move |e: FormEvent| {
    let value = e.value();
    user_content.set(value.clone());

    // Auto-resize textarea based on content (1-6 rows)
    // Count newlines to handle empty trailing lines
    let newline_count = value.matches('\n').count();
    let new_rows = (newline_count + 1).clamp(1, 6) as u32;
    textarea_rows.set(new_rows);

    update_input_text();

    // Hide palette if we have active command
    if active_command().is_some() {
      show_palette.set(false);
      return;
    }

    // Check if we should show command palette
    let trimmed = value.trim_start();
    if trimmed.starts_with('/') {
      let after_slash = &trimmed[1..];
      let keyword = after_slash.split_whitespace().next().unwrap_or(after_slash);

      if after_slash.is_empty() || (!after_slash.contains(' ') && !after_slash.contains('\n')) {
        show_palette.set(true);
        palette_filter.set(format!("/{}", keyword));
      } else {
        show_palette.set(false);
      }
    } else {
      show_palette.set(false);
    }
  };

  // Auto-focus command palette input when it opens
  use_effect(move || {
    let _ = show_palette();
    if show_palette() {
      let _ = dioxus::document::eval(
        r#"
          setTimeout(() => {
            const input = document.querySelector('.command-palette-input');
            if (input && document.activeElement !== input) {
              input.focus();
              input.setSelectionRange(input.value.length, input.value.length);
            }
          }, 50);
        "#,
      );
    }
  });

  // Handle prompt selection from palette
  let on_prompt_select = move |prompt: QuickPrompt| {
    show_palette.set(false);
    active_command.set(Some(prompt.clone()));
    user_content.set(String::new());
    update_input_text();

    // Focus back to content input
    let _ = dioxus::document::eval(
      r#"
        setTimeout(() => {
          const input = document.querySelector('.user-content-input');
          if (input) input.focus();
        }, 10);
      "#,
    );
  };

  // Close palette without selection
  let on_palette_close = move |_| {
    show_palette.set(false);
    let text = user_content();
    if text.trim() == "/" {
      user_content.set(String::new());
      update_input_text();
    }

    // Focus back to input
    let _ = dioxus::document::eval(
      r#"
        setTimeout(() => {
          const input = document.querySelector('.user-content-input');
          if (input) input.focus();
        }, 10);
      "#,
    );
  };

  // Clear active command
  let mut clear_command = move || {
    active_command.set(None);
    update_input_text();

    let _ = dioxus::document::eval(
      r#"
        setTimeout(() => {
          const input = document.querySelector('.user-content-input');
          if (input) input.focus();
        }, 10);
      "#,
    );
  };

  // Parse and send message
  let mut parse_and_send = move || {
    let content = user_content().trim().to_string();
    input_text.set(String::new());

    if let Some(cmd) = active_command() {
      if content.is_empty() {
        return;
      }
      active_command.set(None);
      user_content.set(String::new());
      tx_with_prefix.send((content, Some(cmd)));
    } else {
      if content.is_empty() {
        return;
      }
      user_content.set(String::new());
      tx.send(content);
    }

    // Focus back to input after sending
    let _ = dioxus::document::eval(
      r#"
        setTimeout(() => {
          const input = document.querySelector('.user-content-input');
          if (input) input.focus();
        }, 10);
      "#,
    );
  };

  let send_click = move |_: MouseEvent| {
    parse_and_send();
  };

  // Handle keyboard events
  let handle_keydown = move |e: KeyboardEvent| {
    // Escape: close palette or clear command
    if matches!(e.key(), Key::Escape) {
      if show_palette() {
        show_palette.set(false);
        let text = user_content();
        if text.trim() == "/" {
          user_content.set(String::new());
          update_input_text();
        }
        e.stop_propagation();
      } else if active_command().is_some() {
        clear_command();
        e.stop_propagation();
      }
    }

    // Backspace: if at start of content and has active command, clear command
    if matches!(e.key(), Key::Backspace) && active_command().is_some() {
      let content = user_content();
      if content.is_empty() {
        clear_command();
        e.prevent_default();
        e.stop_propagation();
      }
    }

    // Shift+Enter: new line, Enter: send message
    if e.key() == Key::Enter && has_api_key {
      if e.modifiers().contains(dioxus::prelude::Modifiers::SHIFT) {
        // Allow default behavior (new line)
      } else {
        e.prevent_default();
        parse_and_send();
      }
    }
  };

  rsx! {
    div {
      class: "px-4 py-3 border-t border-border relative z-10 shadow-custom",
      div {
        class: "flex gap-2 items-center",

        // Combined input container (command chip + textarea inside)
        div {
          class: "flex-1 flex items-center gap-2 px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg focus-within:border-primary focus-within:ring-2 focus-within:ring-primary/20 transition-all",

          // Active command chip (inside input)
          if let Some(ref cmd) = active_command() {
            div {
              class: "flex items-center gap-2 px-2 py-1 bg-primary/10 border border-primary/30 rounded-md text-sm font-medium text-primary whitespace-nowrap",
              span {
                class: "text-sm font-mono",
                "/{cmd.keyword}"
              }
              button {
                class: "ml-1 text-text-secondary hover:text-text-primary transition-colors text-sm",
                onclick: move |_| clear_command(),
                "×"
              }
            }
          }

          // User content input
          textarea {
            class: "user-content-input chat-input-textarea flex-1 min-w-0 bg-transparent border-none resize-none outline-none font-mono text-sm",
            rows: textarea_rows(),
            placeholder: placeholder(),
            value: user_content(),
            disabled: !has_api_key,
            oninput: handle_input,
            onkeydown: handle_keydown,
          }
        }

        button {
          class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed text-sm font-medium whitespace-nowrap",
          disabled: !has_api_key || user_content().trim().is_empty(),
          onclick: send_click,
          "Send"
        }
      }
    }

    // Command palette overlay
    CommandPalette {
      prompts: quick_prompts(),
      initial_filter: palette_filter(),
      visible: show_palette(),
      on_select: Callback::new(on_prompt_select),
      on_close: Callback::new(on_palette_close),
    }
  }
}

/// Original InputArea for backward compatibility (deprecated)
/// Use the enhanced InputArea above
#[component]
pub fn InputAreaLegacy(
  input_text: Signal<String>,
  has_api_key: bool,
  on_send: EventHandler<MouseEvent>,
  tx: Coroutine<String>,
) -> Element {
  rsx! {
    div {
      class: "px-4 py-3 border-t border-border relative z-10 shadow-custom",
      div {
        class: "flex gap-2",

        textarea {
          class: "flex-1 px-3 py-2 bg-bg-primary text-text-primary border border-border rounded-lg resize-none focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all font-mono text-sm",
          rows: 1,
          placeholder: if !has_api_key { "Configure API key first..." } else { "Type your message..." },
          value: input_text(),
          disabled: !has_api_key,
          oninput: move |e| input_text.set(e.value()),
          onkeydown: move |e| {
              if e.key() == Key::Enter && has_api_key {
                  e.prevent_default();
                  let text = input_text().trim().to_string();
                  if !text.is_empty() {
                      input_text.set(String::new());
                      tx.send(text);
                  }
              }
          },
        }

        button {
          class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-all disabled:opacity-50 disabled:cursor-not-allowed text-sm font-medium",
          disabled: !has_api_key || input_text().trim().is_empty(),
          onclick: on_send,
          "Send"
        }
      }
    }
  }
}

/// Chat input wrapper with proper signal handling
#[component]
pub fn ChatInput(
  input_text: Signal<String>,
  has_api_key: bool,
  on_send: EventHandler<MouseEvent>,
  tx: Coroutine<String>,
  tx_with_prefix: Coroutine<(String, Option<QuickPrompt>)>,
) -> Element {
  rsx! {
    InputArea {
      input_text: input_text.clone(),
      has_api_key,
      on_send,
      tx,
      tx_with_prefix,
    }
  }
}
