//! SVG Icon components using dioxus-free-icons
//!
//! Design principles:
//! - Consistent sizing: use `w-4 h-4` (16px), `w-5 h-5` (20px), `w-6 h-6` (24px)
//! - Consistent colors: `currentColor` inherits text color
//! - Semantic naming: use descriptive names (SettingsIcon, CheckIcon, etc.)

use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_solid_icons, Icon};

/// Settings gear icon
#[component]
pub fn SettingsIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaGear,
        }
    }
}

/// Lightning bolt icon (for MCP/Servers)
#[component]
pub fn BoltIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaBolt,
        }
    }
}

/// Rocket icon (for Quick Tools/Features)
#[component]
pub fn RocketIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaRocket,
        }
    }
}

/// Check mark icon (for success/ready states)
#[component]
pub fn CheckIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-4 h-4"),
            width: 16,
            height: 16,
            fill: "currentColor",
            icon: fa_solid_icons::FaCheck,
        }
    }
}

/// X/Cross icon (for error/cancel states)
#[component]
pub fn XIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-4 h-4"),
            width: 16,
            height: 16,
            fill: "currentColor",
            icon: fa_solid_icons::FaXmark,
        }
    }
}

/// File text icon (for documents/notes)
#[component]
pub fn FileTextIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-4 h-4"),
            width: 16,
            height: 16,
            fill: "currentColor",
            icon: fa_solid_icons::FaFileLines,
        }
    }
}

/// Plus/Add icon
#[component]
pub fn PlusIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaPlus,
        }
    }
}

/// Trash/Delete icon
#[component]
pub fn TrashIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaTrash,
        }
    }
}

/// Edit/Pencil icon
#[component]
pub fn EditIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaPencil,
        }
    }
}

/// Copy icon
#[component]
pub fn CopyIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaCopy,
        }
    }
}

/// Refresh/Reload icon
#[component]
pub fn RefreshIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaRotateRight,
        }
    }
}

/// Chevron/Arrow right icon
#[component]
pub fn ChevronRightIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-4 h-4"),
            width: 16,
            height: 16,
            fill: "currentColor",
            icon: fa_solid_icons::FaChevronRight,
        }
    }
}

/// Search icon
#[component]
pub fn SearchIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaMagnifyingGlass,
        }
    }
}

/// Keyboard icon (for shortcuts)
#[component]
pub fn KeyboardIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaKeyboard,
        }
    }
}

/// Palette icon (for appearance/theme)
#[component]
pub fn PaletteIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaPalette,
        }
    }
}

/// Server icon (for MCP servers)
#[component]
pub fn ServerIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaServer,
        }
    }
}

/// Robot icon (for AI/Agent)
#[component]
pub fn RobotIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaRobot,
        }
    }
}

/// Circle info icon (for about/help)
#[component]
pub fn InfoIcon(class: Option<&'static str>) -> Element {
    rsx! {
        Icon {
            class: class.unwrap_or("w-5 h-5"),
            width: 20,
            height: 20,
            fill: "currentColor",
            icon: fa_solid_icons::FaCircleInfo,
        }
    }
}

/// Window minimize icon (horizontal line)
#[component]
pub fn MinimizeIcon() -> Element {
    rsx! {
        svg {
            class: "w-3 h-3",
            view_box: "0 0 10 1",
            fill: "currentColor",
            rect {
                x: "0",
                y: "0",
                width: "10",
                height: "1",
            }
        }
    }
}

/// Window maximize icon (square)
#[component]
pub fn MaximizeIcon() -> Element {
    rsx! {
        svg {
            class: "w-3 h-3",
            view_box: "0 0 10 10",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "1",
            rect {
                x: "0.5",
                y: "0.5",
                width: "9",
                height: "9",
            }
        }
    }
}

/// Window close icon (X)
#[component]
pub fn CloseIcon() -> Element {
    rsx! {
        svg {
            class: "w-3 h-3",
            view_box: "0 0 10 10",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "1.2",
            path {
                d: "M0 0L10 10M10 0L0 10",
            }
        }
    }
}
