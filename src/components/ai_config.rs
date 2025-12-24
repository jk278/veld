//! AI Providers Configuration page
//! AI 提供商配置页面

use dioxus::prelude::*;
use crate::theme::use_theme;
use crate::config::{AppConfig, ProviderConfig, ProviderType};

/// AI Configuration page
/// AI 提供商配置页面
#[component]
pub fn AiConfig() -> Element {
    let _theme_mode = use_theme();

    // Load current config
    let mut providers = use_signal(|| {
        AppConfig::load()
            .map(|c| c.ai.providers)
            .unwrap_or_default()
    });

    let mut active_provider = use_signal(|| {
        AppConfig::load()
            .ok()
            .and_then(|c| c.ai.active_provider)
            .unwrap_or_else(|| "claude".to_string())
    });

    // Track which provider is being edited
    let mut editing_provider = use_signal(|| Option::<String>::None);

    // Form state for new/editing provider
    let mut form_id = use_signal(|| String::new());
    let mut form_name = use_signal(|| String::new());
    let mut form_provider_type = use_signal(|| ProviderType::Claude);
    let mut form_api_key = use_signal(|| String::new());
    let mut form_base_url = use_signal(|| String::new());
    let mut form_model = use_signal(|| String::new());

    // Check if we're editing an existing provider or adding a new one
    let is_adding_mode = move || editing_provider().as_ref().map_or(false, |id| id.is_empty());
    let is_editing = move || editing_provider().as_ref().map_or(false, |id| !id.is_empty());

    // Collect providers for rendering to avoid borrow issues
    let providers_list = providers();
    let active = active_provider();

    rsx! {
        div {
            class: "max-w-5xl mx-auto space-y-6",

            h1 {
                class: "text-3xl font-light text-text-primary mb-8",
                "AI Providers Configuration"
            }

            // API compatibility notice
            div {
                class: "bg-primary/10 border border-primary/30 rounded-lg p-4 flex items-start gap-3",
                span {
                    class: "text-xl mt-0.5",
                    "ℹ️"
                }
                div {
                    class: "flex-1",
                    p {
                        class: "text-sm font-medium text-text-primary mb-1",
                        "Anthropic-Compatible API Required"
                    }
                    p {
                        class: "text-xs text-text-secondary leading-relaxed",
                        "All providers must use the Anthropic Claude API format (Messages API). This includes Claude Code, Kimi (Anthropic-compatible endpoint), MiniMax, and other Claude-compatible services."
                    }
                }
            }

            // Active provider selector
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                h2 {
                    class: "text-xl text-text-primary mb-4",
                    "🤖 Active Provider"
                }

                p {
                    class: "text-text-secondary text-sm mb-4",
                    "Select the AI provider to use for requests"
                }

                div { class: "flex flex-wrap gap-2",
                    for provider in providers_list.iter() {
                        button {
                            class: if active == provider.id {
                                "px-4 py-2 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                            } else if !provider.enabled {
                                "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-primary text-text-muted border border-border opacity-50 cursor-not-allowed"
                            } else {
                                "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border hover:bg-primary hover:text-white"
                            },
                            disabled: !provider.enabled,
                            onclick: {
                                let pid = provider.id.clone();
                                move |_| {
                                    active_provider.set(pid.clone());
                                    if let Ok(mut config) = AppConfig::load() {
                                        config.set_active_provider(pid.clone());
                                    }
                                }
                            },
                            {provider.name.clone()}
                            if !provider.api_key.as_ref().map_or(false, |k| !k.is_empty()) {
                                span {
                                    class: "ml-2 text-xs opacity-70",
                                    "(no key)"
                                }
                            }
                        }
                    }
                }
            }

            // Provider list
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                div {
                    class: "flex justify-between items-center mb-4",
                    h2 {
                        class: "text-xl text-text-primary",
                        "📋 Provider List"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            editing_provider.set(Some(String::new())); // Use empty string to signal "add new"
                            form_id.set(String::new());
                            form_name.set(String::new());
                            form_provider_type.set(ProviderType::Claude);
                            form_api_key.set(String::new());
                            form_base_url.set(ProviderType::Claude.default_base_url().to_string());
                            form_model.set(ProviderType::Claude.default_model().to_string());
                        },
                        "➕ Add Provider"
                    }
                }

                div {
                    class: "space-y-3",
                    for provider in providers_list.iter() {
                        div {
                            class: "flex items-center justify-between p-4 bg-bg-primary border border-border rounded-md hover:border-primary transition-colors",

                            div {
                                class: "flex-1",

                                div {
                                    class: "flex items-center gap-3 mb-2",
                                    span {
                                        class: "font-mono font-medium text-text-primary",
                                        {provider.name.clone()}
                                    }
                                    span {
                                        class: "px-2 py-0.5 text-xs bg-bg-secondary text-text-secondary rounded font-mono",
                                        {format!("{:?}", provider.provider_type)}
                                    }
                                    if active == provider.id {
                                        span {
                                            class: "px-2 py-0.5 text-xs bg-primary text-white rounded font-mono",
                                            "ACTIVE"
                                        }
                                    }
                                }

                                div {
                                    class: "flex flex-wrap gap-x-4 gap-y-1 text-sm text-text-secondary",
                                    if let Some(model) = &provider.model {
                                        span {
                                            class: "font-mono text-xs",
                                            "Model: {model}"
                                        }
                                    }
                                    if let Some(url) = &provider.base_url {
                                        span {
                                            class: "font-mono text-xs truncate max-w-xs",
                                            "URL: {url}"
                                        }
                                    }
                                }
                            }

                            div {
                                class: "flex items-center gap-2",
                                label {
                                    class: "flex items-center gap-2 cursor-pointer text-text-secondary hover:text-text-primary transition-colors text-sm",

                                    input {
                                        r#type: "checkbox",
                                        checked: provider.enabled,
                                        oninput: {
                                            let pid = provider.id.clone();
                                            move |e| {
                                                let pid = pid.clone();
                                                if let Ok(mut config) = AppConfig::load() {
                                                    if let Some(pr) = config.ai.providers.iter_mut().find(|p| p.id == pid) {
                                                        pr.enabled = e.checked();
                                                    }
                                                    let _ = config.save();
                                                    providers.set(config.ai.providers.clone());
                                                }
                                            }
                                        },
                                        class: "w-4 h-4 text-primary bg-bg-surface border-border rounded focus:ring-primary focus:ring-2",
                                    }

                                    span {
                                        "Enabled"
                                    }
                                }

                                button {
                                    class: "px-3 py-1 text-sm bg-bg-secondary text-text-primary rounded border border-border hover:bg-primary hover:text-white transition-colors",
                                    onclick: {
                                        let p = provider.clone();
                                        move |_| {
                                            editing_provider.set(Some(p.id.clone()));
                                            form_id.set(p.id.clone());
                                            form_name.set(p.name.clone());
                                            form_provider_type.set(p.provider_type.clone());
                                            form_api_key.set(p.api_key.clone().unwrap_or_default());
                                            form_base_url.set(p.base_url.clone().unwrap_or_default());
                                            form_model.set(p.model.clone().unwrap_or_default());
                                        }
                                    },
                                    "Edit"
                                }

                                button {
                                    class: "px-3 py-1 text-sm bg-bg-secondary text-text-secondary rounded border border-border hover:bg-error hover:text-white transition-colors",
                                    onclick: {
                                        let pid = provider.id.clone();
                                        move |_| {
                                            let pid = pid.clone();
                                            if let Ok(mut config) = AppConfig::load() {
                                                config.ai.providers.retain(|p| p.id != pid);
                                                let _ = config.save();
                                                providers.set(config.ai.providers.clone());
                                            }
                                        }
                                    },
                                    "Delete"
                                }
                            }
                        }
                    }
                }
            }

            // Edit/Add form modal
            if editing_provider().is_some() {
                div {
                    class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
                    onclick: move |_| editing_provider.set(None),

                    div {
                        class: "bg-bg-surface border border-border rounded-xl p-6 max-w-2xl w-full shadow-2xl animate-in fade-in zoom-in duration-200",
                        onclick: move |e: MouseEvent| e.stop_propagation(),

                        // Header with icon
                        div {
                            class: "flex items-center gap-3 mb-6 pb-4 border-b border-border",

                            div {
                                class: "w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center",
                                span {
                                    class: "text-xl",
                                    {if is_adding_mode() { "➕" } else { "✏️" }}
                                }
                            }

                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-xl font-semibold text-text-primary",
                                    {if is_adding_mode() { "Add New Provider" } else { "Edit Provider" }}
                                }
                                p {
                                    class: "text-sm text-text-secondary mt-1",
                                    {if is_adding_mode() { "Configure a new AI provider" } else { "Update the provider configuration below" }}
                                }
                            }

                            button {
                                class: "w-8 h-8 rounded-md bg-bg-primary text-text-secondary hover:text-text-primary hover:bg-bg-secondary flex items-center justify-center transition-colors",
                                onclick: move |_| editing_provider.set(None),
                                "×"
                            }
                        }

                        // Form fields with better grouping
                        div {
                            class: "space-y-5",

                            // Provider Type (input field for flexibility)
                            div {
                                class: "space-y-2.5",
                                label {
                                    class: "flex items-center gap-2 text-sm font-semibold text-text-primary",
                                    span { class: "text-lg", "🤖" }
                                    "Provider Type"
                                }
                                input {
                                    class: "w-full p-3 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all",
                                    r#type: "text",
                                    placeholder: "Claude, Kimi, MiniMax, GLM, UltraThink, or custom...",
                                    value: format!("{:?}", form_provider_type()),
                                    oninput: move |e| {
                                        let type_str = e.value();
                                        form_provider_type.set(match type_str.as_str() {
                                            "Claude" => ProviderType::Claude,
                                            "Kimi" => ProviderType::Kimi,
                                            "MiniMax" => ProviderType::MiniMax,
                                            "GLM" => ProviderType::GLM,
                                            "UltraThink" => ProviderType::UltraThink,
                                            _ => ProviderType::Claude, // Default to Claude for unknown types
                                        });
                                        // Update defaults when provider type changes
                                        let ptype = form_provider_type();
                                        form_base_url.set(ptype.default_base_url().to_string());
                                        form_model.set(ptype.default_model().to_string());
                                    },
                                }
                                p {
                                    class: "text-xs text-text-secondary mt-1",
                                    "Enter a provider name. Common types: Claude, Kimi, MiniMax, GLM, UltraThink"
                                }
                            }

                            // Display Name
                            div {
                                class: "space-y-2",
                                label {
                                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                                    span { "📝" }
                                    "Display Name"
                                }
                                input {
                                    class: "w-full p-2.5 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all",
                                    r#type: "text",
                                    placeholder: "My Claude Instance",
                                    value: form_name(),
                                    oninput: move |e| form_name.set(e.value()),
                                }
                            }

                            // API Key (prominent, required)
                            div {
                                class: "space-y-2",
                                label {
                                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                                    span { "🔑" }
                                    "API Key"
                                    span {
                                        class: "text-xs text-text-muted font-normal ml-auto",
                                        "(required for requests)"
                                    }
                                }
                                div {
                                    class: "relative",
                                    input {
                                        class: "w-full p-3 pr-10 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all",
                                        r#type: "password",
                                        placeholder: "sk-ant-...",
                                        value: form_api_key(),
                                        oninput: move |e| form_api_key.set(e.value()),
                                    }
                                    span {
                                        class: "absolute right-3 top-1/2 -translate-y-1/2 text-text-muted text-xs",
                                        "🔒"
                                    }
                                }
                            }

                            // Advanced settings (collapsible look)
                            div {
                                class: "space-y-4 pt-4 border-t border-border/50",

                                p {
                                    class: "text-xs font-semibold text-text-secondary uppercase tracking-wider mb-3",
                                    "Advanced Settings"
                                }

                                // Base URL and Model in grid
                                div {
                                    class: "grid grid-cols-2 gap-4",

                                    // Base URL
                                    div {
                                        class: "space-y-2",
                                        label {
                                            class: "flex items-center gap-2 text-sm font-medium text-text-secondary",
                                            span { "🌐" }
                                            "Base URL"
                                        }
                                        input {
                                            class: "w-full p-2.5 bg-bg-primary text-text-secondary border border-border rounded-lg font-mono text-sm focus:border-primary outline-none transition-all",
                                            r#type: "text",
                                            placeholder: "Auto-filled",
                                            value: form_base_url(),
                                            oninput: move |e| form_base_url.set(e.value()),
                                        }
                                    }

                                    // Model
                                    div {
                                        class: "space-y-2",
                                        label {
                                            class: "flex items-center gap-2 text-sm font-medium text-text-secondary",
                                            span { "⚙️" }
                                            "Model"
                                        }
                                        input {
                                            class: "w-full p-2.5 bg-bg-primary text-text-secondary border border-border rounded-lg font-mono text-sm focus:border-primary outline-none transition-all",
                                            r#type: "text",
                                            placeholder: "Auto-filled",
                                            value: form_model(),
                                            oninput: move |e| form_model.set(e.value()),
                                        }
                                    }
                                }
                            }
                        }

                        // Form actions (improved styling)
                        div {
                            class: "flex justify-end gap-3 mt-6 pt-5 border-t border-border",

                            button {
                                class: "px-5 py-2.5 rounded-lg bg-bg-primary text-text-secondary border border-border font-medium hover:bg-bg-secondary hover:text-text-primary transition-all",
                                onclick: move |_| editing_provider.set(None),
                                "Cancel"
                            }
                            button {
                                class: "px-5 py-2.5 rounded-lg bg-primary text-white font-medium hover:bg-primary/90 transition-all shadow-lg shadow-primary/25 flex items-center gap-2",
                                onclick: move |_| {
                                    let provider = ProviderConfig {
                                        id: if is_editing() { form_id() } else { format!("{:?}", form_provider_type()).to_lowercase() },
                                        name: if form_name().is_empty() {
                                            format!("{:?}", form_provider_type())
                                        } else {
                                            form_name()
                                        },
                                        provider_type: form_provider_type(),
                                        api_key: if form_api_key().is_empty() { None } else { Some(form_api_key()) },
                                        base_url: if form_base_url().is_empty() { None } else { Some(form_base_url()) },
                                        model: if form_model().is_empty() { None } else { Some(form_model()) },
                                        enabled: true,
                                    };

                                    if let Ok(mut config) = AppConfig::load() {
                                        config.update_provider(provider);
                                        providers.set(config.ai.providers.clone());
                                    }
                                    editing_provider.set(None);
                                },
                                span { "💾" }
                                "Save Provider"
                            }
                        }
                    }
                }
            }
        }
    }
}
