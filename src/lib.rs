//! skey-engine — Stateless Vietnamese input method engine.
//!
//! Letter-based accumulator model for Vietnamese input.
//! Key features:
//!   - Stateless: input string → output string
//!   - Letter struct with variant + tone tracking
//!   - Modifier keys mutate the last vowel in place
//!   - Precomputed UTF-8 tables for output
//!   - Correct tone placement for all diphthong/triphthong cases

pub mod tables;
pub mod engine;
pub mod spelling;
pub mod charset;
pub mod vseq;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Input method.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Method { Telex, Vni, TeipVni, Viqr }

/// Opaque engine handle — holds only configuration.
pub struct SkeyEngine {
    method: Method,
    modern: bool,
    short_w: bool,
    auto_restore: bool,
}

impl SkeyEngine {
    pub fn new(method: Method) -> Self {
        SkeyEngine { method, modern: true, short_w: false, auto_restore: false }
    }

    pub fn set_method(&mut self, method: Method) { self.method = method; }
    pub fn set_modern(&mut self, v: bool) { self.modern = v; }
    pub fn set_short_w(&mut self, v: bool) { self.short_w = v; }
    pub fn set_auto_restore(&mut self, v: bool) { self.auto_restore = v; }

    pub fn transform(&self, input: &str) -> String {
        let composed = match self.method {
            Method::Telex => engine::convert_telex(input, self.modern, self.short_w),
            Method::Vni => engine::convert_vni(input, self.modern, self.short_w),
            Method::TeipVni => engine::convert_teip_vni(input, self.modern, self.short_w),
            Method::Viqr => engine::convert_viqr(input),
        };
        // Auto-restore: if the composed output is not valid Vietnamese and
        // differs from the raw input, return the raw input instead.
        // Skip if the output is all ASCII (e.g. "cow" from toggle-off,
        // English words) — those are intentional, not corrupted.
        // Also skip if the input contains Vietnamese composition markers
        // (dd → đ, vowel+w, double vowels) — the engine intentionally
        // transformed them and the result should be kept.
        let all_ascii = composed.chars().all(|c| c.is_ascii());
        if self.auto_restore && !all_ascii && composed != input
            && !Self::is_valid(&composed)
            && !has_vn_markers(input)
        {
            input.to_string()
        } else {
            composed
        }
    }

    pub fn is_valid(s: &str) -> bool {
        spelling::is_valid_cvc(s)
    }
}

/// Check whether the input string contains Vietnamese composition markers
/// that signal intentional IME transformation. When these are present,
/// auto-restore should not revert the engine output even if the result
/// isn't a complete valid syllable (e.g. abbreviations like "đc").
pub(crate) fn has_vn_markers(input: &str) -> bool {
    // Only "dd" is an unambiguous Vietnamese digraph that signals intentional
    // IME transformation. Other digraphs (aw, ow, aa, ee, oo) also appear in
    // English words and repeated-key patterns — auto-restore should handle them.
    input.to_lowercase().contains("dd")
}

// ── C FFI ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_new(method: i32) -> *mut SkeyEngine {
    let m = match method {
        0 => Method::Telex,    // SKEY_METHOD_TELEX
        1 => Method::Vni,      // SKEY_METHOD_VNI
        2 => Method::Viqr,     // SKEY_METHOD_VIQR
        3 => Method::TeipVni,  // SKEY_METHOD_TEIP_VNI
        _ => Method::Telex,
    };
    Box::into_raw(Box::new(SkeyEngine::new(m)))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_free(engine: *mut SkeyEngine) {
    if !engine.is_null() { unsafe { drop(Box::from_raw(engine)); } }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_set_method(engine: *mut SkeyEngine, method: i32) {
    if engine.is_null() { return; }
    let m = match method {
        0 => Method::Telex,
        1 => Method::Vni,
        2 => Method::Viqr,
        3 => Method::TeipVni,
        _ => Method::Telex,
    };
    unsafe { (*engine).set_method(m); }
}

/// tone_style: 0=traditional (hoà, thuỷ), 1=modern (hòa, thủy). Default modern.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_set_tone_style(e: *mut SkeyEngine, v: i32) {
    if !e.is_null() { unsafe { (*e).set_modern(v != 0); } }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_set_free_marking(e: *mut SkeyEngine, v: i32) {
    // Free marking ON → modern style (tone on 2nd vowel for oa/oe/uy)
    // Free marking OFF → traditional (tone on 1st vowel for oa/oe/uy)
    if !e.is_null() { unsafe { (*e).set_modern(v != 0); } }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_set_short_w(e: *mut SkeyEngine, v: i32) {
    if !e.is_null() { unsafe { (*e).set_short_w(v != 0); } }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_set_auto_restore(e: *mut SkeyEngine, v: i32) {
    if !e.is_null() { unsafe { (*e).set_auto_restore(v != 0); } }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_set_bracket_uo(_e: *mut SkeyEngine, _v: i32) {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_transform(
    engine: *const SkeyEngine,
    input: *const c_char,
) -> *mut c_char {
    if engine.is_null() || input.is_null() { return std::ptr::null_mut(); }
    let e = unsafe { &*engine };
    let s = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s, Err(_) => return std::ptr::null_mut(),
    };
    CString::new(e.transform(s)).unwrap_or_default().into_raw()
}

// ── Charset FFI ──────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_charset_encode(
    input: *const c_char,
    charset: i32,
    out_len: *mut usize,
) -> *mut u8 {
    if input.is_null() || out_len.is_null() { return std::ptr::null_mut(); }
    let s = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s, Err(_) => return std::ptr::null_mut(),
    };
    let cs = match charset {
        0 => charset::VietCharset::Unicode,
        1 => charset::VietCharset::TCVN3,
        2 => charset::VietCharset::VNIWin,
        3 => charset::VietCharset::WinCP1258,
        4 => charset::VietCharset::VIQR,
        _ => charset::VietCharset::Unicode,
    };
    let encoded = charset::encode(s, cs);
    let len = encoded.len();
    let ptr = libc::malloc(len).cast::<u8>();
    if ptr.is_null() { return std::ptr::null_mut(); }
    unsafe { std::ptr::copy_nonoverlapping(encoded.as_ptr(), ptr, len); }
    unsafe { *out_len = len; }
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_charset_decode(
    input: *const u8,
    len: usize,
    charset: i32,
) -> *mut c_char {
    if input.is_null() { return std::ptr::null_mut(); }
    let bytes = unsafe { std::slice::from_raw_parts(input, len) };
    let cs = match charset {
        0 => charset::VietCharset::Unicode,
        1 => charset::VietCharset::TCVN3,
        2 => charset::VietCharset::VNIWin,
        3 => charset::VietCharset::WinCP1258,
        4 => charset::VietCharset::VIQR,
        _ => charset::VietCharset::Unicode,
    };
    let decoded = charset::decode(bytes, cs);
    CString::new(decoded).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_charset_remove_tone(input: *const c_char) -> *mut c_char {
    if input.is_null() { return std::ptr::null_mut(); }
    let s = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s, Err(_) => return std::ptr::null_mut(),
    };
    CString::new(charset::remove_tone(s)).unwrap_or_default().into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_charset_free_buf(ptr: *mut u8) {
    if !ptr.is_null() { unsafe { libc::free(ptr.cast()); } }
}

// ── C FFI (continued) ────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_is_valid(s: *const c_char) -> i32 {
    if s.is_null() { return 0; }
    let s = match unsafe { CStr::from_ptr(s) }.to_str() {
        Ok(s) => s, Err(_) => return 0,
    };
    if SkeyEngine::is_valid(s) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}
