//! Global keyboard shortcuts for Veld
//! Handles registration and handling of system-wide keyboard shortcuts

/// Shortcut event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shortcut {
  CommandPalette, // Open command palette (navigate + focus + type "/")
}

/// Shortcut event
#[derive(Debug, Clone)]
pub struct ShortcutEvent {
  pub shortcut: Shortcut,
}

/// Shortcut manager - simplified wrapper
pub struct ShortcutManager {
  pub shortcuts: Vec<dioxus_desktop::ShortcutHandle>,
}

impl ShortcutManager {
  /// Create a new shortcut manager
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    Ok(ShortcutManager {
      shortcuts: Vec::new(),
    })
  }

  /// Register all default shortcuts (no-op, use use_global_shortcut in components)
  pub fn register_defaults(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }

  /// Check for shortcut events (no-op, handled via callbacks)
  pub fn check_shortcuts(&self) -> Option<Shortcut> {
    None
  }
}

/// Function to manually trigger shortcut (for testing)
pub fn trigger_shortcut(_shortcut: Shortcut) {
  // Testing function - no-op in production
}
