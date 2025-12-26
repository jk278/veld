//! Settings page component - Refactored with UI library
//! 设置页面组件 - 使用 UI 组件库重构

use dioxus::prelude::*;
use crate::config::{AppConfig, ProviderConfig, ProviderType, McpServerConfig};
use crate::components::ui::*;
use crate::components::settings_tabs::{AiProvidersTab, McpServersTab, AppearanceTab, ShortcutsTab};

/// Settings tab
/// 设置标签页
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    AI,
    MCP,
    Appearance,
    Shortcuts,
}

impl SettingsTab {
    fn as_str(&self) -> &'static str {
        match self {
            SettingsTab::AI => "ai",
            SettingsTab::MCP => "mcp",
            SettingsTab::Appearance => "appearance",
            SettingsTab::Shortcuts => "shortcuts",
        }
    }
}

/// Settings page component - Unified settings interface with sidebar navigation
/// 统一设置页面
#[component]
pub fn Settings() -> Element {
    let mut active_tab = use_signal(|| SettingsTab::AI);

    // Load config
    let providers = use_signal(|| {
        AppConfig::load()
            .map(|c| c.ai.providers)
            .unwrap_or_default()
    });

    let mcp_servers = use_signal(|| {
        AppConfig::load()
            .map(|c| c.mcp.servers)
            .unwrap_or_default()
    });

    // Form states for AI providers
    let editing_provider = use_signal(|| Option::<String>::None);
    let form_id = use_signal(|| String::new());
    let form_name = use_signal(|| String::new());
    let form_provider_type = use_signal(|| ProviderType::Claude);
    let form_api_key = use_signal(|| String::new());
    let form_base_url = use_signal(|| String::new());
    let form_model = use_signal(|| String::new());

    // Form states for MCP servers
    let editing_server = use_signal(|| Option::<String>::None);
    let server_form_name = use_signal(|| String::new());
    let server_form_command = use_signal(|| String::new());
    let server_form_args = use_signal(|| String::new());

    rsx! {
        div {
            class: "flex gap-6 h-full",

            // Left sidebar - Navigation
            TabList {
                width: "12rem".to_string(),
                NavTab {
                    label: "AI Providers".to_string(),
                    value: "ai".to_string(),
                    active_value: active_tab().as_str().to_string(),
                    icon: "🤖".to_string(),
                    onclick: move |_| active_tab.set(SettingsTab::AI),
                }
                NavTab {
                    label: "MCP Servers".to_string(),
                    value: "mcp".to_string(),
                    active_value: active_tab().as_str().to_string(),
                    icon: "⚡".to_string(),
                    onclick: move |_| active_tab.set(SettingsTab::MCP),
                }
                NavTab {
                    label: "Appearance".to_string(),
                    value: "appearance".to_string(),
                    active_value: active_tab().as_str().to_string(),
                    icon: "🎨".to_string(),
                    onclick: move |_| active_tab.set(SettingsTab::Appearance),
                }
                NavTab {
                    label: "Shortcuts".to_string(),
                    value: "shortcuts".to_string(),
                    active_value: active_tab().as_str().to_string(),
                    icon: "⌨️".to_string(),
                    onclick: move |_| active_tab.set(SettingsTab::Shortcuts),
                }
            }

            // Right side - Content area
            TabPanel {
                {render_active_tab(
                    active_tab(),
                    providers.clone(),
                    mcp_servers.clone(),
                    editing_provider.clone(),
                    form_id.clone(),
                    form_name.clone(),
                    form_provider_type.clone(),
                    form_api_key.clone(),
                    form_base_url.clone(),
                    form_model.clone(),
                    editing_server.clone(),
                    server_form_name.clone(),
                    server_form_command.clone(),
                    server_form_args.clone(),
                )}
            }
        }
    }
}

/// Render the active tab content
#[allow(clippy::too_many_arguments)]
fn render_active_tab(
    active_tab: SettingsTab,
    providers: Signal<Vec<ProviderConfig>>,
    mcp_servers: Signal<Vec<McpServerConfig>>,
    editing_provider: Signal<Option<String>>,
    form_id: Signal<String>,
    form_name: Signal<String>,
    form_provider_type: Signal<ProviderType>,
    form_api_key: Signal<String>,
    form_base_url: Signal<String>,
    form_model: Signal<String>,
    editing_server: Signal<Option<String>>,
    server_form_name: Signal<String>,
    server_form_command: Signal<String>,
    server_form_args: Signal<String>,
) -> Element {
    match active_tab {
        SettingsTab::AI => rsx! {
            AiProvidersTab {
                providers: providers.clone(),
                editing_provider: editing_provider.clone(),
                form_id: form_id.clone(),
                form_name: form_name.clone(),
                form_provider_type: form_provider_type.clone(),
                form_api_key: form_api_key.clone(),
                form_base_url: form_base_url.clone(),
                form_model: form_model.clone(),
            }
        },
        SettingsTab::MCP => rsx! {
            McpServersTab {
                mcp_servers: mcp_servers.clone(),
                editing_server: editing_server.clone(),
                server_form_name: server_form_name.clone(),
                server_form_command: server_form_command.clone(),
                server_form_args: server_form_args.clone(),
            }
        },
        SettingsTab::Appearance => rsx! {
            AppearanceTab {}
        },
        SettingsTab::Shortcuts => rsx! {
            ShortcutsTab {}
        },
    }
}
