//! MCP Servers tab component
//! MCP 服务器配置标签页

use dioxus::prelude::*;
use crate::config::{AppConfig, McpServerConfig};
use crate::components::ui::*;

/// MCP Servers tab content
#[component]
pub fn McpServersTab(
    mut mcp_servers: Signal<Vec<McpServerConfig>>,
    mut editing_server: Signal<Option<String>>,
    // Form state
    mut server_form_name: Signal<String>,
    mut server_form_command: Signal<String>,
    mut server_form_args: Signal<String>,
) -> Element {
    let servers_list = mcp_servers();
    let server_is_adding = move || editing_server().as_ref().map_or(false, |id| id.is_empty());

    // Helper to join args for display
    let join_args = |args: &[String]| -> String {
        args.join(" ")
    };

    rsx! {
        div {
            class: "space-y-6",
            h1 {
                class: "text-2xl font-semibold text-text-primary",
                "MCP Servers"
            }

            // Add new server button
            div {
                class: "flex justify-end mb-4",
                PrimaryButton {
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
                                SecondaryButton {
                                    class: "px-2 py-1 text-xs".to_string(),
                                    onclick: {
                                        let sname = server.name.clone();
                                        move |_| {
                                            editing_server.set(Some(sname.clone()));
                                        }
                                    },
                                    "Edit"
                                }
                                Button {
                                    variant: ButtonVariant::Cancel,
                                    class: "px-2 py-1 text-xs".to_string(),
                                    onclick: {
                                        let sname = server.name.clone();
                                        let mut servers = mcp_servers.clone();
                                        move |_| {
                                            let name = sname.clone();
                                            if let Ok(mut config) = AppConfig::load() {
                                                config.mcp.servers.retain(|s| s.name != name);
                                                if let Err(e) = config.save() {
                                                    eprintln!("[Settings] Failed to save server deletion: {}", e);
                                                }
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
                                                if let Err(err) = config.save() {
                                                    eprintln!("[Settings] Failed to save server toggle: {}", err);
                                                }
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

            // Edit/Add modal
            Modal {
                show: editing_server().is_some(),
                onclose: move |_| editing_server.set(None),
                max_width: "28rem".to_string(),
                ModalHeader {
                    title: (if server_is_adding() { "Add Server" } else { "Edit Server" }).to_string(),
                    show_close: true,
                    onclose: move |_| editing_server.set(None),
                }
                ModalContent {
                    TextField {
                        label: "Name".to_string(),
                        value: server_form_name(),
                        placeholder: "My Server".to_string(),
                        oninput: move |e: FormEvent| server_form_name.set(e.value()),
                    }
                    TextField {
                        label: "Command".to_string(),
                        value: server_form_command(),
                        placeholder: "npx".to_string(),
                        oninput: move |e: FormEvent| server_form_command.set(e.value()),
                    }
                    TextArea {
                        label: "Args (one per line)".to_string(),
                        value: server_form_args(),
                        rows: 3,
                        placeholder: "arg1\narg2".to_string(),
                        oninput: move |e: FormEvent| server_form_args.set(e.value()),
                    }
                }
                ModalFooter {
                    CancelButton {
                        onclick: move |_| editing_server.set(None),
                        "Cancel"
                    }
                    PrimaryButton {
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
                                if let Err(e) = config.save() {
                                    eprintln!("[Settings] Failed to save server update: {}", e);
                                }
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
