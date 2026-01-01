//! AI Providers tab component
//! AI 提供商配置标签页

use crate::components::ui::*;
use crate::config::{AppConfig, ProviderConfig};
use dioxus::prelude::*;

/// AI Providers tab content
#[component]
pub fn AiProvidersTab(
  mut providers: Signal<Vec<ProviderConfig>>,
  mut editing_provider: Signal<Option<String>>,
  mut form_id: Signal<String>,
  mut form_name: Signal<String>,
  mut form_model: Signal<String>,
  mut form_api_key: Signal<String>,
  mut form_adapter_type: Signal<Option<String>>,
  mut form_base_url: Signal<String>,
) -> Element {
  let providers_list = providers();
  let is_adding_mode = move || { editing_provider().as_ref().map_or(false, |id| id.is_empty()) };

  rsx! {
    div {
      class: "space-y-6",
      // Header with title and add button
      div {
        class: "flex items-center justify-between",
        h1 {
          class: "text-2xl font-semibold text-text-primary",
          "AI Providers"
        }
        PrimaryButton {
          onclick: move |_| {
              editing_provider.set(Some(String::new()));
              form_id.set(String::new());
              form_name.set(String::new());
              form_model.set("gpt-4o-mini".to_string());
              form_api_key.set(String::new());
              form_adapter_type.set(Some("openai".to_string()));
              form_base_url.set(String::new());
          },
          "＋ Add"
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
                let pmodel = provider.model.clone();
                let papi_key = provider.api_key.clone().unwrap_or_default();
                let padapter_type = provider.adapter_type.clone();
                let pbase_url = provider.base_url.clone().unwrap_or_default();
                move |_| {
                    editing_provider.set(Some(pid.clone()));
                    form_id.set(pid.clone());
                    form_name.set(pname.clone());
                    form_model.set(pmodel.clone());
                    form_api_key.set(papi_key.clone());
                    form_adapter_type.set(padapter_type.clone());
                    form_base_url.set(pbase_url.clone());
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
        form_model: form_model.clone(),
        form_api_key: form_api_key.clone(),
        form_adapter_type: form_adapter_type.clone(),
        form_base_url: form_base_url.clone(),
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
      class: "flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 bg-bg-surface border border-border rounded-md hover:border-primary transition-colors",
      div {
        class: "flex-1 min-w-0",
        div {
          class: "flex flex-wrap items-center gap-2 mb-2",
          span {
            class: "font-mono font-medium text-text-primary",
            "{provider.name}"
          }
          StatusBadge {
            status: if is_usable { StatusType::Ready } else if provider.enabled { StatusType::Warning } else { StatusType::Disabled },
            text: if is_usable { "".to_string() } else if provider.enabled { "Missing Key".to_string() } else { "".to_string() },
            small: true,
          }
        }
        div {
          class: "flex flex-wrap gap-x-4 gap-y-1 text-sm text-text-secondary",
          span {
            class: "font-mono text-xs",
            "Model: {provider.model}"
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
        class: "flex flex-wrap items-center gap-2 sm:flex-nowrap",
        label {
          class: "flex items-center gap-2 cursor-pointer text-text-secondary hover:text-text-primary transition-colors text-sm",
          input {
            r#type: "checkbox",
            checked: provider.enabled,
            onchange: move |e| {
                if let Ok(mut config) = AppConfig::load() {
                    if let Some(pr) = config
                        .ai
                        .providers
                        .iter_mut()
                        .find(|p| p.id == provider.id)
                    {
                        pr.enabled = e.checked();
                    }
                    if let Err(err) = config.save() {
                        eprintln!("[Settings] Failed to save provider toggle: {}", err);
                    }
                }
            },
            class: "w-4 h-4 text-primary bg-bg-surface border-border rounded focus:ring-primary focus:ring-2",
          }
          span {
            "Enabled"
          }
        }
        PrimaryButton {
          class: "text-sm".to_string(),
          onclick: move |e| {
              if let Some(handler) = onedit {
                  handler.call(e);
              }
          },
          "Edit"
        }
        SecondaryButton {
          class: "text-sm text-error border border-error hover:bg-error hover:text-white".to_string(),
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
  form_model: Signal<String>,
  form_api_key: Signal<String>,
  form_adapter_type: Signal<Option<String>>,
  form_base_url: Signal<String>,
  onsave: EventHandler<ProviderConfig>,
) -> Element {
  rsx! {
    Modal {
      show,
      onclose,
      max_width: "40rem".to_string(),
      ModalHeader {
        title: (if is_adding_mode { "Add Provider" } else { "Edit Provider" }).to_string(),
        subtitle: (if is_adding_mode {
            "Configure AI provider"
        } else {
            "Update provider settings"
        })
            .to_string(),
        icon: (if is_adding_mode { "➕" } else { "✏️" }).to_string(),
        show_close: true,
        onclose,
      }
      ModalContent {
        div {
          class: "space-y-3",
          // Row 1: Protocol + Model
          div {
            class: "grid grid-cols-2 gap-3",
            // Adapter Type (segmented control)
            div {
              class: "space-y-1",
              label {
                class: "text-xs font-medium text-text-secondary",
                "Protocol"
              }
              // Segmented Control - responsive: horizontal on wide, stacked on narrow
              div {
                class: "grid grid-cols-3 gap-1 p-1 bg-bg-secondary rounded-lg border border-border sm:gap-0",
                // OpenAI
                button {
                  class: format!(
                    "px-2 py-1.5 text-xs font-medium rounded transition-all sm:px-3 sm:py-2 sm:text-sm {}",
                    if form_adapter_type().as_ref().map_or(false, |t| t == "openai") {
                      "bg-bg-surface text-text-primary shadow-sm"
                    } else {
                      "text-text-secondary hover:text-text-primary"
                    }
                  ),
                  onclick: move |_| form_adapter_type.set(Some("openai".to_string())),
                  type: "button",
                  span { class: "hidden sm:inline", "OpenAI" }
                  span { class: "sm:hidden", "OAI" }
                }
                // Anthropic
                button {
                  class: format!(
                    "px-2 py-1.5 text-xs font-medium rounded transition-all sm:px-3 sm:py-2 sm:text-sm {}",
                    if form_adapter_type().as_ref().map_or(false, |t| t == "anthropic") {
                      "bg-bg-surface text-text-primary shadow-sm"
                    } else {
                      "text-text-secondary hover:text-text-primary"
                    }
                  ),
                  onclick: move |_| form_adapter_type.set(Some("anthropic".to_string())),
                  type: "button",
                  span { class: "hidden sm:inline", "Anthropic" }
                  span { class: "sm:hidden", "Anth" }
                }
                // Auto
                button {
                  class: format!(
                    "px-2 py-1.5 text-xs font-medium rounded transition-all sm:px-3 sm:py-2 sm:text-sm {}",
                    if form_adapter_type().is_none() {
                      "bg-bg-surface text-text-primary shadow-sm"
                    } else {
                      "text-text-secondary hover:text-text-primary"
                    }
                  ),
                  onclick: move |_| form_adapter_type.set(None),
                  type: "button",
                  "Auto"
                }
              }
            }
            // Model Name
            div {
              class: "space-y-1",
              label {
                class: "text-xs font-medium text-text-secondary",
                "Model"
              }
              input {
                r#type: "text",
                value: form_model(),
                placeholder: "gpt-4o-mini",
                class: "w-full px-3 py-2 bg-bg-surface border border-border rounded-md text-sm text-text-primary placeholder:text-text-secondary/50 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary",
                oninput: move |e: FormEvent| form_model.set(e.value()),
              }
            }
          }
          // Row 2: Name + Key
          div {
            class: "grid grid-cols-2 gap-3",
            // Display Name
            div {
              class: "space-y-1",
              label {
                class: "text-xs font-medium text-text-secondary",
                "Display Name"
              }
              input {
                r#type: "text",
                value: form_name(),
                placeholder: "My AI Provider",
                class: "w-full px-3 py-2 bg-bg-surface border border-border rounded-md text-sm text-text-primary placeholder:text-text-secondary/50 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary",
                oninput: move |e: FormEvent| form_name.set(e.value()),
              }
            }
            // API Key
            div {
              class: "space-y-1",
              label {
                class: "text-xs font-medium text-text-secondary",
                "API Key"
              }
              input {
                r#type: "password",
                value: form_api_key(),
                placeholder: "sk-...",
                class: "w-full px-3 py-2 bg-bg-surface border border-border rounded-md text-sm text-text-primary placeholder:text-text-secondary/50 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary",
                oninput: move |e: FormEvent| form_api_key.set(e.value()),
              }
            }
          }
          // Row 3: Base URL (optional)
          div {
            class: "space-y-1",
            label {
              class: "text-xs font-medium text-text-secondary",
              "Base URL (optional)"
            }
            input {
              r#type: "text",
              value: form_base_url(),
              placeholder: "https://api.example.com/v1",
              class: "w-full px-3 py-2 bg-bg-surface border border-border rounded-md text-sm text-text-primary placeholder:text-text-secondary/50 focus:outline-none focus:ring-1 focus:ring-primary focus:border-primary",
              oninput: move |e: FormEvent| form_base_url.set(e.value()),
            }
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
                  id: if !form_id().is_empty() {
                      form_id()
                  } else {
                      form_model().replace('/', "-").to_lowercase()
                  },
                  name: if form_name().is_empty() {
                      form_model().clone()
                  } else {
                      form_name()
                  },
                  model: form_model(),
                  api_key: if form_api_key().is_empty() { None } else { Some(form_api_key()) },
                  adapter_type: form_adapter_type(),
                  base_url: if form_base_url().is_empty() { None } else { Some(form_base_url()) },
                  enabled: true,
              };
              onsave.call(provider);
          },
          "Save"
        }
      }
    }
  }
}
