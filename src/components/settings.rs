//! Settings page component - Refactored with UI library
//! 设置页面组件 - 使用 UI 组件库重构

use crate::components::icons::*;
use crate::components::settings_tabs::{
  AboutTab, AiProvidersTab, AppearanceTab, McpServersTab, QuickToolsTab, ShortcutsTab,
};
use crate::components::ui::*;
use crate::config::{AppConfig, McpServerConfig, ProviderConfig};
use crate::hooks::{RESPONSIVE_BREAKPOINT, use_window_size};
use dioxus::prelude::*;

/// Settings tab
/// 设置标签页
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
  AI,
  MCP,
  Appearance,
  Shortcuts,
  QuickTools,
  About,
}

impl SettingsTab {
  fn as_str(&self) -> &'static str {
    match self {
      SettingsTab::AI => "ai",
      SettingsTab::MCP => "mcp",
      SettingsTab::Appearance => "appearance",
      SettingsTab::Shortcuts => "shortcuts",
      SettingsTab::QuickTools => "quick_tools",
      SettingsTab::About => "about",
    }
  }
}

/// Settings page component - Unified settings interface with sidebar navigation
/// 统一设置页面
#[component]
pub fn Settings() -> Element {
  let mut active_tab = use_signal(|| SettingsTab::AI);
  let window_width = use_window_size();

  // Sidebar collapse state (persisted to config file)
  let mut nav_collapsed = use_signal(|| {
    AppConfig::load()
      .map(|c| c.ui.sidebar_collapsed)
      .unwrap_or(false)
  });

  // Persist sidebar state to config when changed
  use_effect(move || {
    let collapsed = nav_collapsed();
    if let Ok(mut config) = AppConfig::load() {
      config.update_sidebar_collapsed(collapsed);
    }
  });

  // Tab switch handler - only collapse on narrow screens
  let tab_switch = |tab: SettingsTab| {
    let mut collapsed = nav_collapsed.clone();
    let width = window_width.clone();
    move |_| {
      active_tab.set(tab);
      if width() < RESPONSIVE_BREAKPOINT {
        collapsed.set(true);
      }
    }
  };

  // Load config
  let providers = use_signal(|| {
    AppConfig::load()
      .map(|c| c.ai.providers)
      .unwrap_or_default()
  });

  let mcp_servers = use_signal(|| AppConfig::load().map(|c| c.mcp.servers).unwrap_or_default());

  // Form states for AI providers
  let editing_provider = use_signal(|| Option::<String>::None);
  let form_id = use_signal(|| String::new());
  let form_name = use_signal(|| String::new());
  let form_model = use_signal(|| String::new());
  let form_api_key = use_signal(|| String::new());
  let form_adapter_type = use_signal(|| Option::<String>::None);
  let form_base_url = use_signal(|| String::new());

  // Form states for MCP servers
  let editing_server = use_signal(|| Option::<String>::None);
  let server_form_name = use_signal(|| String::new());
  let server_form_command = use_signal(|| String::new());
  let server_form_args = use_signal(|| String::new());

  // Tab info helpers
  let tab_info = |tab: SettingsTab| -> (&'static str, Element) {
    match tab {
      SettingsTab::AI => ("AI Providers", rsx!(RobotIcon { class: "w-4 h-4" })),
      SettingsTab::MCP => ("MCP Servers", rsx!(BoltIcon { class: "w-4 h-4" })),
      SettingsTab::Appearance => ("Appearance", rsx!(PaletteIcon { class: "w-4 h-4" })),
      SettingsTab::Shortcuts => ("Shortcuts", rsx!(KeyboardIcon { class: "w-4 h-4" })),
      SettingsTab::QuickTools => ("Quick Tools", rsx!(RocketIcon { class: "w-4 h-4" })),
      SettingsTab::About => ("About", rsx!(InfoIcon { class: "w-4 h-4" })),
    }
  };
  let (current_label, current_icon) = tab_info(active_tab());

  rsx! {
    div {
      class: "relative flex flex-1 overflow-hidden h-full",

      DrawerSidebar {
        collapsed: nav_collapsed(),
        on_close: move |_| nav_collapsed.set(true),

        // Nav tabs
        div {
          class: "flex flex-col h-full",
          div {
            class: "p-4 space-y-1",
            NavTab {
              label: "AI Providers".to_string(),
              value: "ai".to_string(),
              active_value: active_tab().as_str().to_string(),
              icon_element: rsx!(RobotIcon { class: "w-4 h-4" }),
              onclick: tab_switch(SettingsTab::AI),
            }
            NavTab {
              label: "MCP Servers".to_string(),
              value: "mcp".to_string(),
              active_value: active_tab().as_str().to_string(),
              icon_element: rsx!(BoltIcon { class: "w-4 h-4" }),
              onclick: tab_switch(SettingsTab::MCP),
            }
            NavTab {
              label: "Appearance".to_string(),
              value: "appearance".to_string(),
              active_value: active_tab().as_str().to_string(),
              icon_element: rsx!(PaletteIcon { class: "w-4 h-4" }),
              onclick: tab_switch(SettingsTab::Appearance),
            }
            NavTab {
              label: "Shortcuts".to_string(),
              value: "shortcuts".to_string(),
              active_value: active_tab().as_str().to_string(),
              icon_element: rsx!(KeyboardIcon { class: "w-4 h-4" }),
              onclick: tab_switch(SettingsTab::Shortcuts),
            }
            NavTab {
              label: "Quick Tools".to_string(),
              value: "quick_tools".to_string(),
              active_value: active_tab().as_str().to_string(),
              icon_element: rsx!(RocketIcon { class: "w-4 h-4" }),
              onclick: tab_switch(SettingsTab::QuickTools),
            }
            NavTab {
              label: "About".to_string(),
              value: "about".to_string(),
              active_value: active_tab().as_str().to_string(),
              icon_element: rsx!(InfoIcon { class: "w-4 h-4" }),
              onclick: tab_switch(SettingsTab::About),
            }
          }
        }
      }

      // Main content area
      div {
        class: "flex-1 flex flex-col overflow-hidden",

        // Header with current tab + hamburger toggle
        div {
          class: "flex items-center gap-3 px-4 py-3 border-b border-border",
          button {
            class: "w-8 h-8 flex items-center justify-center rounded-full hover:bg-bg-surface text-text-secondary",
            onclick: move |_| nav_collapsed.set(!nav_collapsed()),
            "☰"
          }
          div {
            class: "text-sm font-medium text-text-primary flex items-center gap-2",
            {current_icon}
            span {
              "{current_label}"
            }
          }
        }

        // Tab content panel
        div {
          class: "flex-1 overflow-y-auto p-6",
          {
              render_active_tab(
                  active_tab(),
                  providers.clone(),
                  mcp_servers.clone(),
                  editing_provider.clone(),
                  form_id.clone(),
                  form_name.clone(),
                  form_model.clone(),
                  form_api_key.clone(),
                  form_adapter_type.clone(),
                  form_base_url.clone(),
                  editing_server.clone(),
                  server_form_name.clone(),
                  server_form_command.clone(),
                  server_form_args.clone(),
              )
          }
        }
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
  form_model: Signal<String>,
  form_api_key: Signal<String>,
  form_adapter_type: Signal<Option<String>>,
  form_base_url: Signal<String>,
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
        form_model: form_model.clone(),
        form_api_key: form_api_key.clone(),
        form_adapter_type: form_adapter_type.clone(),
        form_base_url: form_base_url.clone(),
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
      AppearanceTab {

      }
    },
    SettingsTab::Shortcuts => rsx! {
      ShortcutsTab {

      }
    },
    SettingsTab::QuickTools => rsx! {
      QuickToolsTab {

      }
    },
    SettingsTab::About => rsx! {
      AboutTab {

      }
    },
  }
}
