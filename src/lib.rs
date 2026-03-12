//! ailater-im - AI-powered input method for fcitx5
//!
//! This is a Rust-based input method engine for fcitx5 that supports:
//! - Traditional pinyin input
//! - AI-powered prediction using remote or local models
//! - Customizable dictionary and user preferences
//!
//! # Features
//!
//! - **Pinyin Input**: Full pinyin input support with syllable segmentation
//! - **AI Prediction**: Integration with remote AI models (0.8B or custom models)
//! - **Local Model Support**: Optional local inference using candle
//! - **Fuzzy Matching**: Intelligent fuzzy pinyin matching
//! - **User Dictionary**: Learning from user input patterns
//! - **Phrase Prediction**: Context-aware phrase suggestions
//!
//! # Architecture
//!
//! The input method consists of several components:
//!
//! 1. **FFI Layer** (`ffi`, `ffi_exports`, `ffi_fcitx5`): C-compatible interface for fcitx5
//! 2. **Engine** (`engine`): Core input processing logic
//! 3. **Model** (`model`): AI model client for predictions
//! 4. **Dictionary** (`dictionary`): Word lookup and frequency management
//! 5. **Pinyin** (`pinyin`): Pinyin parsing and conversion
//! 6. **Config** (`config`): Configuration management

pub mod ffi;
pub mod ffi_exports;
pub mod ffi_fcitx5;
pub mod engine;
pub mod model;
pub mod config;
pub mod pinyin;
pub mod dictionary;

pub use config::Config;
pub use engine::InputEngine;

// External C++ factory function from ffi_wrapper.cpp
// This symbol is defined by FCITX_ADDON_FACTORY macro in the C++ wrapper
// We declare it here to force the linker to include it from the static library
#[cfg(feature = "fcitx5")]
extern "C" {
    // The factory instance created by FCITX_ADDON_FACTORY macro
    // This is what fcitx5 looks for when loading the addon
    #[link_name = "fcitx_addon_factory_instance"]
    fn fcitx_addon_factory_instance() -> *mut std::ffi::c_void;
}

/// Force include the C++ factory symbol
/// This is called during library initialization to ensure the linker
/// includes the fcitx_addon_factory_instance symbol from the C++ wrapper
#[cfg(feature = "fcitx5")]
#[no_mangle]
pub extern "C" fn ailater_force_cpp_link() -> *mut std::ffi::c_void {
    unsafe { fcitx_addon_factory_instance() }
}

/// Version of the input method
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the input method
pub const IM_NAME: &str = "AI Later";

/// Unique identifier for the input method
pub const IM_UNIQUE_NAME: &str = "ai-later";

/// Language code (zh_CN for Simplified Chinese)
pub const IM_LANG_CODE: &str = "zh_CN";

/// Re-export commonly used types
pub mod prelude {
    pub use crate::config::{Config, ModelConfig, InputConfig, UIConfig, DictionaryConfig};
    pub use crate::engine::{InputEngine, InputState, Candidate};
    pub use crate::model::{ModelBackend, PredictionResult, PredictionSource};
    pub use crate::dictionary::{Dictionary, DictEntry};
    pub use crate::pinyin::{PinyinParser, FuzzyPinyinMatcher, get_candidates};
}
