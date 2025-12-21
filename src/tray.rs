//! System tray management for Veld
//! Handles creating and managing the system tray icon and menu using dioxus-desktop built-in APIs

use dioxus_desktop::trayicon::{TrayIcon, TrayIconAttributes, Icon, menu::{Menu, PredefinedMenuItem}};

/// Tray event types
#[derive(Debug, Clone)]
pub enum TrayEvent {
    ShowFloatingInput,
    Exit,
}

/// System tray manager
pub struct SystemTray {
    _tray_icon: TrayIcon,
    _menu: Menu,
}

impl SystemTray {
    /// Create a new system tray using dioxus-desktop built-in API
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("[SystemTray] Creating tray icon with dioxus-desktop built-in API...");

        // Load icon from file (favicon.ico) with specific size
        let icon = Icon::from_path("assets/favicon.ico", None)?;

        // Create a context menu for the tray icon (required for Linux to show the icon)
        let menu = Menu::new();

        // Add cross-platform compatible menu items
        let show_item = PredefinedMenuItem::about(
            Some("Show Floating Input"),
            None,
        );
        let separator = PredefinedMenuItem::separator();
        let close_item = PredefinedMenuItem::close_window(Some("Exit"));

        menu.append_items(&[&show_item, &separator, &close_item])?;

        // Configure tray icon attributes with menu
        let attrs = TrayIconAttributes {
            icon: Some(icon),
            tooltip: Some("Veld - AI Toolkit".to_string()),
            menu: Some(Box::new(menu.clone())),
            menu_on_left_click: false, // Disable default left-click menu behavior
            ..Default::default()
        };

        // Create the tray icon
        let tray_icon = TrayIcon::new(attrs)?;
        println!("[SystemTray] ✓ Tray icon created successfully with context menu");

        Ok(SystemTray {
            _tray_icon: tray_icon,
            _menu: menu,
        })
    }
}
