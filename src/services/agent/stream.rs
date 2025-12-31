//! Stream handler
//! 流式响应处理器

use super::types::{AgentError, AgentStep, Result};
use crate::services::ai_client::{AiClient, ChatMessage};
use tokio::sync::mpsc;

/// Stream chat response and send chunks via AgentStep
pub async fn stream_chat_response(
  client: AiClient,
  messages: Vec<ChatMessage>,
  tx: mpsc::UnboundedSender<AgentStep>,
  mut abort_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<String> {
  let mut rx = client.chat(messages).await.map_err(|e| AgentError::Ai(e.to_string()))?;
  let mut full_response = String::new();

  loop {
    tokio::select! {
      _ = abort_rx.recv() => {
        eprintln!("[MCP] Stream aborted");
        let _ = tx.send(AgentStep::Final);
        return Ok(String::new());
      }
      chunk_result = rx.recv() => {
        match chunk_result {
          Some(Ok(chunk)) => {
            full_response.push_str(&chunk);
            let _ = tx.send(AgentStep::Chunk(chunk));
          }
          Some(Err(e)) => {
            eprintln!("[MCP] Stream error: {}", e);
            let _ = tx.send(AgentStep::Final);
            return Err(AgentError::Ai(e.to_string()));
          }
          None => {
            let _ = tx.send(AgentStep::Final);
            return Ok(full_response);
          }
        }
      }
    }
  }
}
