//! Tool call parser
//! 工具调用解析器

use super::types::{AgentError, Result, ToolCall};
use serde_json::Value;

/// Parse tool call from AI response
pub fn parse_tool_call(response: &str) -> Result<ToolCall> {
  let response_trimmed = response.trim();

  // Remove markdown code blocks
  let cleaned = if response_trimmed.starts_with("```") {
    let lines: Vec<&str> = response_trimmed.lines().collect();
    let start_idx = if lines[0].contains("json") { 1 } else { 1 };
    let end_idx = lines
      .iter()
      .rposition(|l| *l == "```")
      .unwrap_or(lines.len());
    lines[start_idx..end_idx].join("\n")
  } else {
    response_trimmed.to_string()
  };

  eprintln!(
    "[MCP] Parsing tool call from (first 300 chars): {}...",
    &cleaned.chars().take(300).collect::<String>()
  );

  // Strategy 1: Try direct parse (entire response is JSON)
  if let Ok(v) = serde_json::from_str::<Value>(&cleaned) {
    eprintln!(
      "[MCP] Strategy 1: Direct JSON parse successful, has tool_call key: {}",
      v.get("tool_call").is_some()
    );
    if let Some(tc) = v.get("tool_call") {
      eprintln!("[MCP] Strategy 1: Found tool_call: {}", tc);
      return Ok(
        serde_json::from_value(tc.clone()).map_err(|e| AgentError::ToolParse(e.to_string()))?,
      );
    }
  } else {
    eprintln!("[MCP] Strategy 1: Not valid JSON, trying extraction...");
  }

  // Strategy 2 & 3: Extract JSON from text (when JSON is embedded in response)
  // Find tool_call pattern and extract the value object
  if cleaned.contains("\"tool_call\"") {
    if let Some(start) = cleaned.find("\"tool_call\"") {
      // Find the colon after "tool_call"
      let after_key = &cleaned[start + "\"tool_call\"".len()..];
      if let Some(colon_pos) = after_key.find(':') {
        // Skip colon and any whitespace/braces until we find the value object
        let after_colon = &after_key[colon_pos + 1..];
        let trimmed = after_colon.trim_start();

        if trimmed.starts_with('{') {
          // Found the value object - now find matching closing brace
          let from_brace = &trimmed[1..]; // Skip opening {
          let mut brace_count = 1;
          let mut in_string = false;
          let mut end_idx = 0;

          for (i, c) in from_brace.chars().enumerate() {
            match c {
              '"' if !in_string => in_string = true,
              '"' if in_string => in_string = false,
              '{' if !in_string => brace_count += 1,
              '}' if !in_string => {
                brace_count -= 1;
                if brace_count == 0 {
                  end_idx = i + 1;
                  break;
                }
              }
              _ => {}
            }
          }

          if end_idx > 0 {
            let tool_call_json = &from_brace[..end_idx];
            eprintln!("[MCP] Extracted tool_call JSON: {}", tool_call_json);

            if let Ok(tc) = serde_json::from_str::<Value>(tool_call_json) {
              eprintln!("[MCP] Found tool_call: {}", tc);
              return Ok(
                serde_json::from_value(tc.clone())
                  .map_err(|e| AgentError::ToolParse(e.to_string()))?,
              );
            }
          }
        }
      }
    }
  }

  eprintln!("[MCP] No tool_call found in response");
  Err(AgentError::ToolParse("No tool call found".to_string()))
}
