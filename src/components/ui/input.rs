//! Input components with consistent styling
//! 输入组件 - 统一样式

use dioxus::prelude::*;

/// Base text input field
#[derive(Props, Clone, PartialEq)]
pub struct TextFieldProps {
    /// Input value (two-way binding)
    value: String,
    /// Value change handler
    oninput: EventHandler<FormEvent>,
    /// Placeholder text
    #[props(default)]
    placeholder: String,
    /// Input label
    #[props(optional)]
    label: Option<String>,
    /// Icon emoji
    #[props(optional)]
    icon: Option<String>,
    /// Helper text
    #[props(optional)]
    helper: Option<String>,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Disabled state
    #[props(default)]
    disabled: bool,
    /// Input type attribute
    #[props(default)]
    input_type: String,
}

/// Text input field with label and optional icon
///
/// # Example
/// ```rust
/// rsx! {
///     TextField {
///         label: "Name".to_string(),
///         value: name(),
///         placeholder: "Enter name...",
///         oninput: move |e| name.set(e.value()),
///     }
/// }
/// ```
#[component]
pub fn TextField(props: TextFieldProps) -> Element {
    rsx! {
        div {
            class: "space-y-2",
            if let Some(label) = &props.label {
                label {
                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                    if let Some(icon) = &props.icon {
                        span { class: "text-lg", "{icon}" }
                    }
                    "{label}"
                }
            }
            input {
                class: "input-field {props.class}",
                r#type: "{props.input_type}",
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                oninput: props.oninput,
                disabled: props.disabled,
            }
            if let Some(helper) = &props.helper {
                p {
                    class: "text-xs text-text-muted mt-1",
                    "{helper}"
                }
            }
        }
    }
}

/// Password input field
#[component]
pub fn PasswordField(
    value: String,
    oninput: EventHandler<FormEvent>,
    #[props(default)] placeholder: String,
    #[props(default)] label: Option<String>,
    #[props(default)] helper: Option<String>,
    #[props(default)] disabled: bool,
) -> Element {
    rsx! {
        TextField {
            value,
            oninput,
            placeholder,
            label,
            helper,
            disabled,
            input_type: "password".to_string(),
            class: "pr-10".to_string(),
            // Note: Password icon should be handled by parent for toggle functionality
        }
        span {
            class: "absolute right-3 top-1/2 -translate-y-1/2 text-text-muted text-xs -mt-4",
            "🔒"
        }
    }
}

/// Text area input
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// Text area value
    value: String,
    /// Value change handler
    oninput: EventHandler<FormEvent>,
    /// Placeholder text
    #[props(default)]
    placeholder: String,
    /// Number of rows
    #[props(default)]
    rows: usize,
    /// Label
    #[props(optional)]
    label: Option<String>,
    /// Helper text
    #[props(optional)]
    helper: Option<String>,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Disabled state
    #[props(default)]
    disabled: bool,
}

/// Multi-line text input
///
/// # Example
/// ```rust
/// rsx! {
///     TextArea {
///         label: "Description".to_string(),
///         value: description(),
///         rows: 4,
///         placeholder: "Enter description...",
///         oninput: move |e| description.set(e.value()),
///     }
/// }
/// ```
#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
    rsx! {
        div {
            class: "space-y-2",
            if let Some(label) = &props.label {
                label {
                    class: "block text-text-secondary text-sm font-medium",
                    "{label}"
                }
            }
            textarea {
                class: "input-field {props.class}",
                rows: props.rows,
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                oninput: props.oninput,
                disabled: props.disabled,
            }
            if let Some(helper) = &props.helper {
                p {
                    class: "text-xs text-text-muted mt-1",
                    "{helper}"
                }
            }
        }
    }
}
