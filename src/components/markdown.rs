//! Markdown rendering component for chat messages
//! Markdown 渲染组件

use dioxus::prelude::*;
use pulldown_cmark::{Parser, html, Options};

/// Parse markdown text to HTML string with extensions enabled
pub fn markdown_to_html(markdown: &str) -> String {
    // Enable CommonMark + GFM extensions (tables, strikethrough, tasklists)
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Markdown renderer component
/// Renders markdown content as HTML with proper styling
#[component]
pub fn MarkdownContent(
    /// Markdown text content
    content: String,
    /// Additional CSS classes
    #[props(default)]
    class: String,
) -> Element {
    let html_content = markdown_to_html(&content);

    rsx! {
        div {
            class: format!("markdown-body {}", class),
            dangerous_inner_html: "{html_content}"
        }
    }
}

/// Plain text renderer (for user messages that don't need markdown)
#[component]
pub fn PlainTextContent(
    /// Plain text content
    content: String,
    /// Additional CSS classes
    #[props(default)]
    class: String,
) -> Element {
    rsx! {
        p {
            class: format!("whitespace-pre-wrap break-words {}", class),
            {content}
        }
    }
}
