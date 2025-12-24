//! Chat History Management
//! 聊天历史记录管理 - 完整的会话历史功能

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs;

/// Chat message in history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

/// Chat session (a conversation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub provider_id: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// All chat history data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryData {
    pub sessions: Vec<ChatSession>,
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
        let mut path = dirs::config_dir()
            .ok_or(ChatHistoryError::ConfigDirNotFound)?;
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
    pub fn new_session(provider_id: &str) -> ChatSession {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let session_id = format!("session-{}", now);

        ChatSession {
            id: session_id.clone(),
            title: "New Chat".to_string(),
            provider_id: provider_id.to_string(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<&ChatSession> {
        self.current_session_id.as_ref().and_then(|id| {
            self.sessions.iter().find(|s| s.id == *id)
        })
    }

    /// Get current session as mutable
    pub fn get_current_session_mut(&mut self) -> Option<&mut ChatSession> {
        if let Some(ref current_id) = self.current_session_id {
            self.sessions.iter_mut().find(|s| s.id == *current_id)
        } else {
            None
        }
    }

    /// Add message to current session
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

            session.messages.push(message);
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
    pub fn get_sessions_for_provider(&self, provider_id: &str) -> Vec<&ChatSession> {
        self.sessions.iter()
            .filter(|s| s.provider_id == provider_id)
            .collect()
    }
}
