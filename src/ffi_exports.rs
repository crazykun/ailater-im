//! Fcitx5 plugin entry point
//!
//! This module provides the C-callable interface for fcitx5 to load the input method.

use std::ffi::{c_void, c_char};
use std::ptr;
use std::sync::OnceLock;

use crate::engine::InputEngine;
use crate::config::Config;
use crate::ffi::{FcitxIMClass, FcitxIMEntry, IMReturnValue, FcitxInstance, FcitxInputContext};

/// Global input engine instance
static ENGINE: OnceLock<InputEngine> = OnceLock::new();

/// Plugin data structure
struct PluginData {
    engine: &'static InputEngine,
    instance: *mut FcitxInstance,
}

/// Create the input method instance
/// Called by fcitx5 when loading the plugin
#[no_mangle]
pub extern "C" fn fcitx_im_create(instance: *mut FcitxInstance) -> *mut c_void {
    // Load configuration
    let config = Config::load_or_default();
    
    // Initialize engine
    let engine = ENGINE.get_or_init(|| InputEngine::new(config));
    
    // Create plugin data
    let data = Box::new(PluginData {
        engine,
        instance,
    });
    
    Box::into_raw(data) as *mut c_void
}

/// Destroy the input method instance
/// Called by fcitx5 when unloading the plugin
#[no_mangle]
pub extern "C" fn fcitx_im_destroy(data: *mut c_void) {
    if data.is_null() {
        return;
    }
    
    unsafe {
        let _ = Box::from_raw(data as *mut PluginData);
    }
}

/// Handle key event
/// Called by fcitx5 for each key press/release
#[no_mangle]
pub extern "C" fn fcitx_im_key_event(
    data: *mut c_void,
    ic: *mut FcitxInputContext,
    keysym: u32,
    keycode: u32,
    state: u32,
    is_release: bool,
) -> i32 {
    if data.is_null() {
        return IMReturnValue::Forward as i32;
    }
    
    let plugin_data = unsafe { &*(data as *const PluginData) };
    
    let result = plugin_data.engine.handle_key(
        plugin_data.instance,
        ic,
        keysym,
        keycode,
        state,
        is_release,
    );
    
    result as i32
}

/// Reset input context
/// Called when the input context is reset
#[no_mangle]
pub extern "C" fn fcitx_im_reset(data: *mut c_void, ic: *mut FcitxInputContext) {
    if data.is_null() {
        return;
    }
    
    let plugin_data = unsafe { &*(data as *const PluginData) };
    plugin_data.engine.reset(ic);
}

/// Focus in handler
/// Called when the input context gains focus
#[no_mangle]
pub extern "C" fn fcitx_im_focus_in(data: *mut c_void, ic: *mut FcitxInputContext) {
    if data.is_null() {
        return;
    }
    
    let plugin_data = unsafe { &*(data as *const PluginData) };
    plugin_data.engine.focus_in(ic);
}

/// Focus out handler
/// Called when the input context loses focus
#[no_mangle]
pub extern "C" fn fcitx_im_focus_out(data: *mut c_void, ic: *mut FcitxInputContext) {
    if data.is_null() {
        return;
    }
    
    let plugin_data = unsafe { &*(data as *const PluginData) };
    plugin_data.engine.focus_out(ic);
}

/// Get the IM class structure
/// This is the main entry point that fcitx5 looks for
#[no_mangle]
pub extern "C" fn fcitx_im_get_class() -> *mut FcitxIMClass {
    static IM_CLASS: FcitxIMClass = FcitxIMClass {
        create: Some(fcitx_im_create),
        destroy: Some(fcitx_im_destroy),
    };
    
    &IM_CLASS as *const FcitxIMClass as *mut FcitxIMClass
}

/// Get the list of input methods provided by this addon
#[no_mangle]
#[allow(static_mut_refs)]
pub extern "C" fn fcitx_im_get_entries() -> *mut FcitxIMEntry {
    static mut ENTRIES: [FcitxIMEntry; 2] = [
        FcitxIMEntry {
            unique_name: b"ai-later\0".as_ptr() as *const c_char,
            name: b"AI Later\0".as_ptr() as *const c_char,
            icon_name: b"ailater-im\0".as_ptr() as *const c_char,
            priority: 100,
            lang_code: b"zh_CN\0".as_ptr() as *const c_char,
            user_data: ptr::null_mut(),
        },
        FcitxIMEntry {
            unique_name: ptr::null(),
            name: ptr::null(),
            icon_name: ptr::null(),
            priority: 0,
            lang_code: ptr::null(),
            user_data: ptr::null_mut(),
        },
    ];
    
    unsafe { ENTRIES.as_mut_ptr() }
}

/// Initialize the addon
/// Called when fcitx5 loads the addon
#[no_mangle]
pub extern "C" fn fcitx_addon_init(_instance: *mut FcitxInstance) -> bool {
    // Initialize logging
    #[cfg(debug_assertions)]
    {
        env_logger::init();
    }
    
    log::info!("ailater-im addon initialized");
    true
}

/// Uninitialize the addon
/// Called when fcitx5 unloads the addon
#[no_mangle]
pub extern "C" fn fcitx_addon_uninit() {
    log::info!("ailater-im addon uninitialized");
}

// FFI exports for fcitx5
#[cfg(target_os = "linux")]
mod linux_exports {
    use super::*;
    
    /// Reload configuration
    #[no_mangle]
    pub extern "C" fn fcitx_im_reload_config(data: *mut c_void) {
        if data.is_null() {
            return;
        }
        
        // Reload configuration
        let _config = Config::load_or_default();
        log::info!("Configuration reloaded");
        
        // Note: In a real implementation, we would update the engine's config
        // This would require the engine to support config updates
    }
    
    /// Get configuration UI
    /// Returns a UI description for fcitx5-configtool
    #[no_mangle]
    pub extern "C" fn fcitx_im_get_config_desc() -> *mut c_void {
        // In a real implementation, this would return a configuration description
        // for the fcitx5 configuration tool
        ptr::null_mut()
    }
}

/// C-compatible callback structure for key events
#[repr(C)]
pub struct KeyEventCallback {
    pub callback: Option<unsafe extern "C" fn(*mut c_void, *mut FcitxInputContext, u32, u32, u32, bool) -> i32>,
    pub user_data: *mut c_void,
}

/// C-compatible callback structure for reset events
#[repr(C)]
pub struct ResetCallback {
    pub callback: Option<unsafe extern "C" fn(*mut c_void, *mut FcitxInputContext)>,
    pub user_data: *mut c_void,
}

// Ensure the C structures are compatible
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;
    
    #[test]
    fn test_struct_sizes() {
        assert_eq!(size_of::<FcitxIMClass>(), size_of::<usize>() * 2);
    }
    
    #[test]
    fn test_class_creation() {
        let class = fcitx_im_get_class();
        assert!(!class.is_null());
        
        unsafe {
            assert!((*class).create.is_some());
            assert!((*class).destroy.is_some());
        }
    }
    
    #[test]
    fn test_entries() {
        let entries = fcitx_im_get_entries();
        assert!(!entries.is_null());
        
        unsafe {
            let name = CStr::from_ptr((*entries).name);
            assert_eq!(name.to_str().unwrap(), "AI Pinyin");
        }
    }
}
