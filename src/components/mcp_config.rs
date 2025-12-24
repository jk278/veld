//! MCP (Model Context Protocol) Servers Configuration page
//! MCP 服务器配置页面

use dioxus::prelude::*;
use crate::theme::use_theme;
use crate::config::{AppConfig, McpServerConfig};

/// MCP Configuration page
/// MCP 服务器配置页面
#[component]
pub fn McpConfig() -> Element {
    let _theme_mode = use_theme();

    // Load current config
    let mut servers = use_signal(|| {
        AppConfig::load()
            .map(|c| c.mcp.servers)
            .unwrap_or_default()
    });

    // Track which server is being edited
    let mut editing_server = use_signal(|| Option::<String>::None);

    // Form state for new/editing server
    let mut form_name = use_signal(|| String::new());
    let mut form_command = use_signal(|| String::new());
    let mut form_args = use_signal(|| String::new());
    let mut form_env = use_signal(|| String::new());

    // Check if we're editing an existing server or adding a new one
    let is_adding_mode = move || editing_server().as_ref().map_or(false, |id| id.is_empty());

    // Collect servers for rendering to avoid borrow issues
    let servers_list = servers();

    rsx! {
        div {
            class: "max-w-5xl mx-auto space-y-6",

            h1 {
                class: "text-3xl font-light text-text-primary mb-8",
                "MCP Servers Configuration"
            }

            // MCP info notice
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
                        "Model Context Protocol (MCP)"
                    }
                    p {
                        class: "text-xs text-text-secondary leading-relaxed",
                        "Configure MCP servers to extend AI capabilities with tools like filesystem access, web search, Git operations, and more. Servers run as local processes and communicate via stdio."
                    }
                }
            }

            // Server list
            section {
                class: "bg-bg-surface border border-border rounded-lg p-6 space-y-4",

                div {
                    class: "flex justify-between items-center mb-4",
                    h2 {
                        class: "text-xl text-text-primary",
                        "📋 Server List"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            editing_server.set(Some(String::new()));
                            form_name.set(String::new());
                            form_command.set("npx".to_string());
                            form_args.set(String::new());
                            form_env.set(String::new());
                        },
                        "➕ Add Server"
                    }
                }

                div {
                    class: "space-y-3",
                    if servers_list.is_empty() {
                        p {
                            class: "text-text-secondary text-sm text-center py-4",
                            "No MCP servers configured"
                        }
                    } else {
                        for server in servers_list.iter() {
                            div {
                                class: "flex items-center justify-between p-4 bg-bg-primary border border-border rounded-md hover:border-primary transition-colors",

                                div {
                                    class: "flex-1",

                                    div {
                                        class: "flex items-center gap-3 mb-2",
                                        span {
                                            class: "font-mono font-medium text-text-primary",
                                            {server.name.clone()}
                                        }
                                        if server.enabled {
                                            span {
                                                class: "px-2 py-0.5 text-xs bg-success/10 text-success border border-success/30 rounded font-mono",
                                                "Enabled"
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
                                        span {
                                            class: "font-mono text-xs",
                                            "Command: {server.command}"
                                        }
                                        if !server.args.is_empty() {
                                            span {
                                                class: "font-mono text-xs",
                                                "Args: {server.args.join(\" \")}"
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
                                            checked: server.enabled,
                                            oninput: {
                                                let sname = server.name.clone();
                                                move |e| {
                                                    let sname = sname.clone();
                                                    if let Ok(mut config) = AppConfig::load() {
                                                        if let Some(sr) = config.mcp.servers.iter_mut().find(|s| s.name == sname) {
                                                            sr.enabled = e.checked();
                                                        }
                                                        let _ = config.save();
                                                        servers.set(config.mcp.servers.clone());
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
                                            let s = server.clone();
                                            move |_| {
                                                editing_server.set(Some(s.name.clone()));
                                                form_name.set(s.name.clone());
                                                form_command.set(s.command.clone());
                                                form_args.set(s.args.join(" "));
                                                form_env.set(s.env.as_ref()
                                                    .map(|m| m.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("\n"))
                                                    .unwrap_or_default());
                                            }
                                        },
                                        "Edit"
                                    }

                                    button {
                                        class: "px-3 py-1 text-sm bg-bg-secondary text-text-secondary rounded border border-border hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20 transition-colors",
                                        onclick: {
                                            let sname = server.name.clone();
                                            move |_| {
                                                let sname = sname.clone();
                                                if let Ok(mut config) = AppConfig::load() {
                                                    config.mcp.servers.retain(|s| s.name != sname);
                                                    let _ = config.save();
                                                    servers.set(config.mcp.servers.clone());
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
            }

            // Edit/Add form modal
            if editing_server().is_some() {
                div {
                    class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
                    onclick: move |_| editing_server.set(None),

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
                                    {if is_adding_mode() { "➕" } else { "✏️" }}
                                }
                            }

                            div {
                                class: "flex-1",
                                h3 {
                                    class: "text-xl font-semibold text-text-primary",
                                    {if is_adding_mode() { "Add New Server" } else { "Edit Server" }}
                                }
                                p {
                                    class: "text-sm text-text-secondary mt-1",
                                    {if is_adding_mode() { "Configure a new MCP server" } else { "Update the server configuration below" }}
                                }
                            }

                            button {
                                class: "w-8 h-8 rounded-md bg-bg-primary text-text-secondary hover:text-text-primary hover:bg-bg-secondary flex items-center justify-center transition-colors",
                                onclick: move |_| editing_server.set(None),
                                "×"
                            }
                        }

                        // Form fields
                        div {
                            class: "space-y-5",

                            // Display Name
                            div {
                                class: "space-y-2",
                                label {
                                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                                    span { "📝" }
                                    "Name"
                                }
                                input {
                                    class: "w-full p-2.5 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all",
                                    r#type: "text",
                                    placeholder: "Context7",
                                    value: form_name(),
                                    oninput: move |e| form_name.set(e.value()),
                                }
                                p {
                                    class: "text-xs text-text-muted",
                                    "Server name (ID will be auto-generated)"
                                }
                            }

                            // Command
                            div {
                                class: "space-y-2",
                                label {
                                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                                    span { "⚡" }
                                    "Command"
                                }
                                input {
                                    class: "w-full p-2.5 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all",
                                    r#type: "text",
                                    placeholder: "npx or /path/to/executable",
                                    value: form_command(),
                                    oninput: move |e| form_command.set(e.value()),
                                }
                            }

                            // Arguments
                            div {
                                class: "space-y-2",
                                label {
                                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                                    span { "📋" }
                                    "Arguments"
                                }
                                input {
                                    class: "w-full p-2.5 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all",
                                    r#type: "text",
                                    placeholder: "arg1 arg2 arg3 (space-separated)",
                                    value: form_args(),
                                    oninput: move |e| form_args.set(e.value()),
                                }
                                p {
                                    class: "text-xs text-text-muted",
                                    "Space-separated arguments passed to the command"
                                }
                            }

                            // Environment Variables
                            div {
                                class: "space-y-2",
                                label {
                                    class: "flex items-center gap-2 text-sm font-medium text-text-primary",
                                    span { "🔐" }
                                    "Environment Variables"
                                }
                                textarea {
                                    class: "w-full p-2.5 bg-bg-primary text-text-primary border border-border rounded-lg font-mono text-sm focus:border-primary focus:ring-2 focus:ring-primary/20 outline-none transition-all resize-none",
                                    rows: 3,
                                    placeholder: "KEY=value\nANOTHER_KEY=value",
                                    value: form_env(),
                                    oninput: move |e| form_env.set(e.value()),
                                }
                                p {
                                    class: "text-xs text-text-muted",
                                    "One KEY=value pair per line (optional)"
                                }
                            }
                        }

                        // Form actions
                        div {
                            class: "flex justify-end gap-3 mt-6 pt-5 border-t border-border",

                            button {
                                class: "px-5 py-2.5 rounded-lg bg-bg-primary text-text-secondary border border-border font-medium hover:bg-bg-secondary hover:text-text-primary transition-all",
                                onclick: move |_| editing_server.set(None),
                                "Cancel"
                            }
                            button {
                                class: "px-5 py-2.5 rounded-lg bg-primary text-white font-medium hover:bg-primary/90 transition-all shadow-lg shadow-primary/25 flex items-center gap-2",
                                onclick: move |_| {
                                    // Parse args from space-separated string
                                    let args_vec = if form_args().trim().is_empty() {
                                        vec![]
                                    } else {
                                        form_args().split_whitespace().map(|s| s.to_string()).collect()
                                    };

                                    // Parse env from KEY=value lines
                                    let env_map = if form_env().trim().is_empty() {
                                        None
                                    } else {
                                        let mut map = std::collections::HashMap::new();
                                        for line in form_env().lines() {
                                            if let Some((key, value)) = line.split_once('=') {
                                                map.insert(key.trim().to_string(), value.trim().to_string());
                                            }
                                        }
                                        if map.is_empty() { None } else { Some(map) }
                                    };

                                    let server = McpServerConfig {
                                        name: form_name(),
                                        command: if form_command().is_empty() { "npx".to_string() } else { form_command() },
                                        args: args_vec,
                                        env: env_map,
                                        enabled: true,
                                    };

                                    if let Ok(mut config) = AppConfig::load() {
                                        config.update_mcp_server(server);
                                        servers.set(config.mcp.servers.clone());
                                    }
                                    editing_server.set(None);
                                },
                                span { "💾" }
                                "Save Server"
                            }
                        }
                    }
                }
            }
        }
    }
}
