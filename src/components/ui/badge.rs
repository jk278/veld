//! Badge components for status indicators
//! 徽章组件 - 状态指示器

use dioxus::prelude::*;

/// Badge component
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    /// Badge content
    children: Element,
    /// Badge variant
    #[props(default)]
    variant: BadgeVariant,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Small size
    #[props(default)]
    small: bool,
}

/// Badge variant
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

impl BadgeVariant {
    fn base_class(&self) -> &'static str {
        match self {
            BadgeVariant::Default => "bg-bg-secondary text-text-secondary",
            BadgeVariant::Primary => "bg-primary text-white",
            BadgeVariant::Success => "bg-success text-white",
            BadgeVariant::Warning => "bg-warning text-white",
            BadgeVariant::Error => "bg-error text-white",
            BadgeVariant::Info => "bg-info text-white",
        }
    }
}

/// Badge component for labels and tags
///
/// # Example
/// ```rust
/// rsx! {
///     Badge {
///         variant: BadgeVariant::Success,
///         "Ready"
///     }
/// }
/// ```
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let size_class = if props.small { "text-xs px-2 py-0.5" } else { "text-sm px-2.5 py-1" };
    let base_class = props.variant.base_class();

    rsx! {
        span {
            class: "inline-flex items-center rounded-full font-mono font-medium {base_class} {size_class} {props.class}",
            {props.children}
        }
    }
}

/// Status badge with icon and text
#[derive(Props, Clone, PartialEq)]
pub struct StatusBadgeProps {
    /// Status type
    #[props(default)]
    status: StatusType,
    /// Status text
    #[props(default)]
    text: String,
    /// Show icon
    #[props(default)]
    show_icon: bool,
    /// Small size
    #[props(default)]
    small: bool,
}

/// Status type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StatusType {
    #[default]
    Info,
    Ready,
    Disabled,
    Warning,
    Error,
    Loading,
}

impl StatusType {
    fn variant(&self) -> BadgeVariant {
        match self {
            StatusType::Ready => BadgeVariant::Success,
            StatusType::Disabled => BadgeVariant::Default,
            StatusType::Warning => BadgeVariant::Warning,
            StatusType::Error => BadgeVariant::Error,
            StatusType::Loading => BadgeVariant::Info,
            StatusType::Info => BadgeVariant::Info,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            StatusType::Ready => "✓",
            StatusType::Disabled => "○",
            StatusType::Warning => "⚠️",
            StatusType::Error => "✕",
            StatusType::Loading => "⋯",
            StatusType::Info => "ℹ️",
        }
    }

    fn default_text(&self) -> &'static str {
        match self {
            StatusType::Info => "Info",
            StatusType::Ready => "Ready",
            StatusType::Disabled => "Disabled",
            StatusType::Warning => "Warning",
            StatusType::Error => "Error",
            StatusType::Loading => "Loading",
        }
    }
}

/// Status badge with optional icon
#[component]
pub fn StatusBadge(props: StatusBadgeProps) -> Element {
    let display_text = if props.text.is_empty() {
        props.status.default_text()
    } else {
        props.text.as_str()
    };

    rsx! {
        Badge {
            variant: props.status.variant(),
            small: props.small,
            class: "bg-opacity-90",
            if props.show_icon {
                span {
                    class: "mr-1",
                    "{props.status.icon()}"
                }
            }
            "{display_text}"
        }
    }
}

/// Inline tag badge
#[component]
pub fn Tag(
    children: Element,
    #[props(default)] variant: BadgeVariant,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        Badge {
            variant,
            class: "rounded-md {class}",
            {children}
        }
    }
}

/// Provider type badge
#[component]
pub fn ProviderBadge(
    provider_type: String,
    #[props(default)] small: bool,
) -> Element {
    rsx! {
        Badge {
            variant: BadgeVariant::Default,
            small,
            "{provider_type}"
        }
    }
}
