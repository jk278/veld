//! Window management for Veld
//! Handles creating and managing floating input windows

// use dioxus::prelude::*;
// use dioxus_desktop::DesktopContext;

/// Window state
#[derive(Debug, Clone)]
pub struct WindowState {
    pub is_visible: bool,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Window manager for Veld
pub struct WindowManager {
    pub floating_window_visible: bool,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        WindowManager {
            floating_window_visible: false,
        }
    }

    /// Show the floating input window
    pub fn show_floating_input(&mut self) {
        self.floating_window_visible = true;
        println!("Floating input window shown");
    }

    /// Hide the floating input window
    pub fn hide_floating_input(&mut self) {
        self.floating_window_visible = false;
        println!("Floating input window hidden");
    }

    /// Toggle the floating input window
    pub fn toggle_floating_input(&mut self) {
        self.floating_window_visible = !self.floating_window_visible;
        if self.floating_window_visible {
            println!("Floating input window shown");
        } else {
            println!("Floating input window hidden");
        }
    }
}
