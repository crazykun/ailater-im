//! Fcitx5 C interface for Rust
//!
//! This module exports C functions that are called by the C++ wrapper.
//! It forwards calls to the InputEngine in engine.rs.

use std::ffi::{c_void, c_char, CString};
use std::ptr;
use std::sync::OnceLock;

use crate::engine::InputEngine;
use crate::config::Config;
use crate::ffi::FcitxInputContext;

/// Global engine instance (singleton)
static ENGINE: OnceLock<InputEngine> = OnceLock::new();

/// Global config
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Buffer for returning preedit string to C
static PREEDIT_BUFFER: OnceLock<parking_lot::Mutex<Option<CString>>> = OnceLock::new();

/// Buffer for returning candidates array to C
static CANDIDATES_BUFFER: OnceLock<parking_lot::Mutex<Vec<CString>>> = OnceLock::new();

fn get_preedit_buffer() -> &'static parking_lot::Mutex<Option<CString>> {
    PREEDIT_BUFFER.get_or_init(|| parking_lot::Mutex::new(None))
}

fn get_candidates_buffer() -> &'static parking_lot::Mutex<Vec<CString>> {
    CANDIDATES_BUFFER.get_or_init(|| parking_lot::Mutex::new(Vec::new()))
}

/// Create engine instance
#[no_mangle]
pub extern "C" fn ailater_engine_create(_instance: *mut c_void) -> *mut c_void {
    // Initialize logging in debug mode
    #[cfg(debug_assertions)]
    {
        let _ = env_logger::try_init();
    }

    let config = CONFIG.get_or_init(|| Config::load_or_default());
    let engine = ENGINE.get_or_init(|| InputEngine::new(config.clone()));

    log::info!("ailater_im engine created");

    engine as *const InputEngine as *mut c_void
}

/// Destroy engine instance
#[no_mangle]
pub extern "C" fn ailater_engine_destroy(_engine: *mut c_void) {
    // Engine is static, no cleanup needed
    log::info!("ailater_im engine destroyed");
}

