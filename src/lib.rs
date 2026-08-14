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

use std::collections::HashSet;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::OnceLock;

/// Input method.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Method { Telex, Vni, TeipVni, Viqr }

/// Embedded Vietnamese word list (data/vietnamese.cm.dict).
///
/// Compiled into the binary — no runtime file loading.  The file is kept
/// sorted by byte order (LC_ALL=C sort) in the repo so lookups are binary
/// searches on a parsed-once `Vec<&str>`.  Data source: ibus-bamboo's
/// vietnamese.cm.dict (GPLv3), see data/LICENSE.vietnamese.cm.dict.
static DICT: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Tone-stripped forms of the dictionary words, sorted and deduplicated.
/// Vietnamese tone marks modify vowels in place (ơ→ớ), so a mid-word
/// composition like "phươc" never appears as a string prefix of the final
/// word "phước" — it matches the tone-stripped base form instead.
static BASES: OnceLock<Vec<String>> = OnceLock::new();

/// Access the embedded dictionary, parsing it on first use.
fn dict() -> &'static Vec<&'static str> {
    DICT.get_or_init(|| {
        include_str!("../data/vietnamese.cm.dict")
            .lines()
            .collect()
    })
}

/// Tone-stripped base forms of the dictionary, sorted and deduplicated.
fn bases() -> &'static Vec<String> {
    BASES.get_or_init(|| {
        let mut v: Vec<String> =
            dict().iter().map(|w| charset::remove_tone(w)).collect();
        v.sort();
        v.dedup();
        v
    })
}

/// Opaque engine handle — holds only configuration.
pub struct SkeyEngine {
    method: Method,
    modern: bool,
    short_w: bool,
    auto_restore: bool,
    dict: bool,
    user_words: HashSet<String>,
}

impl SkeyEngine {
    pub fn new(method: Method) -> Self {
        SkeyEngine {
            method,
            modern: true,
            short_w: false,
            auto_restore: false,
            dict: false,
            user_words: HashSet::new(),
        }
    }

    pub fn set_method(&mut self, method: Method) { self.method = method; }
    pub fn set_modern(&mut self, v: bool) { self.modern = v; }
    pub fn set_short_w(&mut self, v: bool) { self.short_w = v; }
    pub fn set_auto_restore(&mut self, v: bool) { self.auto_restore = v; }
    /// Dictionary mode: auto-restore validates the composed output against
    /// the embedded Vietnamese word list instead of syllable rules.  Words
    /// the list lacks (user vocabulary) can be added with `add_word`.
    pub fn set_dict(&mut self, v: bool) { self.dict = v; }
    /// Add a user word (kept lowercase).  User words are checked before the
    /// embedded list, so they override a missing entry.
    pub fn add_word(&mut self, word: &str) {
        self.user_words.insert(word.to_lowercase());
    }

    /// Remove all user words (reload of the user dictionary file).
    pub fn clear_words(&mut self) {
        self.user_words.clear();
    }

