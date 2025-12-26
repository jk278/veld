//! Settings page component
//! 设置页面组件 - 统一配置入口（左侧导航+右侧内容）

use dioxus::prelude::*;
use crate::theme::use_theme;
use crate::config::{AppConfig, ProviderConfig, ProviderType, McpServerConfig, ThemeMode};

/// Settings tab
/// 设置标签页
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SettingsTab {
    AI,
    MCP,
    Appearance,
    Shortcuts,
}

/// Settings page component
/// 统一设置页面
#[component]
pub fn Settings() -> Element {
    let mut theme_mode = use_theme();
    let mut active_tab = use_signal(|| SettingsTab::AI);

    // Load config
    let mut providers = use_signal(|| {
        AppConfig::load()
            .map(|c| c.ai.providers)
            .unwrap_or_default()
    });

    let mut mcp_servers = use_signal(|| {
        AppConfig::load()
            .map(|c| c.mcp.servers)
            .unwrap_or_default()
    });

    // Form states for AI providers
    let mut editing_provider = use_signal(|| Option::<String>::None);
    let mut form_id = use_signal(|| String::new());
    let mut form_name = use_signal(|| String::new());
    let mut form_provider_type = use_signal(|| ProviderType::Claude);
    let mut form_api_key = use_signal(|| String::new());
    let mut form_base_url = use_signal(|| String::new());
    let mut form_model = use_signal(|| String::new());

    // Form states for MCP servers
    let mut editing_server = use_signal(|| Option::<String>::None);
    let mut server_form_name = use_signal(|| String::new());
    let mut server_form_command = use_signal(|| String::new());
    let mut server_form_args = use_signal(|| String::new());

    let providers_list = providers();
    let servers_list = mcp_servers();

    // Helper to check if adding mode
    let is_adding_mode = move || editing_provider().as_ref().map_or(false, |id| id.is_empty());
    let is_editing = move || editing_provider().as_ref().map_or(false, |id| !id.is_empty());
    let server_is_adding = move || editing_server().as_ref().map_or(false, |id| id.is_empty());

    // Helper to check if a provider is usable
    let is_provider_usable = |provider: &ProviderConfig| -> bool {
        provider.enabled && provider.api_key.as_ref().map_or(false, |k| !k.is_empty())
    };

    // Helper to format provider type for display
    let format_provider_type = |ptype: &ProviderType| -> String {
        format!("{:?}", ptype)
    };

    // Helper to join args for display
    let join_args = |args: &[String]| -> String {
        args.join(" ")
    };

    rsx! {
        div {
            class: "flex gap-6 h-full",

            // Left sidebar - Navigation
            div {
                class: "w-48 shrink-0 flex flex-col gap-1 border-r border-border pr-6",
                // AI Providers nav item
                button {
                    class: if active_tab() == SettingsTab::AI {
                        "w-full text-left px-3 py-2 rounded-lg bg-bg-surface text-primary font-medium transition-colors"
                    } else {
                        "w-full text-left px-3 py-2 rounded-lg text-text-secondary hover:bg-bg-surface hover:text-text-primary transition-colors"
                    },
                    onclick: move |_| active_tab.set(SettingsTab::AI),
                    span { class: "mr-2", "🤖" }
                    "AI Providers"
                }
                // MCP Servers nav item
                button {
                    class: if active_tab() == SettingsTab::MCP {
                        "w-full text-left px-3 py-2 rounded-lg bg-bg-surface text-primary font-medium transition-colors"
                    } else {
                        "w-full text-left px-3 py-2 rounded-lg text-text-secondary hover:bg-bg-surface hover:text-text-primary transition-colors"
                    },
                    onclick: move |_| active_tab.set(SettingsTab::MCP),
                    span { class: "mr-2", "⚡" }
                    "MCP Servers"
                }
                // Appearance nav item
                button {
                    class: if active_tab() == SettingsTab::Appearance {
                        "w-full text-left px-3 py-2 rounded-lg bg-bg-surface text-primary font-medium transition-colors"
                    } else {
                        "w-full text-left px-3 py-2 rounded-lg text-text-secondary hover:bg-bg-surface hover:text-text-primary transition-colors"
                    },
                    onclick: move |_| active_tab.set(SettingsTab::Appearance),
                    span { class: "mr-2", "🎨" }
                    "Appearance"
                }
                // Shortcuts nav item
                button {
                    class: if active_tab() == SettingsTab::Shortcuts {
                        "w-full text-left px-3 py-2 rounded-lg bg-bg-surface text-primary font-medium transition-colors"
                    } else {
                        "w-full text-left px-3 py-2 rounded-lg text-text-secondary hover:bg-bg-surface hover:text-text-primary transition-colors"
                    },
                    onclick: move |_| active_tab.set(SettingsTab::Shortcuts),
                    span { class: "mr-2", "⌨️" }
                    "Shortcuts"
                }
            }

            // Right side - Content area
            div {
                class: "flex-1 max-w-5xl overflow-y-auto",

                // AI Providers tab
                if active_tab() == SettingsTab::AI {
                    div {
                        class: "space-y-6",
                        h1 {
                            class: "text-2xl font-semibold text-text-primary",
                            "AI Providers"
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
                                    "All providers must use the Anthropic Claude API format (Messages API)."
                                }
                            }
                        }

                        // Add new provider button
                        div {
                            class: "flex justify-end mb-4",
                            button {
                                class: "btn-primary",
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
                                            span {
                                                class: "px-2 py-0.5 text-xs bg-bg-secondary text-text-secondary rounded font-mono",
                                                "{format_provider_type(&provider.provider_type)}"
                                            }
                                            if is_provider_usable(provider) {
                                                span {
                                                    class: "px-2 py-0.5 text-xs bg-success text-white rounded font-mono",
                                                    "✓ Ready"
                                                }
                                            } else if provider.enabled {
                                                span {
                                                    class: "px-2 py-0.5 text-xs bg-warning text-white rounded font-mono",
                                                    "⚠️ Missing Key"
                                                }
                                            } else {
                                                span {
                                                    class: "px-2 py-0.5 text-xs bg-bg-secondary text-text-muted rounded font-mono",
                                                    "Disabled"
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
                                                    let mut providers = providers.clone();
                                                    move |e| {
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
                                            span { "Enabled" }
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
                                            class: "px-3 py-1 text-sm bg-bg-secondary text-text-secondary rounded border border-border hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20 transition-colors",
                                            onclick: {
                                                let pid = provider.id.clone();
                                                let mut providers = providers.clone();
                                                move |_| {
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

                        // Edit/Add form (modal-like)
                        if editing_provider().is_some() {
                            div {
                                class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
                                onclick: move |_| editing_provider.set(None),

                                div {
                                    class: "bg-bg-surface border border-border rounded-xl p-6 max-w-2xl w-full shadow-2xl animate-in fade-in zoom-in duration-200",
                                    onclick: move |e: MouseEvent| e.stop_propagation(),

                                    // Header
                                    div {
                                        class: "flex items-center gap-3 mb-6 pb-4 border-b border-border",
                                        div {
                                            class: "w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center",
                                            span {
                                                class: "text-xl",
                                                if is_adding_mode() { "➕" } else { "✏️" }
                                            }
                                        }
                                        div {
                                            class: "flex-1",
                                            h3 {
                                                class: "text-xl font-semibold text-text-primary",
                                                if is_adding_mode() { "Add New Provider" } else { "Edit Provider" }
                                            }
                                            p {
                                                class: "text-sm text-text-secondary mt-1",
                                                if is_adding_mode() { "Configure a new AI provider" } else { "Update the provider configuration below" }
                                            }
                                        }
                                        button {
                                            class: "w-8 h-8 rounded-md bg-bg-primary text-text-secondary hover:text-text-primary hover:bg-bg-secondary flex items-center justify-center transition-colors",
                                            onclick: move |_| editing_provider.set(None),
                                            "×"
                                        }
                                    }

                                    // Form fields
                                    div {
                                        class: "space-y-5",
                                        // Provider Type
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
                                                placeholder: "Claude, Kimi, MiniMax, GLM, UltraThink...",
                                                value: format!("{:?}", form_provider_type()),
                                                oninput: move |e| {
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

                                        // API Key
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

                                        // Advanced settings
                                        div {
                                            class: "space-y-4 pt-4 border-t border-border/50",
                                            p {
                                                class: "text-xs font-semibold text-text-secondary uppercase tracking-wider mb-3",
                                                "Advanced Settings"
                                            }
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

                                    // Form actions
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

                // MCP Servers tab
                else if active_tab() == SettingsTab::MCP {
                    div {
                        class: "space-y-6",
                        h1 {
                            class: "text-2xl font-semibold text-text-primary",
                            "MCP Servers"
                        }

                        // Add new server button
                        div {
                            class: "flex justify-end mb-4",
                            button {
                                class: "btn-primary",
                                onclick: move |_| {
                                    editing_server.set(Some(String::new()));
                                    server_form_name.set(String::new());
                                    server_form_command.set(String::new());
                                    server_form_args.set(String::new());
                                },
                                "＋ Add Server"
                            }
                        }

                        // Servers list
                        div {
                            class: "space-y-3",
                            for server in servers_list.iter() {
                                div {
                                    class: "bg-bg-surface border border-border rounded-lg p-4 space-y-3",
                                    div {
                                        class: "flex items-center justify-between",
                                        h3 {
                                            class: "text-lg font-medium text-text-primary",
                                            "{server.name}"
                                        }
                                        div {
                                            class: "flex gap-2",
                                            button {
                                                class: "px-2 py-1 text-xs bg-bg-secondary text-text-secondary rounded border border-border hover:text-text-primary transition-colors",
                                                onclick: {
                                                    let sname = server.name.clone();
                                                    move |_| {
                                                        editing_server.set(Some(sname.clone()));
                                                    }
                                                },
                                                "Edit"
                                            }
                                            button {
                                                class: "px-2 py-1 text-xs bg-bg-secondary text-text-secondary rounded border border-border hover:text-error transition-colors",
                                                onclick: {
                                                    let sname = server.name.clone();
                                                    let mut servers = mcp_servers.clone();
                                                    move |_| {
                                                        let name = sname.clone();
                                                        if let Ok(mut config) = AppConfig::load() {
                                                            config.mcp.servers.retain(|s| s.name != name);
                                                            let _ = config.save();
                                                            servers.set(config.mcp.servers.clone());
                                                        }
                                                    }
                                                },
                                                "Delete"
                                            }
                                        }
                                    }

                                    div {
                                        class: "grid grid-cols-2 gap-4 text-sm",
                                        div {
                                            class: "text-text-muted",
                                            "Command: {server.command}"
                                        }
                                        if !server.args.is_empty() {
                                            div {
                                                class: "text-text-muted",
                                                "Args: {join_args(&server.args)}"
                                            }
                                        }
                                    }

                                    div {
                                        class: "flex items-center gap-2",
                                        label {
                                            class: "flex items-center gap-2 cursor-pointer text-text-secondary hover:text-text-primary transition-colors",
                                            input {
                                                r#type: "checkbox",
                                                checked: server.enabled,
                                                onchange: {
                                                    let sname = server.name.clone();
                                                    let mut servers = mcp_servers.clone();
                                                    move |e| {
                                                        if let Ok(mut config) = AppConfig::load() {
                                                            if let Some(s) = config.mcp.servers.iter_mut().find(|s| s.name == sname) {
                                                                s.enabled = e.checked();
                                                            }
                                                            let _ = config.save();
                                                            servers.set(config.mcp.servers.clone());
                                                        }
                                                    }
                                                },
                                                class: "w-4 h-4 text-primary bg-bg-surface border-border rounded focus:ring-primary focus:ring-2"
                                            }
                                            span {
                                                class: "text-sm",
                                                "Enabled"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Edit/Add form
                        if editing_server().is_some() {
                            div {
                                class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                                div {
                                    class: "bg-bg-surface border border-border rounded-lg p-6 max-w-md w-full",
                                    h2 {
                                        class: "text-xl font-semibold text-text-primary mb-4",
                                        if server_is_adding() { "Add Server" } else { "Edit Server" }
                                    }

                                    div {
                                        class: "space-y-4",
                                        div {
                                            class: "space-y-2",
                                            label {
                                                class: "block text-text-secondary text-sm font-medium",
                                                "Name"
                                            }
                                            input {
                                                class: "input-field",
                                                placeholder: "My Server",
                                                value: server_form_name(),
                                                oninput: move |e| server_form_name.set(e.value())
                                            }
                                        }

                                        div {
                                            class: "space-y-2",
                                            label {
                                                class: "block text-text-secondary text-sm font-medium",
                                                "Command"
                                            }
                                            input {
                                                class: "input-field",
                                                placeholder: "npx",
                                                value: server_form_command(),
                                                oninput: move |e| server_form_command.set(e.value())
                                            }
                                        }

                                        div {
                                            class: "space-y-2",
                                            label {
                                                class: "block text-text-secondary text-sm font-medium",
                                                "Args (one per line)"
                                            }
                                            textarea {
                                                class: "input-field",
                                                rows: 3,
                                                placeholder: "arg1\narg2",
                                                value: server_form_args(),
                                                oninput: move |e| server_form_args.set(e.value())
                                            }
                                        }
                                    }

                                    div {
                                        class: "flex justify-end gap-3 mt-6",
                                        button {
                                            class: "btn-cancel",
                                            onclick: move |_| editing_server.set(None),
                                            "Cancel"
                                        }
                                        button {
                                            class: "btn-primary",
                                            onclick: move |_| {
                                                let args: Vec<String> = server_form_args().lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
                                                let new_server = McpServerConfig {
                                                    name: if server_form_name().is_empty() { "New Server".to_string() } else { server_form_name() },
                                                    command: server_form_command(),
                                                    args,
                                                    env: None,
                                                    enabled: true,
                                                };

                                                if let Ok(mut config) = AppConfig::load() {
                                                    if server_is_adding() {
                                                        config.mcp.servers.push(new_server);
                                                    } else if let Some(s) = config.mcp.servers.iter_mut().find(|s| s.name == editing_server().unwrap_or_default()) {
                                                        *s = new_server;
                                                    }
                                                    let _ = config.save();
                                                    mcp_servers.set(config.mcp.servers.clone());
                                                }

                                                editing_server.set(None);
                                            },
                                            "Save"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Appearance tab
                else if active_tab() == SettingsTab::Appearance {
                    div {
                        class: "space-y-6",
                        h1 {
                            class: "text-2xl font-semibold text-text-primary",
                            "Appearance"
                        }

                        section {
                            class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
                            h2 {
                                class: "text-lg text-text-primary mb-4",
                                "Theme"
                            }
                            div {
                                class: "flex flex-wrap gap-2 items-center",
                                button {
                                    class: if theme_mode() == ThemeMode::Light {
                                        "px-4 py-2 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                                    } else {
                                        "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border hover:bg-bg-secondary"
                                    },
                                    onclick: move |_| theme_mode.set(ThemeMode::Light),
                                    "☀️ Light"
                                }
                                button {
                                    class: if theme_mode() == ThemeMode::Dark {
                                        "px-4 py-2 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                                    } else {
                                        "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border hover:bg-bg-secondary"
                                    },
                                    onclick: move |_| theme_mode.set(ThemeMode::Dark),
                                    "🌙 Dark"
                                }
                                button {
                                    class: if theme_mode() == ThemeMode::System {
                                        "px-4 py-2 rounded font-mono text-sm transition-all bg-primary text-white border border-border"
                                    } else {
                                        "px-4 py-2 rounded font-mono text-sm transition-all bg-bg-surface text-text-primary border border-border hover:bg-bg-secondary"
                                    },
                                    onclick: move |_| theme_mode.set(ThemeMode::System),
                                    "🖥️ System"
                                }
                            }
                        }
                    }
                }

                // Shortcuts tab
                else if active_tab() == SettingsTab::Shortcuts {
                    div {
                        class: "space-y-6",
                        h1 {
                            class: "text-2xl font-semibold text-text-primary",
                            "Keyboard Shortcuts"
                        }

                        section {
                            class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",
                            h2 {
                                class: "text-lg text-text-primary mb-4",
                                "Global Shortcuts"
                            }
                            div {
                                class: "space-y-3",
                                div {
                                    class: "flex justify-between items-center py-2 border-b border-border last:border-b-0",
                                    span {
                                        class: "text-text-secondary",
                                        "Show Floating Input"
                                    }
                                    code {
                                        class: "px-3 py-1 bg-bg-primary text-primary rounded font-mono text-sm",
                                        "Ctrl+Shift+Space"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
