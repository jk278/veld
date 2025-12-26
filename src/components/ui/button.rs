//! Button component with consistent styling
//! 按钮组件 - 统一样式和交互

use dioxus::prelude::*;

/// Button variant/style type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Cancel,
}

impl ButtonVariant {
    pub fn class_name(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "btn-primary",
            ButtonVariant::Secondary => "btn-secondary",
            ButtonVariant::Cancel => "btn-cancel",
        }
    }
}

/// Button component properties
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    /// Button content
    children: Element,
    /// Click handler
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    /// Button variant
    #[props(default)]
    variant: ButtonVariant,
    /// Disabled state
    #[props(default)]
    disabled: bool,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Button type attribute
    #[props(default)]
    button_type: String,
}

/// Button component with consistent styling
///
/// # Example
/// ```rust
/// rsx! {
///     Button { variant: ButtonVariant::Primary, onclick: move |_| {}, "Submit" }
///     Button { variant: ButtonVariant::Cancel, onclick: move |_| {}, "Cancel" }
/// }
/// ```
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_class = props.variant.class_name();
    let disabled_class = if props.disabled { " opacity-50 cursor-not-allowed" } else { "" };

    rsx! {
        button {
            class: "{base_class} {disabled_class} {props.class}",
            r#type: "{props.button_type}",
            disabled: props.disabled,
            onclick: props.onclick,
            {props.children}
        }
    }
}

// Convenience components for common button types

/// Primary button - main action
#[component]
pub fn PrimaryButton(
    children: Element,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Primary,
            onclick,
            disabled,
            class,
            {children}
        }
    }
}

/// Secondary button - alternative action
#[component]
pub fn SecondaryButton(
    children: Element,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            onclick,
            disabled,
            class,
            {children}
        }
    }
}

/// Cancel button - dismissive action
#[component]
pub fn CancelButton(
    children: Element,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Cancel,
            onclick,
            disabled,
            class,
            {children}
        }
    }
}
