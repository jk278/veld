//! Session management handlers
//! 会话管理处理器

use dioxus::prelude::*;
use crate::config::AppConfig;
use crate::chat_history::{ChatHistoryData, ChatMessage as HistoryMessage};
use super::message_list::ChatMessage;

/// Create new chat handler
///
/// Returns a FnMut closure that requires ownership
pub fn use_new_chat_handler(
    mut chat_history: Signal<ChatHistoryData>,
    mut messages: Signal<Vec<ChatMessage>>,
    active_provider_id: Signal<String>,
) -> impl FnMut() + Clone {
    move || {
        // Don't create new session if current one is empty
        if messages().is_empty() {
            return;
        }

        // First, save current messages to current session before creating new one
        let current_msgs = messages();
        let mut history = chat_history.write();
        if let Some(session) = history.get_current_session_mut() {
            let history_msgs: Vec<HistoryMessage> = current_msgs.into_iter().map(Into::into).collect();
            session.messages = history_msgs;
        }
        let _ = history.save();

        // Then create new session
        let provider_id = active_provider_id();
        let _new_session_id = history.create_new_session(&provider_id);
        let _ = history.save();
        messages.set(vec![]);

        // Trigger UI update by cloning and dropping the borrow first
        let history_clone = (*history).clone();
        drop(history);
        chat_history.set(history_clone);
    }
}

/// Create switch session handler
pub fn use_switch_session_handler(
    mut chat_history: Signal<ChatHistoryData>,
    mut messages: Signal<Vec<ChatMessage>>,
) -> impl FnMut(String) + Clone {
    move |session_id: String| {
        // Save current messages before switching
        let current_msgs = messages();
        let mut history = chat_history.write();
        if let Some(session) = history.get_current_session_mut() {
            let history_msgs: Vec<HistoryMessage> = current_msgs.into_iter().map(Into::into).collect();
            session.messages = history_msgs;
        }
        let _ = history.save();

        // Then switch to the target session
        history.switch_session(&session_id);
        let _ = history.save();

        // Load target session's messages directly (bypass use_effect placeholder check)
        if let Some(session) = history.get_current_session() {
            let session_msgs: Vec<ChatMessage> = session.messages.iter().cloned().map(Into::into).collect();
            messages.set(session_msgs);
        }

        // Trigger UI update by cloning and dropping the borrow first
        let history_clone = (*history).clone();
        drop(history);
        chat_history.set(history_clone);
    }
}

/// Create delete session handler
pub fn use_delete_session_handler(
    mut chat_history: Signal<ChatHistoryData>,
) -> impl FnMut(String) + Clone {
    move |session_id: String| {
        let mut history = chat_history.write();
        history.delete_session(&session_id);
        let _ = history.save();

        // Trigger UI update by cloning and dropping the borrow first
        let history_clone = (*history).clone();
        drop(history);
        chat_history.set(history_clone);
    }
}

/// Create switch provider handler
pub fn use_switch_provider_handler(
    mut active_provider_id: Signal<String>,
) -> impl FnMut(String) + Clone {
    move |provider_id: String| {
        if let Ok(mut config) = AppConfig::load() {
            config.set_active_provider(provider_id.clone());
            active_provider_id.set(provider_id);
        }
    }
}

/// Create send message handler
pub fn use_send_message_handler(
    mut input_text: Signal<String>,
    tx: Coroutine<String>,
) -> impl FnMut() + Clone {
    move || {
        let text = input_text().trim().to_string();
        if text.is_empty() {
            return;
        }
        input_text.set(String::new());
        tx.send(text);
    }
}
