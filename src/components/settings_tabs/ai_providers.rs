//! AI Providers tab component
//! AI 提供商配置标签页

use dioxus::prelude::*;
use crate::config::{AppConfig, ProviderConfig, ProviderType};
use crate::components::ui::*;

/// AI Providers tab content
#[component]
pub fn AiProvidersTab(
    mut providers: Signal<Vec<ProviderConfig>>,
    mut editing_provider: Signal<Option<String>>,
    // Form state signals
    mut form_id: Signal<String>,
    mut form_name: Signal<String>,
    mut form_provider_type: Signal<ProviderType>,
    mut form_api_key: Signal<String>,
    mut form_base_url: Signal<String>,
    mut form_model: Signal<String>,
) -> Element {
    let providers_list = providers();
    let is_adding_mode = move || editing_provider().as_ref().map_or(false, |id| id.is_empty());

    rsx! {
        div {
            class: "space-y-6",
            h1 {
                class: "text-2xl font-semibold text-text-primary",
                "AI Providers"
            }

            // API compatibility notice
            InfoCard {
                title: "Anthropic-Compatible API Required".to_string(),
                message: "All providers must use the Anthropic Claude API format (Messages API).".to_string(),
                icon: "ℹ️".to_string(),
                variant: InfoCardVariant::Info,
            }

            // Add new provider button
            div {
                class: "flex justify-end mb-4",
                PrimaryButton {
                    onclick: move |_| {
                        editing_provider.set(Some(String::new()));
                        form_id.set(String::new());
                        form_name.set(String::new());
                        form_provider_type.set(ProviderType::Claude);
                        form_api_key.set(String::new());
                        form_base_url.set(String::new());
                        form_model.set(String::new());
                    },
                    "＋ Add Provider"
                }
            }

            // Providers list
            div {
                class: "space-y-3",
                for provider in providers_list.iter() {
                    ProviderListItem {
                        provider: provider.clone(),
                        key: "{provider.id}",
                        onedit: {
                            let pid = provider.id.clone();
                            let pname = provider.name.clone();
                            let ptype = provider.provider_type.clone();
                            let papi_key = provider.api_key.clone().unwrap_or_default();
                            let pbase_url = provider.base_url.clone().unwrap_or_default();
                            let pmodel = provider.model.clone().unwrap_or_default();
                            move |_| {
                                editing_provider.set(Some(pid.clone()));
                                form_id.set(pid.clone());
                                form_name.set(pname.clone());
                                form_provider_type.set(ptype.clone());
                                form_api_key.set(papi_key.clone());
                                form_base_url.set(pbase_url.clone());
                                form_model.set(pmodel.clone());
                            }
                        },
                        ondelete: {
                            let pid = provider.id.clone();
                            let mut providers = providers.clone();
                            move |_| {
                                if let Ok(mut config) = AppConfig::load() {
                                    config.ai.providers.retain(|p| p.id != pid);
                                    if let Err(e) = config.save() {
                                        eprintln!("[Settings] Failed to save provider deletion: {}", e);
                                    }
                                    providers.set(config.ai.providers.clone());
                                }
                            }
                        },
                    }
                }
            }

            // Edit/Add modal
            ProviderModal {
                show: editing_provider().is_some(),
                onclose: move |_| editing_provider.set(None),
                is_adding_mode: is_adding_mode(),
                form_id: form_id.clone(),
                form_name: form_name.clone(),
                form_provider_type: form_provider_type.clone(),
                form_api_key: form_api_key.clone(),
                form_base_url: form_base_url.clone(),
                form_model: form_model.clone(),
                onsave: {
                    let mut providers = providers.clone();
                    move |provider_config| {
                        if let Ok(mut config) = AppConfig::load() {
                            config.update_provider(provider_config);
                            if let Err(e) = config.save() {
                                eprintln!("[Settings] Failed to save provider update: {}", e);
                            }
                            providers.set(config.ai.providers.clone());
                        }
                        editing_provider.set(None);
                    }
                },
            }
        }
    }
}

