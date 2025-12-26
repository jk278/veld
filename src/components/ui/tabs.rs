//! Tab component for settings/navigation
//! 标签页组件 - 统一导航样式

use dioxus::prelude::*;

/// Tab item
#[derive(Props, Clone, PartialEq)]
pub struct TabProps {
    /// Tab label
    label: String,
    /// Tab value/identifier
    value: String,
    /// Current active tab value
    active_value: String,
    /// Icon emoji
    #[props(default)]
    icon: String,
    /// Click handler
    onclick: EventHandler<String>,
    /// Disabled state
    #[props(default)]
    disabled: bool,
}

/// Individual tab button
#[component]
pub fn Tab(props: TabProps) -> Element {
    let is_active = props.active_value == props.value;
    let base_class = "nav-tab";
    let active_class = if is_active { "active" } else { "" };

    rsx! {
        button {
            class: "{base_class} {active_class}",
            onclick: move |_| props.onclick.call(props.value.clone()),
            disabled: props.disabled,
            if !props.icon.is_empty() {
                span { class: "mr-2", "{props.icon}" }
            }
            "{props.label}"
        }
    }
}

/// Tab list container
#[component]
pub fn TabList(
    children: Element,
    #[props(default)] class: String,
    #[props(default)] width: String,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 border-r border-border pr-6 {class}",
            style: "width: {width}",
            {children}
        }
    }
}

/// Tab panel container
#[component]
pub fn TabPanel(
    children: Element,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div {
            class: "flex-1 max-w-5xl overflow-y-auto {class}",
            {children}
        }
    }
}

/// Tabs container component
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    /// Tab panels (content for each tab)
    children: Element,
    /// Current active tab value
    active_value: String,
    /// Tab change handler
    onchange: EventHandler<String>,
}

/// Tabs component with integrated list and panels
///
/// # Example
/// ```rust
/// rsx! {
///     Tabs {
///         active_value: active_tab(),
///         onchange: move |v| active_tab.set(v),
///         TabList {
///             Tab {
///                 label: "Settings".to_string(),
///                 value: "settings".to_string(),
///                 active_value: active_tab(),
///                 onclick: move |v| active_tab.set(v),
///                 icon: "⚙️".to_string(),
///             }
///         }
///         TabPanel {
///             h1 { "Settings Content" }
///         }
///     }
/// }
/// ```
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    rsx! {
        div {
            class: "flex gap-6 h-full",
            {props.children}
        }
    }
}

/// Navigation tab (simplified for settings sidebar)
#[component]
pub fn NavTab(
    label: String,
    value: String,
    active_value: String,
    #[props(default)] icon: String,
    onclick: EventHandler<String>,
) -> Element {
    let is_active = active_value == value;
    let base_class = "nav-tab";
    let active_class = if is_active { "active" } else { "" };

    rsx! {
        button {
            class: "{base_class} {active_class}",
            onclick: move |_| onclick.call(value.clone()),
            if !icon.is_empty() {
                span { class: "mr-2", "{icon}" }
            }
            "{label}"
        }
    }
}
