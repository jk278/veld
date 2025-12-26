//! Card components for content grouping
//! 卡片组件 - 内容分组容器

use dioxus::prelude::*;

/// Card container
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Card content
    children: Element,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Click handler (makes card interactive)
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
}

/// Card container with consistent styling
///
/// # Example
/// ```rust
/// rsx! {
///     Card {
///         h3 { "Title" }
///         p { "Content" }
///     }
/// }
/// ```
#[component]
pub fn Card(props: CardProps) -> Element {
    rsx! {
        div {
            class: "card cursor-pointer hover:shadow-md transition-shadow {props.class}",
            onclick: props.onclick,
            {props.children}
        }
    }
}

/// Non-interactive card variant
#[component]
pub fn StaticCard(
    children: Element,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div {
            class: "card {class}",
            {children}
        }
    }
}

/// Card header section
#[component]
pub fn CardHeader(
    children: Element,
    #[props(default)] class: String,
    #[props(default)] title: String,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-4 border-b border-border {class}",
            if !title.is_empty() {
                h3 {
                    class: "text-lg font-semibold text-text-primary",
                    "{title}"
                }
            }
            {children}
        }
    }
}

/// Card content section
#[component]
pub fn CardContent(
    children: Element,
    #[props(default)] class: String,
    #[props(default)] padding: String,
) -> Element {
    let padding_class = if padding.is_empty() { "p-4" } else { padding.as_str() };

    rsx! {
        div {
            class: "{padding_class} {class}",
            {children}
        }
    }
}

/// Card footer section
#[component]
pub fn CardFooter(
    children: Element,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-4 border-t border-border {class}",
            {children}
        }
    }
}

/// Info card with icon and message
#[derive(Props, Clone, PartialEq)]
pub struct InfoCardProps {
    /// Card title
    title: String,
    /// Card message/description
    message: String,
    /// Icon emoji
    #[props(default)]
    icon: String,
    /// Card variant
    #[props(default)]
    variant: InfoCardVariant,
    /// Additional CSS classes
    #[props(default)]
    class: String,
}

/// Info card variant
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum InfoCardVariant {
    #[default]
    Info,
    Warning,
    Success,
    Error,
}

impl InfoCardVariant {
    fn bg_color(&self) -> &'static str {
        match self {
            InfoCardVariant::Info => "bg-primary/10",
            InfoCardVariant::Warning => "bg-warning/10",
            InfoCardVariant::Success => "bg-success/10",
            InfoCardVariant::Error => "bg-error/10",
        }
    }

    fn border_color(&self) -> &'static str {
        match self {
            InfoCardVariant::Info => "border-primary/30",
            InfoCardVariant::Warning => "border-warning/30",
            InfoCardVariant::Success => "border-success/30",
            InfoCardVariant::Error => "border-error/30",
        }
    }

    fn default_icon(&self) -> &'static str {
        match self {
            InfoCardVariant::Info => "ℹ️",
            InfoCardVariant::Warning => "⚠️",
            InfoCardVariant::Success => "✅",
            InfoCardVariant::Error => "❌",
        }
    }
}

/// Information card with icon
#[component]
pub fn InfoCard(props: InfoCardProps) -> Element {
    let icon = if props.icon.is_empty() {
        props.variant.default_icon()
    } else {
        props.icon.as_str()
    };

    rsx! {
        div {
            class: "{props.variant.bg_color()} border {props.variant.border_color()} rounded-lg p-4 flex items-start gap-3 {props.class}",
            span {
                class: "text-xl mt-0.5",
                "{icon}"
            }
            div {
                class: "flex-1",
                p {
                    class: "text-sm font-medium text-text-primary mb-1",
                    "{props.title}"
                }
                p {
                    class: "text-xs text-text-secondary leading-relaxed",
                    "{props.message}"
                }
            }
        }
    }
}

/// List item card
#[derive(Props, Clone, PartialEq)]
pub struct ListItemProps {
    /// Item title
    title: String,
    /// Item subtitle
    #[props(default)]
    subtitle: String,
    /// Right side content (actions, badges, etc.)
    right: Element,
    /// Click handler
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    /// Active state
    #[props(default)]
    active: bool,
    /// Additional CSS classes
    #[props(default)]
    class: String,
}

/// List item card
#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-4 bg-bg-surface border border-border rounded-md hover:border-primary transition-colors {props.class}",
            onclick: props.onclick,
            div {
                class: "flex-1",
                div {
                    class: "flex items-center gap-3 mb-2",
                    span {
                        class: "font-mono font-medium text-text-primary",
                        "{props.title}"
                    }
                    {props.right}
                }
                if !props.subtitle.is_empty() {
                    div {
                        class: "text-sm text-text-secondary",
                        "{props.subtitle}"
                    }
                }
            }
        }
    }
}