    /// Dictionary mode validity: `word` (lowercase) is a real Vietnamese
    /// word or a viable mid-word intermediate, so composition stays visible
    /// in real time while typing.
    ///
    /// Two tiers:
    /// - Tone-marked output must match a dictionary word exactly, or be an
    ///   intermediate whose base form is a strict prefix of some dictionary
    ///   word's base ("lượ" → base "lươ" ⊂ "lươc" of "lược").  An exact
    ///   base match alone is NOT enough — that would accept wrong-tone
    ///   words like "lước" just because "lược" shares its base form.
    /// - Tone-less output is an intermediate: keep it when it is (or could
    ///   become) the tone-stripped base form of a dictionary word, e.g.
    ///   "phươc" (typing "phuowc") is the base of "phước".
    fn dict_known(&self, word: &str) -> bool {
        let base = charset::remove_tone(word);
        if base != word {
            // Tone marks present — exact word, or strict base-prefix.
            if self.user_words.contains(word)
                || dict().binary_search(&word).is_ok()
            {
                return true;
            }
            for w in &self.user_words {
                let wb = charset::remove_tone(w);
                if wb.starts_with(&base) && wb != base {
                    return true;
                }
            }
            let bs = bases();
            match bs.binary_search(&base) {
                Ok(_) => false, // tone-variant of a real word — reject
                Err(i) => i < bs.len() && bs[i].starts_with(&base),
            }
        } else {
            // No tone marks — base-form exact or prefix match.
            for w in &self.user_words {
                if charset::remove_tone(w).starts_with(&base) {
                    return true;
                }
            }
            let bs = bases();
            match bs.binary_search(&base) {
                Ok(_) => true,
                Err(i) => i < bs.len() && bs[i].starts_with(&base),
            }
        }
    }

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
        let known = if self.dict {
            // Dictionary mode: valid = in the embedded word list (or a
            // prefix of one) or added by the user.  Lookup is lowercase —
            // the dict is stored so.
            self.dict_known(&composed.to_lowercase())
        } else {
            Self::is_valid(&composed)
        };
        if self.auto_restore && !all_ascii && composed != input
            && !known && !has_vn_markers(input)
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
    // Vietnamese composition markers that signal intentional IME transforms.
    //   dd → đ                  (d-stroke — always intentional)
    //   aaa/eee/ooo             (3+ same vowel = circumflex toggle in progress)
    //   aww/oww/uww             (vowel+w+w = horn/breve toggle chain)
    //
    // We deliberately do NOT match "aw"/"ow"/"uw" (single w) — those appear
    // in English words (cow, how, law) whose output (cơ, hơ, lă) is valid
    // Vietnamese anyway, so auto-restore wouldn't touch them.
    // The toggle chains (vowww → vơw) produce invalid output that needs
    // protection from auto-restore.
    let lower = input.to_lowercase();
    lower.contains("dd")
        || lower.contains("aaa")
        || lower.contains("eee")
        || lower.contains("ooo")
        || lower.contains("aww")
        || lower.contains("oww")
        || lower.contains("uww")
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
pub unsafe extern "C" fn skey_engine_set_dict(e: *mut SkeyEngine, v: i32) {
    if !e.is_null() { unsafe { (*e).set_dict(v != 0); } }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_add_word(e: *mut SkeyEngine, word: *const c_char) {
    if e.is_null() || word.is_null() { return; }
    let s = match unsafe { CStr::from_ptr(word) }.to_str() {
        Ok(s) => s, Err(_) => return,
    };
    unsafe { (*e).add_word(s); }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_clear_words(e: *mut SkeyEngine) {
    if !e.is_null() { unsafe { (*e).clear_words(); } }
}

/// All embedded dictionary words joined by '\n' (sorted).  Caller frees
/// with skey_free_string().
#[unsafe(no_mangle)]
pub unsafe extern "C" fn skey_engine_dict_words() -> *mut c_char {
    CString::new(dict().join("\n")).unwrap_or_default().into_raw()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn eng(auto_restore: bool, dict: bool) -> SkeyEngine {
        let mut e = SkeyEngine::new(Method::Telex);
        e.set_auto_restore(auto_restore);
        e.set_dict(dict);
        e
    }

    #[test]
    fn dict_restores_nonwords() {
        let e = eng(true, true);
        // "lước" is a structurally valid syllable but not a real word —
        // rule-based check keeps it, dictionary check restores it.
        assert_eq!(e.transform("luwocs"), "luwocs");
        // English "need" → "nêd" is not in the dictionary either.
        assert_eq!(e.transform("need"), "need");
        // "force" → "fỏce" is not a word → restored.
        assert_eq!(e.transform("force"), "force");
    }

    #[test]
    fn dict_keeps_prefixes_realtime() {
        // Mid-word prefixes of real words must not be restored, so the
        // composition stays visible while typing.
        let e = eng(true, true);
        assert_eq!(e.transform("phuowc"), "phươc"); // prefix of "phước"
        assert_eq!(e.transform("phuowcs"), "phước"); // complete word
        assert_eq!(e.transform("truwoc"), "trươc"); // prefix of "trước"
        // Tone-marked intermediates whose base is a strict prefix of a
        // dictionary word's base ("lượ" → "lươ" ⊂ "lươc" of "lược").
        assert_eq!(e.transform("luow"), "lươ");
        assert_eq!(e.transform("luowj"), "lượ");
        assert_eq!(e.transform("luowjc"), "lược");
        // User words' prefixes are kept too.
        let mut e2 = eng(true, true);
        e2.add_word("lước");
        assert_eq!(e2.transform("luwoc"), "lươc");
    }

    #[test]
    fn dict_keeps_real_words() {
        let e = eng(true, true);
        assert_eq!(e.transform("truwocs"), "trước");
        assert_eq!(e.transform("khoong"), "không");
        assert_eq!(e.transform("Vieetj"), "Việt");
        assert_eq!(e.transform("luoojcw"), "lược"); // luộc + w → lược
    }

    #[test]
    fn dict_keeps_abbreviations() {
        let e = eng(true, true);
        assert_eq!(e.transform("ddc"), "đc"); // has_vn_markers protection
    }

    #[test]
    fn double_vowel_swap_survives_auto_restore() {
        // lắm + a → lấm (hook↔circumflex swap) is a valid word, so
        // auto-restore must NOT revert it to raw keystrokes.
        let e = eng(true, false);
        assert_eq!(e.transform("lawsma"), "lấm");
        let e = eng(true, true);
        assert_eq!(e.transform("lawsma"), "lấm"); // dict mode keeps it too
        assert_eq!(e.transform("luowjco"), "luộc"); // lược + o → luộc (ươ→uô)
    }

    #[test]
    fn dict_off_keeps_rule_based_behavior() {
        let e = eng(true, false);
        assert_eq!(e.transform("luwocs"), "lước"); // rule-valid → kept
        assert_eq!(e.transform("need"), "need"); // rule-invalid → restored
    }

    #[test]
    fn dict_user_words_override() {
        let mut e = eng(true, true);
        assert_eq!(e.transform("luwocs"), "luwocs");
        e.add_word("Lước"); // stored lowercase — matched case-insensitively
        assert_eq!(e.transform("luwocs"), "lước");
        assert_eq!(e.transform("LUWOCS"), "LƯỚC");
    }

    #[test]
    fn dict_embedded_is_sorted() {
        // binary_search requires byte-order sorting of the embedded file.
        let dict = DICT.get_or_init(|| {
            include_str!("../data/vietnamese.cm.dict")
                .lines()
                .collect()
        });
        assert!(dict.len() > 7000, "dict unexpectedly small: {}", dict.len());
        for w in dict.windows(2) {
            assert!(w[0] < w[1], "dict not sorted at {:?}", w);
        }
    }
}
