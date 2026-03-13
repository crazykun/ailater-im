//! FFI bindings for fcitx5 C API
//!
//! This module provides safe Rust wrappers around the fcitx5 C API.

use std::ffi::{c_char, c_int, c_void, CString};

/// Fcitx5 key symbols
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySym {
    None = 0,
    BackSpace = 0xff08,
    Tab = 0xff09,
    Return = 0xff0d,
    Escape = 0xff1b,
    Space = 0x0020,
    Delete = 0xffff,
    Left = 0xff51,
    Up = 0xff52,
    Right = 0xff53,
    Down = 0xff54,
    PageUp = 0xff55,
    PageDown = 0xff56,
    Plus = 0x002b,
    Minus = 0x002d,
    Equal = 0x003d,
    ShiftL = 0xffe1,
    ShiftR = 0xffe2,
    ControlL = 0xffe3,
    ControlR = 0xffe4,
    AltL = 0xffe9,
    AltR = 0xffea,
    SuperL = 0xffeb,
    SuperR = 0xffec,
}

impl KeySym {
    /// Create from raw key value
    pub fn from_raw(value: u32) -> Self {
        match value {
            0xff08 => KeySym::BackSpace,
            0xff09 => KeySym::Tab,
            0xff0d => KeySym::Return,
            0xff1b => KeySym::Escape,
            0x0020 => KeySym::Space,
            0xffff => KeySym::Delete,
            0xff51 => KeySym::Left,
            0xff52 => KeySym::Up,
            0xff53 => KeySym::Right,
            0xff54 => KeySym::Down,
            0xff55 => KeySym::PageUp,
            0xff56 => KeySym::PageDown,
            0x002b => KeySym::Plus,
            0x002d => KeySym::Minus,
            0x003d => KeySym::Equal,
            0xffe1 => KeySym::ShiftL,
            0xffe2 => KeySym::ShiftR,
            0xffe3 => KeySym::ControlL,
            0xffe4 => KeySym::ControlR,
            0xffe9 => KeySym::AltL,
            0xffea => KeySym::AltR,
            0xffeb => KeySym::SuperL,
            0xffec => KeySym::SuperR,
            _ => KeySym::None,
        }
    }

    /// Check if this is a letter key (a-z)
    pub fn is_letter(&self, raw: u32) -> bool {
        (raw >= 0x0041 && raw <= 0x005a) || (raw >= 0x0061 && raw <= 0x007a)
    }

    /// Check if this is a number key (0-9)
    pub fn is_number(&self, raw: u32) -> bool {
        raw >= 0x0030 && raw <= 0x0039
    }

    /// Check if this is a printable character
    pub fn is_printable(&self, raw: u32) -> bool {
        raw >= 0x0020 && raw < 0x007f
    }
}

/// Key state modifiers
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyState(pub u32);

impl KeyState {
    pub const NONE: KeyState = KeyState(0);
    pub const SHIFT: KeyState = KeyState(1 << 0);
    pub const CTRL: KeyState = KeyState(1 << 1);
    pub const ALT: KeyState = KeyState(1 << 2);
    pub const SUPER: KeyState = KeyState(1 << 3);

    pub fn has_shift(&self) -> bool {
        (self.0 & Self::SHIFT.0) != 0
    }

    pub fn has_ctrl(&self) -> bool {
        (self.0 & Self::CTRL.0) != 0
    }

    pub fn has_alt(&self) -> bool {
        (self.0 & Self::ALT.0) != 0
    }

    pub fn has_super(&self) -> bool {
        (self.0 & Self::SUPER.0) != 0
    }
}

/// Return value for key event handlers
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IMReturnValue {
    /// Ignore the key event
    Ignore = 0,
    /// Forward the key event to the application
    Forward = 1,
    /// Consume the key event (don't forward)
    Consume = 2,
}

/// Opaque pointer to FcitxInstance
#[repr(transparent)]
pub struct FcitxInstance(pub *mut c_void);

/// Opaque pointer to FcitxInputContext
#[repr(transparent)]
pub struct FcitxInputContext(pub *mut c_void);

/// Fcitx5 IM Class structure
#[repr(C)]
pub struct FcitxIMClass {
    pub create: Option<unsafe extern "C" fn(*mut FcitxInstance) -> *mut c_void>,
    pub destroy: Option<unsafe extern "C" fn(*mut c_void)>,
}

/// Input method entry information
#[repr(C)]
pub struct FcitxIMEntry {
    pub unique_name: *const c_char,
    pub name: *const c_char,
    pub icon_name: *const c_char,
    pub priority: c_int,
    pub lang_code: *const c_char,
    pub user_data: *mut c_void,
}

// External fcitx5 API functions (linked at runtime)
extern "C" {
    #[link_name = "fcitx_instance_commit_string"]
    pub fn fcitx_instance_commit_string(
        instance: *mut FcitxInstance,
        ic: *mut FcitxInputContext,
        str: *const c_char,
    );

    #[link_name = "fcitx_instance_set_preedit"]
    pub fn fcitx_instance_set_preedit(
        instance: *mut FcitxInstance,
        ic: *mut FcitxInputContext,
        str: *const c_char,
        cursor_pos: c_int,
    );
}

/// Safe wrapper for committing a string
pub unsafe fn commit_string(instance: *mut FcitxInstance, ic: *mut FcitxInputContext, text: &str) {
    if let Ok(c_text) = CString::new(text) {
        fcitx_instance_commit_string(instance, ic, c_text.as_ptr());
    }
}

/// Safe wrapper for setting preedit text
pub unsafe fn set_preedit(
    instance: *mut FcitxInstance,
    ic: *mut FcitxInputContext,
    text: &str,
    cursor_pos: i32,
) {
    if let Ok(c_text) = CString::new(text) {
        fcitx_instance_set_preedit(instance, ic, c_text.as_ptr(), cursor_pos);
    }
}
