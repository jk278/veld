//! Modal dialog components
//! 模态框组件 - 统一弹窗样式

use dioxus::prelude::*;

/// Modal container with overlay
#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    /// Modal content
    children: Element,
    /// Whether modal is visible
    #[props(default)]
    show: bool,
    /// Close handler (for overlay click)
    #[props(default)]
    onclose: EventHandler<MouseEvent>,
    /// Maximum width
    #[props(default)]
    max_width: String,
    /// Additional container CSS classes
    #[props(default)]
    class: String,
}

/// Modal dialog with overlay
///
/// # Example
/// ```rust
/// rsx! {
///     Modal {
///         show: is_open(),
///         onclose: move |_| is_open.set(false),
///         h2 { "Title" }
///         p { "Content" }
///     }
/// }
/// ```
#[component]
pub fn Modal(props: ModalProps) -> Element {
    if !props.show {
        return rsx! { };
    }

    rsx! {
        div {
            class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            onclick: props.onclose,
            div {
                class: "bg-bg-surface border border-border rounded-xl p-6 w-full shadow-2xl animate-in fade-in zoom-in duration-200 {props.class}",
                style: "max-width: {props.max_width}",
                onclick: move |e: MouseEvent| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

/// Modal header with title and close button
#[component]
pub fn ModalHeader(
    children: Element,
    #[props(default)] title: String,
    #[props(default)] subtitle: String,
    #[props(default)] icon: String,
    #[props(default)] show_close: bool,
    #[props(default)] onclose: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-3 mb-6 pb-4 border-b border-border",
            if !icon.is_empty() {
                div {
                    class: "w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center",
                    span {
                        class: "text-xl",
                        "{icon}"
                    }
                }
            }
            div {
                class: "flex-1",
                if !title.is_empty() {
                    h3 {
                        class: "text-xl font-semibold text-text-primary",
                        "{title}"
                    }
                }
                if !subtitle.is_empty() {
                    p {
                        class: "text-sm text-text-secondary mt-1",
                        "{subtitle}"
                    }
                }
                {children}
            }
            if show_close {
                button {
                    class: "w-8 h-8 rounded-md bg-bg-primary text-text-secondary hover:text-text-primary hover:bg-bg-secondary flex items-center justify-center transition-colors",
                    onclick: onclose,
                    "×"
                }
            }
        }
    }
}

/// Modal content area
#[component]
pub fn ModalContent(
    children: Element,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div {
            class: "space-y-5 {class}",
            {children}
        }
    }
}

/// Modal footer with action buttons
#[component]
pub fn ModalFooter(
    children: Element,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div {
            class: "flex justify-end gap-3 mt-6 pt-5 border-t border-border {class}",
            {children}
        }
    }
}

/// Form section within modal
#[component]
pub fn FormSection(
    children: Element,
    #[props(default)] title: String,
    #[props(default)] description: String,
) -> Element {
    rsx! {
        div {
            class: "space-y-2.5",
            if !title.is_empty() {
                label {
                    class: "flex items-center gap-2 text-sm font-semibold text-text-primary",
                    "{title}"
                }
            }
            if !description.is_empty() {
                p {
                    class: "text-xs text-text-secondary",
                    "{description}"
                }
            }
            {children}
        }
    }
}

/// Advanced settings section
#[component]
pub fn AdvancedSection(
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "space-y-4 pt-4 border-t border-border/50",
            p {
                class: "text-xs font-semibold text-text-secondary uppercase tracking-wider mb-3",
                "Advanced Settings"
            }
            {children}
        }
    }
}
