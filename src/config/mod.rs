//! Configuration management for Veld
//! Provides unified configuration loading, saving, and management

use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// Re-export from hooks for default sidebar width
pub use crate::hooks::DEFAULT_SIDEBAR_WIDTH;

/// Global save lock to prevent concurrent writes to config file
/// Ensures all save operations are serialized
static SAVE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_save_lock() -> &'static Mutex<()> {
  SAVE_LOCK.get_or_init(|| Mutex::new(()))
}

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
  pub theme: ThemeConfig,
  pub ai: AiConfig,
  pub mcp: McpConfig,
  pub shortcuts: ShortcutConfig,
  pub ui: UiConfig,
  pub quick_tools: QuickToolsConfig,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
  pub mode: ThemeMode,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
  pub providers: Vec<ProviderConfig>,
  pub active_provider: Option<String>,
}

/// MCP (Model Context Protocol) configuration
/// MCPs are tools available to AI - enabled servers are all active (no selection needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
  pub servers: Vec<McpServerConfig>,
}

impl Default for McpConfig {
  fn default() -> Self {
    McpConfig {
      servers: vec![McpServerConfig {
        name: "Context7".to_string(),
        command: "npx".to_string(),
        args: vec!["-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
        env: None,
        enabled: false,
      }],
    }
  }
}

/// Individual MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
  pub name: String,
  pub command: String,
  pub args: Vec<String>,
  pub env: Option<std::collections::HashMap<String, String>>,
  pub enabled: bool,
}

/// Individual AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
  pub id: String,
  pub name: String,
  /// Model name (e.g., "gpt-4o-mini", "claude-3-5-sonnet", "glm-4-plus")
  pub model: String,
  pub api_key: Option<String>,
  /// Adapter type: "openai" (OpenAI-compatible) or "anthropic" (Anthropic-compatible)
  pub adapter_type: Option<String>,
  /// Optional: Custom endpoint URL (overrides default adapter endpoint)
  pub base_url: Option<String>,
  pub enabled: bool,
}

/// Shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
  pub command_palette: Option<String>, // Open command palette shortcut (e.g., "Ctrl+Shift+Space")
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
  pub sidebar_collapsed: bool,
  pub zoom_level: f64,  // 0.5 ~ 2.0, default 1.0
  pub sidebar_width: u32,  // Sidebar width in pixels (200-600)
  pub window_width: Option<u32>,  // Persisted window width (physical pixels)
  pub window_height: Option<u32>, // Persisted window height (physical pixels)
}

/// Quick Tools (presets) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickToolsConfig {
  pub prompts: Vec<QuickPrompt>,
}

/// Quick prompt preset for fast AI actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuickPrompt {
  pub id: String,
  pub name: String,
  pub keyword: String, // Keyword for /command (e.g., "summarize")
  pub prefix: String,  // System prompt prefix to inject
  #[serde(default)]
  pub placeholder: Option<String>, // Optional placeholder (auto-generated if None)
}

impl QuickPrompt {
  /// Get the placeholder text, auto-generating if not set
  pub fn get_placeholder(&self) -> String {
    self
      .placeholder
      .clone()
      .unwrap_or_else(|| format!("Enter content for {}...", self.name.to_lowercase()))
  }
}

impl Default for UiConfig {
  fn default() -> Self {
    UiConfig {
      sidebar_collapsed: false,
      zoom_level: 1.0,
      sidebar_width: DEFAULT_SIDEBAR_WIDTH,
      window_width: None,
      window_height: None,
    }
  }
}

