use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

/// REPL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplConfig {
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    
    /// Enable auto-completion
    pub auto_completion: bool,
    
    /// Enable multiline editing
    pub multiline_mode: bool,
    
    /// Show execution time for each command
    pub show_timing: bool,
    
    /// Maximum history entries
    pub max_history: usize,
    
    /// Color scheme
    pub color_scheme: ColorScheme,
    
    /// Prompt style
    pub prompt_style: PromptStyle,
    
    /// Enable VI mode
    pub vi_mode: bool,
    
    /// Auto-save history
    pub auto_save_history: bool,
    
    /// Show tips on startup
    pub show_tips: bool,
    
    /// Startup script path
    pub startup_script: Option<PathBuf>,

    /// Automatically indent new lines in multiline mode
    #[serde(default = "default_true")] 
    pub auto_indent: bool,
    
    /// When completing a function, auto insert trailing parentheses
    #[serde(default = "default_true")] 
    pub auto_insert_function_parens: bool,

    /// Use fancy (miette) error reports
    #[serde(default = "default_true")]
    pub fancy_errors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Monokai,
    Dracula,
    SolarizedDark,
    SolarizedLight,
    Nord,
    OneDark,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptStyle {
    Simple,
    Minimal,
    Powerline,
    Arrow,
    Custom(String),
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            auto_completion: true,
            multiline_mode: true,
            show_timing: true,
            max_history: 10000,
            color_scheme: ColorScheme::Default,
            prompt_style: PromptStyle::Arrow,
            vi_mode: false,
            auto_save_history: true,
            show_tips: true,
            startup_script: None,
            auto_indent: true,
            auto_insert_function_parens: true,
            fancy_errors: true,
        }
    }
}

fn default_true() -> bool { true }

impl ReplConfig {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: ReplConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }
    
    /// Get configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("veyra");
        path.push("repl-config.toml");
        Ok(path)
    }
    
    /// Get history file path
    pub fn history_path() -> Result<PathBuf> {
        let mut path = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
        path.push("veyra");
        path.push("repl-history.txt");
        Ok(path)
    }
}