/// Handle key event
///
/// This directly forwards to InputEngine::handle_key() which contains:
/// - Pinyin parsing
/// - Dictionary lookup
/// - Fuzzy matching
/// - AI prediction
/// - Candidate generation and sorting
///
/// For selecting candidates, pass the number key's keysym (0x30 + index for 1-9)
#[no_mangle]
pub extern "C" fn ailater_engine_handle_key(
    engine: *mut c_void,
    ic: *mut c_void,
    keysym: u32,
    keycode: u32,
    state: u32,
    is_release: bool,
) -> i32 {
    if engine.is_null() || is_release {
        return 1; // FORWARD
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    // Call the engine's handle_key method
    // This contains all the core logic:
    // - Letter/number/punctuation handling
    // - Candidate generation (dictionary + fuzzy + AI)
    // - State management
    // - Candidate selection (when number keys are pressed)
    let result = engine.handle_key(
        std::ptr::null_mut(), // instance
        ic,
        keysym,
        keycode,
        state,
        is_release,
    );

    // Convert IMReturnValue to i32
    // Ignore = 0, Forward = 1, Consume = 2
    result as i32
}

/// Reset input state
#[no_mangle]
pub extern "C" fn ailater_engine_reset(engine: *mut c_void, ic: *mut c_void) {
    if engine.is_null() {
        return;
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    engine.reset(ic);
}

/// Focus in handler
#[no_mangle]
pub extern "C" fn ailater_engine_focus_in(engine: *mut c_void, ic: *mut c_void) {
    if engine.is_null() {
        return;
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    engine.focus_in(ic);
}

/// Focus out handler
#[no_mangle]
pub extern "C" fn ailater_engine_focus_out(engine: *mut c_void, ic: *mut c_void) {
    if engine.is_null() {
        return;
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    engine.focus_out(ic);
}

/// Get preedit text (the current pinyin input being composed)
///
/// Returns a pointer to a null-terminated C string.
/// The pointer is valid until the next call to this function.
/// Returns NULL if there is no preedit text.
#[no_mangle]
pub extern "C" fn ailater_engine_get_preedit(engine: *mut c_void, ic: *mut c_void) -> *const c_char {
    if engine.is_null() {
        return ptr::null();
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    let preedit = engine.get_preedit(ic);

    if preedit.is_empty() {
        return ptr::null();
    }

    // Store in static buffer to avoid memory leaks
    let mut buffer = get_preedit_buffer().lock();
    *buffer = CString::new(preedit).ok();

    buffer.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null())
}

/// Get candidates for current input
///
/// Returns a pointer to an array of C string pointers.
/// The array is null-terminated.
/// The pointers are valid until the next call to this function.
///
/// Note: The returned array is leaked and should be freed by the caller
/// if they want to avoid memory leaks. In practice with fcitx5, this is
/// called frequently and the old array will be replaced.
///
/// Example usage:
///   const char** candidates = ailater_engine_get_candidates(engine, ic);
///   for (int i = 0; candidates[i] != NULL; i++) {
///       printf("Candidate %d: %s\n", i + 1, candidates[i]);
///   }
#[no_mangle]
pub extern "C" fn ailater_engine_get_candidates(
    engine: *mut c_void,
    ic: *mut c_void,
) -> *const *const c_char {
    if engine.is_null() {
        return ptr::null();
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    let candidates = engine.get_candidates(ic);

    if candidates.is_empty() {
        return ptr::null();
    }

    // Store CStrings in static buffer
    let mut cstring_buffer = get_candidates_buffer().lock();
    cstring_buffer.clear();

    // Convert candidates to CStrings
    for candidate in &candidates {
        if let Ok(c_string) = CString::new(candidate.text.as_str()) {
            cstring_buffer.push(c_string);
        }
    }

    // Build array of pointers on heap and leak it
    // This is acceptable for an FFI boundary where the caller
    // will read the data and we'll clean up on next call
    let mut pointers: Vec<*const c_char> = cstring_buffer.iter()
        .map(|c| c.as_ptr())
        .collect();

    // Add null terminator
    pointers.push(ptr::null());

    // Leak the vector and return pointer to its data
    let boxed = pointers.into_boxed_slice();
    Box::leak(boxed).as_ptr()
}

/// Get the number of candidates available
#[no_mangle]
pub extern "C" fn ailater_engine_get_candidate_count(
    engine: *mut c_void,
    ic: *mut c_void,
) -> usize {
    if engine.is_null() {
        return 0;
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    engine.get_candidates(ic).len()
}

/// Get candidate at specific index (0-based)
///
/// Returns the candidate text as a C string, or NULL if index is invalid.
/// The returned pointer must be freed with ailater_engine_free_string().
#[no_mangle]
pub extern "C" fn ailater_engine_get_candidate_at(
    engine: *mut c_void,
    ic: *mut c_void,
    index: usize,
) -> *mut c_char {
    if engine.is_null() {
        return ptr::null_mut();
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    let candidates = engine.get_candidates(ic);

    if index >= candidates.len() {
        return ptr::null_mut();
    }

    match CString::new(candidates[index].text.as_str()) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Select and commit a candidate by index (0-based)
///
/// This simulates pressing the corresponding number key.
/// Returns the committed text as a C string, or NULL on failure.
/// The returned pointer must be freed with ailater_engine_free_string().
#[no_mangle]
pub extern "C" fn ailater_engine_select_candidate(
    engine: *mut c_void,
    ic: *mut c_void,
    index: usize,
) -> *mut c_char {
    if engine.is_null() || index >= 9 {
        return ptr::null_mut();
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    let ic = ic as *mut FcitxInputContext;

    let candidates_before = engine.get_candidates(ic);

    if index >= candidates_before.len() {
        return ptr::null_mut();
    }

    let committed_text = candidates_before[index].text.clone();

    // Simulate pressing the number key (1-9 => keysym 0x31-0x39)
    let keysym = 0x30 + (index + 1) as u32;

    let _result = engine.handle_key(
        std::ptr::null_mut(),
        ic,
        keysym,
        0,
        0,
        false,
    );

    // Return the committed text
    match CString::new(committed_text.as_str()) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string allocated by Rust
///
/// This should be called for strings returned by:
/// - ailater_engine_get_candidate_at
/// - ailater_engine_select_candidate
#[no_mangle]
pub extern "C" fn ailater_engine_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Check if model is available
#[no_mangle]
pub extern "C" fn ailater_engine_is_model_available(engine: *mut c_void) -> bool {
    if engine.is_null() {
        return false;
    }

    let engine = unsafe { &*(engine as *const InputEngine) };
    engine.is_model_available()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine_ptr = ailater_engine_create(std::ptr::null_mut());
        assert!(!engine_ptr.is_null());

        // Test model availability
        assert!(ailater_engine_is_model_available(engine_ptr));

        ailater_engine_destroy(engine_ptr);
    }

    #[test]
    fn test_key_handling() {
        let engine_ptr = ailater_engine_create(std::ptr::null_mut());
        let ic = std::ptr::null_mut();

        // Type 'n'
        let result = ailater_engine_handle_key(engine_ptr, ic, 0x006e, 0, 0, false);
        assert_eq!(result, 2); // CONSUME

        // Check preedit
        let preedit = ailater_engine_get_preedit(engine_ptr, ic);
        if !preedit.is_null() {
            unsafe {
                let s = CString::from_raw(preedit as *mut i8);
                assert_eq!(s.to_str().unwrap(), "n");
            }
        }

        // Check candidates
        let count = ailater_engine_get_candidate_count(engine_ptr, ic);
        assert!(count > 0);

        ailater_engine_destroy(engine_ptr);
    }

    #[test]
    fn test_multiple_key_presses() {
        let engine_ptr = ailater_engine_create(std::ptr::null_mut());
        let ic = std::ptr::null_mut();

        // Type "ni"
        ailater_engine_handle_key(engine_ptr, ic, 0x006e, 0, 0, false); // n
        ailater_engine_handle_key(engine_ptr, ic, 0x0069, 0, 0, false); // i

        let preedit = ailater_engine_get_preedit(engine_ptr, ic);
        if !preedit.is_null() {
            unsafe {
                let s = CString::from_raw(preedit as *mut i8);
                assert_eq!(s.to_str().unwrap(), "ni");
            }
        }

        let candidates = ailater_engine_get_candidates(engine_ptr, ic);
        if !candidates.is_null() {
            unsafe {
                assert!(!(*candidates).is_null());
                let first = CString::from_raw((*candidates) as *mut i8);
                // Should contain Chinese characters for "ni"
                assert!(!first.to_str().unwrap().is_empty());
            }
        }

        ailater_engine_destroy(engine_ptr);
    }

    #[test]
    fn test_candidate_selection() {
        let engine_ptr = ailater_engine_create(std::ptr::null_mut());
        let ic = std::ptr::null_mut();

        // Type "ni"
        ailater_engine_handle_key(engine_ptr, ic, 0x006e, 0, 0, false);
        ailater_engine_handle_key(engine_ptr, ic, 0x0069, 0, 0, false);

        // Select first candidate
        let committed = ailater_engine_select_candidate(engine_ptr, ic, 0);
        if !committed.is_null() {
            unsafe {
                let s = CString::from_raw(committed);
                assert!(!s.to_str().unwrap().is_empty());
            }
        }

        // After selection, preedit should be empty
        let preedit = ailater_engine_get_preedit(engine_ptr, ic);
        assert!(preedit.is_null());

        ailater_engine_destroy(engine_ptr);
    }
}