/// Provider list item component
#[component]
fn ProviderListItem(
    provider: ProviderConfig,
    #[props(optional)] onedit: Option<EventHandler<MouseEvent>>,
    #[props(optional)] ondelete: Option<EventHandler<MouseEvent>>,
) -> Element {
    let is_usable = provider.enabled && provider.api_key.as_ref().map_or(false, |k| !k.is_empty());

    rsx! {
        div {
            class: "flex items-center justify-between p-4 bg-bg-surface border border-border rounded-md hover:border-primary transition-colors",
            div {
                class: "flex-1",
                div {
                    class: "flex items-center gap-3 mb-2",
                    span {
                        class: "font-mono font-medium text-text-primary",
                        "{provider.name}"
                    }
                    ProviderBadge {
                        provider_type: format!("{:?}", provider.provider_type),
                        small: true,
                    }
                    StatusBadge {
                        status: if is_usable { StatusType::Ready }
                                  else if provider.enabled { StatusType::Warning }
                                  else { StatusType::Disabled },
                        text: if is_usable { "".to_string() }
                               else if provider.enabled { "Missing Key".to_string() }
                               else { "".to_string() },
                        small: true,
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
                        onchange: move |e| {
                            if let Ok(mut config) = AppConfig::load() {
                                if let Some(pr) = config.ai.providers.iter_mut().find(|p| p.id == provider.id) {
                                    pr.enabled = e.checked();
                                }
                                if let Err(err) = config.save() {
                                    eprintln!("[Settings] Failed to save provider toggle: {}", err);
                                }
                            }
                        },
                        class: "w-4 h-4 text-primary bg-bg-surface border-border rounded focus:ring-primary focus:ring-2",
                    }
                    span { "Enabled" }
                }
                SecondaryButton {
                    class: "px-3 py-1 text-sm".to_string(),
                    onclick: move |e| {
                        if let Some(handler) = onedit {
                            handler.call(e);
                        }
                    },
                    "Edit"
                }
                Button {
                    variant: ButtonVariant::Cancel,
                    class: "px-3 py-1 text-sm hover:bg-red-50 dark:hover:bg-red-900/20".to_string(),
                    onclick: move |e| {
                        if let Some(handler) = ondelete {
                            handler.call(e);
                        }
                    },
                    "Delete"
                }
            }
        }
    }
}

/// Provider edit/add modal
#[component]
fn ProviderModal(
    show: bool,
    onclose: EventHandler<MouseEvent>,
    is_adding_mode: bool,
    form_id: Signal<String>,
    form_name: Signal<String>,
    form_provider_type: Signal<ProviderType>,
    form_api_key: Signal<String>,
    form_base_url: Signal<String>,
    form_model: Signal<String>,
    onsave: EventHandler<ProviderConfig>,
) -> Element {
    rsx! {
        Modal {
            show,
            onclose,
            max_width: "40rem".to_string(),
            ModalHeader {
                title: (if is_adding_mode { "Add New Provider" } else { "Edit Provider" }).to_string(),
                subtitle: (if is_adding_mode { "Configure a new AI provider" } else { "Update the provider configuration below" }).to_string(),
                icon: (if is_adding_mode { "➕" } else { "✏️" }).to_string(),
                show_close: true,
                onclose,
            }
            ModalContent {
                FormSection {
                    title: "".to_string(),
                    TextField {
                        label: "Provider Type".to_string(),
                        value: format!("{:?}", form_provider_type()),
                        placeholder: "Claude, Kimi, MiniMax, GLM...".to_string(),
                        oninput: move |e: FormEvent| {
                            let type_str = e.value();
                            form_provider_type.set(match type_str.as_str() {
                                "Claude" => ProviderType::Claude,
                                "Kimi" => ProviderType::Kimi,
                                "MiniMax" => ProviderType::MiniMax,
                                "GLM" => ProviderType::GLM,
                                "UltraThink" => ProviderType::UltraThink,
                                _ => ProviderType::Claude,
                            });
                            let ptype = form_provider_type();
                            form_base_url.set(ptype.default_base_url().to_string());
                            form_model.set(ptype.default_model().to_string());
                        },
                    }
                }
                TextField {
                    label: "Display Name".to_string(),
                    icon: "📝".to_string(),
                    value: form_name(),
                    placeholder: "My Claude Instance".to_string(),
                    oninput: move |e: FormEvent| form_name.set(e.value()),
                }
                TextField {
                    label: "API Key".to_string(),
                    icon: "🔑".to_string(),
                    value: form_api_key(),
                    placeholder: "sk-ant-...".to_string(),
                    helper: "(required for requests)".to_string(),
                    input_type: "password".to_string(),
                    oninput: move |e: FormEvent| form_api_key.set(e.value()),
                }
            }
            AdvancedSection {
                div {
                    class: "grid grid-cols-2 gap-4",
                    TextField {
                        label: "Base URL".to_string(),
                        icon: "🌐".to_string(),
                        value: form_base_url(),
                        placeholder: "Auto-filled".to_string(),
                        class: "text-text-secondary".to_string(),
                        oninput: move |e: FormEvent| form_base_url.set(e.value()),
                    }
                    TextField {
                        label: "Model".to_string(),
                        icon: "⚙️".to_string(),
                        value: form_model(),
                        placeholder: "Auto-filled".to_string(),
                        class: "text-text-secondary".to_string(),
                        oninput: move |e: FormEvent| form_model.set(e.value()),
                    }
                }
            }
            ModalFooter {
                CancelButton {
                    onclick: onclose,
                    "Cancel"
                }
                PrimaryButton {
                    onclick: move |_| {
                        let provider = ProviderConfig {
                            id: if !form_id().is_empty() { form_id() } else { format!("{:?}", form_provider_type()).to_lowercase() },
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
                        onsave.call(provider);
                    },
                    "💾 Save Provider"
                }
            }
        }
    }
}
