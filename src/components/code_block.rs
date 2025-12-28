//! Code block rendering component with syntax highlighting
//! 代码块渲染组件（支持语法高亮、复制、滚动）

use dioxus::prelude::*;
use std::sync::OnceLock;
use syntect::{
  highlighting::{Theme, ThemeSet},
  html::{highlighted_html_for_string, ClassStyle, ClassedHTMLGenerator},
  parsing::SyntaxSet,
};

// Use dioxus's spawn for async tasks in the UI context
use dioxus::prelude::spawn;

// NOTE: Using OnceLock for lazy initialization (Rust 1.70+)
// These are loaded once and reused for all code blocks

/// Global syntax set (lazy-loaded)
/// Sublime Text syntax definitions for 100+ languages
fn syntax_set() -> &'static SyntaxSet {
  static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
  SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
}

/// Global theme set (lazy-loaded)
fn theme_set() -> &'static ThemeSet {
  static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
  THEME_SET.get_or_init(|| ThemeSet::load_defaults())
}

/// Get the default dark theme for code highlighting
fn dark_theme() -> &'static Theme {
  &theme_set().themes["base16-ocean.dark"]
}

/// Get the default light theme for code highlighting
fn light_theme() -> &'static Theme {
  &theme_set().themes["base16-ocean.light"]
}

/// Language display names mapping
/// Maps file extensions/language identifiers to human-readable names
fn language_display_name(lang: &str) -> &str {
  match lang.to_lowercase().as_str() {
    "rs" | "rust" => "Rust",
    "py" | "python" | "python3" => "Python",
    "js" | "javascript" => "JavaScript",
    "ts" | "typescript" => "TypeScript",
    "jsx" => "JSX",
    "tsx" => "TSX",
    "c" | "h" => "C",
    "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "c++" => "C++",
    "cs" => "C#",
    "java" => "Java",
    "go" => "Go",
    "rb" | "ruby" => "Ruby",
    "php" => "PHP",
    "swift" => "Swift",
    "kt" | "kotlin" => "Kotlin",
    "scala" => "Scala",
    "sh" | "bash" | "shell" => "Shell",
    "zsh" => "Zsh",
    "fish" => "Fish",
    "ps1" | "powershell" => "PowerShell",
    "json" => "JSON",
    "yaml" | "yml" => "YAML",
    "toml" => "TOML",
    "xml" => "XML",
    "html" | "htm" => "HTML",
    "css" => "CSS",
    "scss" | "sass" => "SCSS",
    "less" => "Less",
    "md" | "markdown" => "Markdown",
    "sql" => "SQL",
    "dockerfile" => "Dockerfile",
    "makefile" => "Makefile",
    "cmake" => "CMake",
    "tex" => "LaTeX",
    "r" => "R",
    "lua" => "Lua",
    "dart" => "Dart",
    "ex" | "exs" | "elixir" => "Elixir",
    "erl" | "hrl" | "erlang" => "Erlang",
    "clj" | "cljs" | "clojure" => "Clojure",
    "fs" | "fsharp" => "F#",
    "vb" => "VB.NET",
    "asm" => "Assembly",
    "nim" => "Nim",
    "julia" => "Julia",
    "matlab" => "MATLAB",
    "perl" => "Perl",
    "raku" => "Raku",
    "vue" => "Vue",
    "svelte" => "Svelte",
    _ => lang,
  }
}

/// Highlight code to HTML with inline styles
/// Returns HTML string with syntax highlighting applied
pub fn highlight_code_inline(code: &str, lang: &str, dark: bool) -> String {
  let syntax = syntax_set()
    .find_syntax_by_token(lang)
    .or_else(|| syntax_set().find_syntax_by_extension(lang))
    .or_else(|| syntax_set().find_syntax_by_name(lang))
    .unwrap_or_else(|| syntax_set().find_syntax_plain_text());

  let theme = if dark { dark_theme() } else { light_theme() };
  highlighted_html_for_string(code, syntax_set(), syntax, theme)
    .unwrap_or_else(|_| html_escape(code))
}

/// Highlight code to HTML with CSS classes
/// Returns HTML string with class names for external styling
pub fn highlight_code_classed(code: &str, lang: &str) -> String {
  let syntax = syntax_set()
    .find_syntax_by_token(lang)
    .or_else(|| syntax_set().find_syntax_by_extension(lang))
    .or_else(|| syntax_set().find_syntax_by_name(lang))
    .unwrap_or_else(|| syntax_set().find_syntax_plain_text());

  let mut html_generator =
    ClassedHTMLGenerator::new_with_class_style(syntax, syntax_set(), ClassStyle::Spaced);

  for line in syntect::util::LinesWithEndings::from(code) {
    let _ = html_generator.parse_html_for_line_which_includes_newline(line);
  }

  html_generator.finalize()
}

/// Simple HTML escape for fallback
fn html_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

/// Code block component with syntax highlighting, copy button, and horizontal scroll
#[component]
pub fn CodeBlock(
  /// Code content
  code: String,
  /// Programming language (file extension or name)
  #[props(default)]
  language: String,
  /// Whether to use dark theme
  #[props(default)]
  dark: bool,
) -> Element {
  let mut copy_status = use_signal(|| false);
  let lang_display = language_display_name(&language);

  // Generate highlighted HTML on first render
  let highlighted_html = highlight_code_inline(&code, &language, dark);

  let copy_code = {
    let code = code.clone();
    move |_| {
      if arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(&code))
        .is_ok()
      {
        copy_status.set(true);
        // Reset status after 2 seconds
        let mut copy_status = copy_status.clone();
        spawn(async move {
          tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
          copy_status.set(false);
        });
      }
    }
  };

  rsx! {
    div {
      class: format!("code-block-container {}", if dark { "dark" } else { "light" }),

      // Header: language label + copy button
      div {
        class: "code-block-header",
        span {
          class: "code-block-language",
          "{lang_display}"
        }
        button {
          class: format!("code-block-copy {}", if copy_status() { "copied" } else { "" }),
          onclick: copy_code,
          aria_label: "Copy code",
          disabled: copy_status(),
          if copy_status() {
            svg {
              fill: "none",
              stroke: "currentColor",
              view_box: "0 0 24 24",
              "stroke-width": "2",
              width: "16",
              height: "16",
              path {
                d: "M5 13l4 4L19 7",
              }
            }
          } else {
            svg {
              fill: "none",
              stroke: "currentColor",
              view_box: "0 0 24 24",
              "stroke-width": "2",
              width: "16",
              height: "16",
              path {
                d: "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z",
              }
            }
          }
        }
      }

      // Scrollable code area
      div {
        class: "code-block-content",
        // Render highlighted HTML
        div {
          class: "code-block-inner",
          dangerous_inner_html: "{highlighted_html}",
        }
      }
    }
  }
}

/// Inline code component (for `code` spans)
#[component]
pub fn InlineCode(
  /// Code content
  code: String,
  /// Programming language (optional, for syntax hint)
  #[props(default)]
  language: String,
) -> Element {
  rsx! {
    code {
      class: "inline-code",
      {code}
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_language_display_name() {
    assert_eq!(language_display_name("rs"), "Rust");
    assert_eq!(language_display_name("python"), "Python");
    assert_eq!(language_display_name("unknown"), "unknown");
  }

  #[test]
  fn test_html_escape() {
    assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    assert_eq!(html_escape("a & b"), "a &amp; b");
  }
}
