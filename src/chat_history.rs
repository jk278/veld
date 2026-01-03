//! Chat History Management
//! 聊天历史记录管理 - 完整的会话历史功能

use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Chat message in history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
  pub id: String,
  pub role: String,
  pub content: String,
  pub timestamp: u64,
}

/// Chat session (a conversation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
  pub id: String,
  pub title: String,
  pub provider_id: String,
  pub messages: Vec<ChatMessage>,
  pub created_at: u64,
  pub updated_at: u64,
}

/// All chat history data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatHistoryData {
  pub sessions: Vec<Session>,
  pub current_session_id: Option<String>,
}

impl Default for ChatHistoryData {
  fn default() -> Self {
    Self {
      sessions: Vec::new(),
      current_session_id: None,
    }
  }
}

/// Result type for chat history operations
pub type Result<T> = std::result::Result<T, ChatHistoryError>;

/// Chat history error type
#[derive(Debug, thiserror::Error)]
pub enum ChatHistoryError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
  #[error("JSON serialization error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("Configuration directory not found")]
  ConfigDirNotFound,
}

impl ChatHistoryData {
  /// Get the history file path
  fn get_history_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().ok_or(ChatHistoryError::ConfigDirNotFound)?;
    path.push("veld");
    fs::create_dir_all(&path)?;
    path.push("chat_history.json");
    Ok(path)
  }

  /// Load all chat history
  pub fn load() -> Result<Self> {
    let path = Self::get_history_path()?;

    if !path.exists() {
      return Ok(Self::default());
    }

    let content = fs::read_to_string(&path)?;
    let history: ChatHistoryData = serde_json::from_str(&content)?;
    Ok(history)
  }

  /// Save all chat history
  pub fn save(&self) -> Result<()> {
    let path = Self::get_history_path()?;
    let json = serde_json::to_string_pretty(self)?;
    fs::write(&path, json)?;
    Ok(())
  }

  /// Create a new session
  pub fn new_session(provider_id: &str) -> Session {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let session_id = format!("session-{}", now);

    Session {
      id: session_id.clone(),
      title: "New Chat".to_string(),
      provider_id: provider_id.to_string(),
      messages: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Get current session
  pub fn get_current_session(&self) -> Option<&Session> {
    self
      .current_session_id
      .as_ref()
      .and_then(|id| self.sessions.iter().find(|s| s.id == *id))
  }

  /// Get current session as mutable
  pub fn get_current_session_mut(&mut self) -> Option<&mut Session> {
    if let Some(ref current_id) = self.current_session_id {
      self.sessions.iter_mut().find(|s| s.id == *current_id)
    } else {
      None
    }
  }

  /// Add message to current session
  /// If message ID already exists, update it instead of adding duplicate
  pub fn add_message(&mut self, message: ChatMessage) {
    if let Some(session) = self.get_current_session_mut() {
      let is_first_user_message = session.messages.is_empty() && message.role == "user";
      let title_preview = if is_first_user_message {
        let content = message.content.clone();
        if content.len() > 40 {
          format!("{}...", content.chars().take(40).collect::<String>())
        } else {
          content.clone()
        }
      } else {
        String::new()
      };

      // Check if message with this ID already exists, update if so
      if let Some(existing_msg) = session.messages.iter_mut().find(|m| m.id == message.id) {
        existing_msg.content = message.content;
        existing_msg.timestamp = message.timestamp;
      } else {
        session.messages.push(message);
      }

      let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
      session.updated_at = now;

      // Auto-generate title from first user message
      if is_first_user_message {
        session.title = title_preview;
      }
    }
  }

  /// Switch to a different session
  pub fn switch_session(&mut self, session_id: &str) {
    self.current_session_id = Some(session_id.to_string());
  }

  /// Create a new session and switch to it
  pub fn create_new_session(&mut self, provider_id: &str) -> String {
    let session = Self::new_session(provider_id);
    let session_id = session.id.clone();
    self.sessions.insert(0, session);
    self.current_session_id = Some(session_id.clone());
    session_id
  }

  /// Delete a session
  pub fn delete_session(&mut self, session_id: &str) {
    self.sessions.retain(|s| s.id != session_id);
    if self.current_session_id.as_ref().map(|s| s.as_str()) == Some(session_id) {
      self.current_session_id = self.sessions.first().map(|s| s.id.clone());
    }
  }

  /// Clear messages in current session
  pub fn clear_current_session(&mut self) {
    if let Some(session) = self.get_current_session_mut() {
      session.messages.clear();
      session.title = "New Chat".to_string();
    }
  }

  /// Get sessions for a specific provider
  pub fn get_sessions_for_provider(&self, provider_id: &str) -> Vec<&Session> {
    self
      .sessions
      .iter()
      .filter(|s| s.provider_id == provider_id)
      .collect()
  }

  /// Update a specific message in current session
  pub fn update_message(&mut self, message_id: &str, new_content: String) {
    if let Some(session) = self.get_current_session_mut() {
      if let Some(msg) = session.messages.iter_mut().find(|m| m.id == message_id) {
        msg.content = new_content;
        let now = std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_secs();
        session.updated_at = now;
      }
    }
  }

  /// Truncate messages from a specific index onwards (for regeneration)
  /// Returns the messages that were removed
  pub fn truncate_from_index(&mut self, index: usize) -> Vec<ChatMessage> {
    if let Some(session) = self.get_current_session_mut() {
      if index < session.messages.len() {
        let removed = session.messages.split_off(index);
        let now = std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap()
          .as_secs();
        session.updated_at = now;
        return removed;
      }
    }
    Vec::new()
  }

  /// Get message index by ID
  pub fn get_message_index(&self, message_id: &str) -> Option<usize> {
    self
      .get_current_session()
      .and_then(|session| session.messages.iter().position(|m| m.id == message_id))
  }
}