impl Default for QuickToolsConfig {
  fn default() -> Self {
    QuickToolsConfig {
      prompts: vec![
        QuickPrompt {
          id: "summarize".into(),
          name: "Summarize".into(),
          keyword: "summarize".into(),
          prefix: "请简洁总结以下内容，突出关键要点：\n\n".into(),
          placeholder: None,
        },
        QuickPrompt {
          id: "explain".into(),
          name: "Explain".into(),
          keyword: "explain".into(),
          prefix: "请详细解释以下代码或概念，用通俗易懂的语言：\n\n".into(),
          placeholder: None,
        },
        QuickPrompt {
          id: "translate".into(),
          name: "Translate".into(),
          keyword: "translate".into(),
          prefix: "请将以下内容翻译成英文：\n\n".into(),
          placeholder: None,
        },
        QuickPrompt {
          id: "refactor".into(),
          name: "Refactor".into(),
          keyword: "refactor".into(),
          prefix: "请分析以下代码并提供重构建议，改进可读性和性能：\n\n".into(),
          placeholder: None,
        },
        QuickPrompt {
          id: "doc".into(),
          name: "Generate Docs".into(),
          keyword: "doc".into(),
          prefix: "请为以下代码生成完整的文档注释：\n\n".into(),
          placeholder: None,
        },
        QuickPrompt {
          id: "test".into(),
          name: "Generate Tests".into(),
          keyword: "test".into(),
          prefix: "请为以下代码生成单元测试：\n\n".into(),
          placeholder: None,
        },
      ],
    }
  }
}

/// Theme mode enum (moved from theme.rs for centralized configuration)
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ThemeMode {
  Light,
  Dark,
  System,
}

impl Default for ThemeMode {
  fn default() -> Self {
    ThemeMode::System
  }
}

