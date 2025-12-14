//! System tray management for Veld
//! Handles creating and managing the system tray icon and menu

use tray_icon::{
    menu::Menu,
    TrayIcon, TrayIconBuilder,
};

/// Tray event types
#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowFloatingInput,
    Exit,
}

/// System tray manager
pub struct SystemTray {
    _tray_icon: TrayIcon,
}

impl SystemTray {
    /// Create a new system tray
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create a simple menu with items
        let menu = Menu::new();

        // Create icon (use a simple colored icon for now)
        let icon = create_icon()?;

        // Build the tray icon
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Veld - AI Toolkit")
            .with_icon(icon)
            .build()?;

        Ok(SystemTray {
            _tray_icon,
        })
    }

    /// Handle tray events (to be called in the event loop)
    pub fn handle_events(&self) -> Option<TrayEvent> {
        // TODO: Implement tray event handling
        None
    }
}

fn create_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    // Load favicon.ico as the tray icon
    let icon_path = std::path::Path::new("assets/favicon.ico");

    // Check if the file exists
    if !icon_path.exists() {
        return Err(format!("Icon file not found: {}", icon_path.display()).into());
    }

    // Load the icon from path
    let icon = tray_icon::Icon::from_path(icon_path, Some((64, 64)))?;

    Ok(icon)
}


