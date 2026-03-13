//! Configuration management for the input method
//!
//! Handles loading, saving, and managing user preferences.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Model configuration
    pub model: ModelConfig,
    /// Input behavior settings
    pub input: InputConfig,
    /// UI settings
    pub ui: UIConfig,
    /// Dictionary settings
    pub dictionary: DictionaryConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            input: InputConfig::default(),
            ui: UIConfig::default(),
            dictionary: DictionaryConfig::default(),
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model type: "remote", "local", or "hybrid"
    #[serde(default = "default_model_type")]
    pub model_type: String,

    /// Remote model API endpoint
    #[serde(default = "default_api_endpoint")]
    pub api_endpoint: String,

    /// API key for remote model
    #[serde(default)]
    pub api_key: Option<String>,

    /// Local model path
    #[serde(default)]
    pub local_model_path: Option<String>,

    /// Model name for remote API
    #[serde(default = "default_model_name")]
    pub model_name: String,

    /// Maximum tokens for prediction
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Enable caching for predictions
    #[serde(default = "default_true")]
    pub enable_cache: bool,

    /// Cache size (number of entries)
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

fn default_model_type() -> String {
    "none".to_string()
}
fn default_api_endpoint() -> String {
    "http://localhost:8080/v1".to_string()
}
fn default_model_name() -> String {
    "qwen-0.8b".to_string()
}
fn default_max_tokens() -> u32 {
    50
}
fn default_temperature() -> f32 {
    0.7
}
fn default_true() -> bool {
    true
}
fn default_cache_size() -> usize {
    10000
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_type: default_model_type(),
            api_endpoint: default_api_endpoint(),
            api_key: None,
            local_model_path: None,
            model_name: default_model_name(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            enable_cache: default_true(),
            cache_size: default_cache_size(),
        }
    }
}

/// Input behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    /// Maximum preedit length
    #[serde(default = "default_max_preedit")]
    pub max_preedit_length: usize,

    /// Number of candidates to show
    #[serde(default = "default_num_candidates")]
    pub num_candidates: usize,

    /// Enable fuzzy pinyin matching
    #[serde(default = "default_true")]
    pub fuzzy_pinyin: bool,

    /// Enable smart correction
    #[serde(default = "default_true")]
    pub smart_correction: bool,

    /// Page size for candidate list
    #[serde(default = "default_page_size")]
    pub page_size: usize,

    /// Auto-commit on punctuation
    #[serde(default = "default_true")]
    pub auto_commit_on_punctuation: bool,

    /// Enable phrase prediction
    #[serde(default)]
    pub enable_phrase_prediction: bool,

    /// Minimum input length for AI prediction
    #[serde(default = "default_min_ai_input")]
    pub min_ai_input_length: usize,
}

fn default_max_preedit() -> usize {
    64
}
fn default_num_candidates() -> usize {
    30
}
fn default_page_size() -> usize {
    5
}
fn default_min_ai_input() -> usize {
    2
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            max_preedit_length: default_max_preedit(),
            num_candidates: default_num_candidates(),
            fuzzy_pinyin: default_true(),
            smart_correction: default_true(),
            page_size: default_page_size(),
            auto_commit_on_punctuation: default_true(),
            enable_phrase_prediction: default_true(),
            min_ai_input_length: default_min_ai_input(),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Show candidate numbers
    #[serde(default = "default_true")]
    pub show_candidate_numbers: bool,

    /// Vertical candidate list
    #[serde(default)]
    pub vertical_candidate_list: bool,

    /// Font size for candidates
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    /// Custom font family
    #[serde(default)]
    pub font_family: Option<String>,
}

fn default_font_size() -> u32 {
    12
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            show_candidate_numbers: default_true(),
            vertical_candidate_list: false,
            font_size: default_font_size(),
            font_family: None,
        }
    }
}

/// Dictionary configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryConfig {
    /// System dictionary path
    #[serde(default = "default_system_dict")]
    pub system_dictionary: String,

    /// User dictionary path
    #[serde(default = "default_user_dict")]
    pub user_dictionary: String,

    /// Enable user dictionary learning
    #[serde(default = "default_true")]
    pub enable_learning: bool,

    /// Maximum user dictionary size
    #[serde(default = "default_user_dict_size")]
    pub max_user_dictionary_size: usize,

    /// Frequency decay factor (for forgetting old entries)
    #[serde(default = "default_decay_factor")]
    pub frequency_decay: f32,
}

fn default_system_dict() -> String {
    "/usr/share/ailater-im/dict/system.dict".to_string()
}
fn default_user_dict() -> String {
    "~/.local/share/ailater-im/user.dict".to_string()
}
fn default_user_dict_size() -> usize {
    100000
}
fn default_decay_factor() -> f32 {
    0.99
}

impl Default for DictionaryConfig {
    fn default() -> Self {
        Self {
            system_dictionary: default_system_dict(),
            user_dictionary: default_user_dict(),
            enable_learning: default_true(),
            max_user_dictionary_size: default_user_dict_size(),
            frequency_decay: default_decay_factor(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &Path) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content).unwrap_or_else(|e| {
            log::warn!("Failed to parse config file: {}, using defaults", e);
            Config::default()
        });
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    /// Get the default config path
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("ailater-im")
            .join("config.toml")
    }

    /// Load or create default configuration
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            Self::load(&path).unwrap_or_else(|e| {
                log::warn!("Failed to load config: {}, using defaults", e);
                Config::default()
            })
        } else {
            let config = Config::default();
            if let Err(e) = config.save(&path) {
                log::warn!("Failed to save default config: {}", e);
            }
            config
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.model.model_type, "remote");
        assert!(config.input.fuzzy_pinyin);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.model.model_type, parsed.model.model_type);
    }
}