/// Configuration error type
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
  #[error("JSON serialization error: {0}")]
  Json(#[from] serde_json::Error),
  #[error("Configuration directory not found")]
  ConfigDirNotFound,
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

impl AppConfig {
  /// Get the configuration directory path
  fn get_config_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("veld");
    path
  }

  /// Get the configuration file path
  fn get_config_path() -> PathBuf {
    let mut path = Self::get_config_dir();
    path.push("config.json");
    path
  }

  /// Create default configuration
  pub fn default() -> Self {
    AppConfig {
      theme: ThemeConfig {
        mode: ThemeMode::System,
      },
      ai: AiConfig {
        providers: vec![
          ProviderConfig {
            id: "claude".to_string(),
            name: "Claude".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            api_key: None,
            adapter_type: Some("anthropic".to_string()),
            base_url: None,
            enabled: true,
          },
          ProviderConfig {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: None,
            adapter_type: Some("openai".to_string()),
            base_url: None,
            enabled: true,
          },
          ProviderConfig {
            id: "glm".to_string(),
            name: "GLM (智谱)".to_string(),
            model: "glm-4-plus".to_string(),
            api_key: None,
            adapter_type: Some("anthropic".to_string()),
            base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
            enabled: true,
          },
          ProviderConfig {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            model: "MiniMax-M2.1".to_string(),
            api_key: None,
            adapter_type: Some("anthropic".to_string()),
            base_url: Some("https://api.minimax.chat/v1".to_string()),
            enabled: true,
          },
        ],
        active_provider: Some("claude".to_string()),
      },
      mcp: McpConfig {
        servers: vec![McpServerConfig {
          name: "Context7".to_string(),
          command: "npx".to_string(),
          args: vec!["-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
          env: None,
          enabled: true,
        }],
      },
      shortcuts: ShortcutConfig {
        command_palette: Some("Ctrl+Shift+Space".to_string()),
      },
      ui: UiConfig::default(),
      quick_tools: QuickToolsConfig::default(),
    }
  }

  /// Load configuration from file
  pub fn load() -> Result<Self> {
    let config_path = Self::get_config_path();

    if !config_path.exists() {
      return Ok(Self::default());
    }

    let content = fs::read_to_string(&config_path).map_err(ConfigError::Io)?;

    let config: AppConfig = serde_json::from_str(&content).map_err(ConfigError::Json)?;

    Ok(config)
  }

  /// Save configuration to file
  pub fn save(&self) -> Result<()> {
    // Acquire global lock to prevent concurrent writes
    let _lock = get_save_lock()
      .lock()
      .map_err(|_| ConfigError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to acquire save lock",
      )))?;

    let config_dir = Self::get_config_dir();
    fs::create_dir_all(&config_dir).map_err(ConfigError::Io)?;

    let config_path = Self::get_config_path();
    let json = serde_json::to_string_pretty(self).map_err(ConfigError::Json)?;

    fs::write(&config_path, json).map_err(ConfigError::Io)?;

    Ok(())
  }

  /// Update theme configuration
  pub fn update_theme(&mut self, mode: ThemeMode) {
    self.theme.mode = mode;
    // Save in background thread to avoid blocking UI
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update AI configuration
  pub fn update_ai(&mut self, ai_config: AiConfig) {
    self.ai = ai_config;
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update a single provider configuration
  pub fn update_provider(&mut self, provider: ProviderConfig) {
    if let Some(pos) = self.ai.providers.iter().position(|p| p.id == provider.id) {
      self.ai.providers[pos] = provider;
    } else {
      self.ai.providers.push(provider);
    }
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Set active provider
  /// IMPORTANT: Only sets the active pointer, does NOT validate if provider has API key.
  /// The caller should ensure the provider is actually usable (enabled + has API key).
  pub fn set_active_provider(&mut self, provider_id: String) {
    self.ai.active_provider = Some(provider_id);
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Get the active provider only if it's actually usable (enabled + has API key)
  /// Returns None if active provider is missing, disabled, or has no API key
  pub fn get_usable_provider(&self) -> Option<&ProviderConfig> {
    let active_id = self.ai.active_provider.as_ref()?;
    self.ai.providers.iter().find(|p| {
      p.id == *active_id && p.enabled && p.api_key.as_ref().map_or(false, |k| !k.is_empty())
    })
  }

  /// Update MCP configuration
  pub fn update_mcp(&mut self, mcp_config: McpConfig) {
    self.mcp = mcp_config;
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update a single MCP server configuration
  pub fn update_mcp_server(&mut self, server: McpServerConfig) {
    if let Some(pos) = self.mcp.servers.iter().position(|s| s.name == server.name) {
      self.mcp.servers[pos] = server;
    } else {
      self.mcp.servers.push(server);
    }
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Get enabled MCP servers (for AI agent tool context)
  pub fn get_enabled_mcps(&self) -> Vec<&McpServerConfig> {
    self.mcp.servers.iter().filter(|s| s.enabled).collect()
  }

  /// Update shortcuts configuration
  pub fn update_shortcuts(&mut self, shortcuts: ShortcutConfig) {
    self.shortcuts = shortcuts;
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update sidebar collapsed state
  pub fn update_sidebar_collapsed(&mut self, collapsed: bool) {
    self.ui.sidebar_collapsed = collapsed;
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update window size (width x height in physical pixels)
  pub fn update_window_size(&mut self, width: u32, height: u32) {
    self.ui.window_width = Some(width);
    self.ui.window_height = Some(height);
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update sidebar width (clamped to 200-400px)
  pub fn update_sidebar_width(&mut self, width: u32) {
    self.ui.sidebar_width = width.clamp(200, 400);
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update zoom level
  pub fn update_zoom_level(&mut self, zoom: f64) {
    self.ui.zoom_level = zoom.clamp(0.5, 2.0);
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }

  /// Update quick tools configuration
  pub fn update_quick_tools(&mut self, quick_tools: QuickToolsConfig) {
    self.quick_tools = quick_tools;
    // Save in background thread
    let config = self.clone();
    std::thread::spawn(move || {
      let _ = config.save();
    });
  }
}

/// Helper function to get just the theme mode (for backward compatibility)
pub fn load_theme_mode() -> Result<Option<ThemeMode>> {
  match AppConfig::load() {
    Ok(config) => Ok(Some(config.theme.mode)),
    Err(_) => Ok(None),
  }
}

/// Helper function to save theme mode (for backward compatibility)
pub fn save_theme_mode(mode: ThemeMode) -> Result<()> {
  match AppConfig::load() {
    Ok(mut config) => {
      config.update_theme(mode);
      Ok(())
    }
    Err(_) => {
      // If config doesn't exist, create default and update
      let mut config = AppConfig::default();
      config.update_theme(mode);
      Ok(())
    }
  }
}
