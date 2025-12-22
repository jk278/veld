//! Configuration management for Veld
//! Provides unified configuration loading, saving, and management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeConfig,
    pub ai: AiConfig,
    pub shortcuts: ShortcutConfig,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
}

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: Option<String>,  // openai/anthropic/local
    pub api_key: Option<String>,   // 加密存储的API密钥
    pub model: Option<String>,     // 默认模型
    pub temperature: Option<f32>,  // 生成参数
}

/// Shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub activate: Option<String>,      // 主要激活快捷键
    pub quick_summarize: Option<String>,
    pub quick_translate: Option<String>,
    pub quick_explain: Option<String>,
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
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
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
                provider: None,
                api_key: None,
                model: None,
                temperature: None,
            },
            shortcuts: ShortcutConfig {
                activate: Some("Ctrl+Shift+Space".to_string()),
                quick_summarize: Some("Ctrl+Shift+S".to_string()),
                quick_translate: Some("Ctrl+Shift+T".to_string()),
                quick_explain: Some("Ctrl+Shift+E".to_string()),
            },
        }
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            println!("[Config] No config file found, using defaults");
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(ConfigError::Io)?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(ConfigError::Json)?;

        println!("[Config] Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir();
        fs::create_dir_all(&config_dir)
            .map_err(ConfigError::Io)?;

        let config_path = Self::get_config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(ConfigError::Json)?;

        fs::write(&config_path, json)
            .map_err(ConfigError::Io)?;

        println!("[Config] Configuration saved successfully");
        Ok(())
    }

    /// Update theme configuration
    pub fn update_theme(&mut self, mode: ThemeMode) {
        self.theme.mode = mode;
        // Save in background thread to avoid blocking UI
        let config = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = config.save() {
                eprintln!("[Config] Failed to save theme config: {}", e);
            }
        });
    }

    /// Update AI configuration
    pub fn update_ai(&mut self, ai_config: AiConfig) {
        self.ai = ai_config;
        // Save in background thread
        let config = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = config.save() {
                eprintln!("[Config] Failed to save AI config: {}", e);
            }
        });
    }

    /// Update shortcuts configuration
    pub fn update_shortcuts(&mut self, shortcuts: ShortcutConfig) {
        self.shortcuts = shortcuts;
        // Save in background thread
        let config = self.clone();
        std::thread::spawn(move || {
            if let Err(e) = config.save() {
                eprintln!("[Config] Failed to save shortcuts config: {}", e);
            }
        });
    }
}

/// Helper function to get just the theme mode (for backward compatibility)
pub fn load_theme_mode() -> Result<Option<ThemeMode>> {
    match AppConfig::load() {
        Ok(config) => Ok(Some(config.theme.mode)),
        Err(e) => {
            eprintln!("[Config] Failed to load theme mode: {}", e);
            Ok(None)
        }
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
