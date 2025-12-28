//! Markdown rendering component for chat messages
//! Markdown 渲染组件（支持代码块语法高亮）

use crate::components::code_block::CodeBlock;
use dioxus::prelude::*;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// Represents a segment of parsed markdown content
#[derive(Clone, Debug)]
pub enum MarkdownSegment {
  /// Code block with language and content
  CodeBlock { language: String, code: String },
  /// HTML content (everything else)
  Html(String),
}

/// Parse markdown into segments, separating code blocks from other content
pub fn parse_markdown_segments(markdown: &str) -> Vec<MarkdownSegment> {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_TABLES);
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TASKLISTS);

  let parser = Parser::new_ext(markdown, options);
  let mut segments = Vec::new();
  let mut current_html = String::new();
  let mut in_code_block = false;
  let mut code_language = String::new();
  let mut code_content = String::new();

  for event in parser {
    match event {
      Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
        // Flush accumulated HTML before code block
        if !current_html.is_empty() {
          segments.push(MarkdownSegment::Html(current_html.clone()));
          current_html.clear();
        }

        // Extract language from info (first word)
        code_language = info.split_whitespace().next().unwrap_or("").to_string();
        code_content.clear();
        in_code_block = true;
      }
      Event::End(TagEnd::CodeBlock) => {
        if in_code_block {
          segments.push(MarkdownSegment::CodeBlock {
            language: code_language.clone(),
            code: code_content.clone(),
          });
          in_code_block = false;
          code_content.clear();
        }
      }
      Event::Text(text) | Event::Code(text) => {
        if in_code_block {
          code_content.push_str(&text);
        } else {
          // Escape HTML for inline text
          current_html.push_str(&html_escape::encode_text(&text));
        }
      }
      Event::SoftBreak | Event::HardBreak => {
        if in_code_block {
          code_content.push('\n');
        } else {
          current_html.push_str("<br>");
        }
      }
      Event::Html(html) | Event::InlineHtml(html) => {
        if !in_code_block {
          current_html.push_str(&html);
        }
      }
      Event::Start(tag) => {
        if !in_code_block {
          current_html.push_str(&start_tag_to_html(&tag));
        }
      }
      Event::End(tag_end) => {
        if !in_code_block {
          current_html.push_str(&end_tag_to_html(&tag_end));
        }
      }
      _ => {}
    }
  }

  // Flush remaining HTML
  if !current_html.is_empty() {
    segments.push(MarkdownSegment::Html(current_html));
  }

  segments
}

/// Convert pulldown_cmark start tag to HTML
fn start_tag_to_html(tag: &Tag) -> String {
  match tag {
    Tag::Paragraph => "<p>".to_string(),
    Tag::Heading { level, .. } => format!("<h{}>", level),
    Tag::BlockQuote(_) => "<blockquote>".to_string(),
    Tag::CodeBlock(_) => "<pre><code>".to_string(),
    Tag::List(_) => "<ul>".to_string(),
    Tag::Item => "<li>".to_string(),
    Tag::Table(_) => "<table>".to_string(),
    Tag::TableHead => "<thead>".to_string(),
    Tag::TableRow => "<tr>".to_string(),
    Tag::TableCell => "<td>".to_string(),
    Tag::Emphasis => "<em>".to_string(),
    Tag::Strong => "<strong>".to_string(),
    Tag::Strikethrough => "<s>".to_string(),
    Tag::Link {
      dest_url, title, ..
    } => {
      format!("<a href=\"{}\" title=\"{}\">", dest_url, title)
    }
    Tag::Image {
      dest_url, title, ..
    } => {
      format!("<img src=\"{}\" alt=\"{}\" />", dest_url, title)
    }
    _ => "".to_string(),
  }
}

/// Convert pulldown_cmark end tag to HTML
fn end_tag_to_html(tag_end: &TagEnd) -> String {
  match tag_end {
    TagEnd::Paragraph => "</p>".to_string(),
    TagEnd::Heading(_) => "</h>".to_string(),
    TagEnd::BlockQuote(_) => "</blockquote>".to_string(),
    TagEnd::CodeBlock => "</code></pre>".to_string(),
    TagEnd::List(_) => "</ul>".to_string(),
    TagEnd::Item => "</li>".to_string(),
    TagEnd::Table => "</table>".to_string(),
    TagEnd::TableHead => "</thead>".to_string(),
    TagEnd::TableRow => "</tr>".to_string(),
    TagEnd::TableCell => "</td>".to_string(),
    TagEnd::Emphasis => "</em>".to_string(),
    TagEnd::Strong => "</strong>".to_string(),
    TagEnd::Strikethrough => "</s>".to_string(),
    TagEnd::Link => "</a>".to_string(),
    TagEnd::Image => "".to_string(),
    _ => "".to_string(),
  }
}

/// Parse markdown text to HTML string with extensions enabled
/// (Legacy function for simple markdown without code block highlighting)
pub fn markdown_to_html(markdown: &str) -> String {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_TABLES);
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TASKLISTS);

  let parser = Parser::new_ext(markdown, options);
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, parser);
  html_output
}

/// Render a single markdown segment to Element
fn render_segment(segment: &MarkdownSegment, dark: bool) -> Element {
  match segment {
    MarkdownSegment::CodeBlock { language, code } => {
      rsx! {
        CodeBlock {
          code: code.clone(),
          language: language.clone(),
          dark,
        }
      }
    }
    MarkdownSegment::Html(html) => {
      rsx! {
        div {
          dangerous_inner_html: "{html}",
        }
      }
    }
  }
}

/// Enhanced Markdown renderer component with code block highlighting
/// Renders markdown content with syntax-highlighted code blocks
#[component]
pub fn MarkdownContent(
  /// Markdown text content
  content: String,
  /// Additional CSS classes
  #[props(default)]
  class: String,
  /// Whether to use dark theme for code blocks
  #[props(default)]
  dark: bool,
) -> Element {
  let segments = parse_markdown_segments(&content);

  rsx! {
    div {
      class: format!("markdown-body {}", class),
      for segment in segments.iter() {
        {render_segment(segment, dark)}
      }
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

// Module for HTML escaping
mod html_escape {
  pub fn encode_text(s: &str) -> String {
    s.replace('&', "&amp;")
      .replace('<', "&lt;")
      .replace('>', "&gt;")
  }
}
