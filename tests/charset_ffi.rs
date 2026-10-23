use std::ffi::{CStr, CString};

use skey_engine::{skey_charset_decode, skey_charset_encode, skey_charset_free_buf};

unsafe fn ffi_encode(input: &str, charset: i32) -> Vec<u8> {
    let input = CString::new(input).unwrap();
    let mut len = 0;
    let ptr = skey_charset_encode(input.as_ptr(), charset, &mut len);
    assert!(!ptr.is_null(), "charset {} returned a null buffer", charset);
    let bytes = std::slice::from_raw_parts(ptr, len).to_vec();
    skey_charset_free_buf(ptr);
    bytes
}

unsafe fn ffi_decode(input: &[u8], charset: i32) -> String {
    let ptr = skey_charset_decode(input.as_ptr(), input.len(), charset);
    assert!(!ptr.is_null(), "charset {} returned a null string", charset);
    let value = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    skey_engine::skey_free_string(ptr);
    value
}

#[test]
fn ffi_routes_all_charset_ids() {
    // IDs 5–12 used to silently fall through to Unicode in lib.rs.  Every
    // legacy ID must now produce its own encoded representation and decode it
    // back to the original text.
    for charset in 1..=12 {
        let encoded = unsafe { ffi_encode("ư", charset) };
        assert_ne!(encoded, "ư".as_bytes(), "charset {} fell back to UTF-8", charset);
        assert_eq!(unsafe { ffi_decode(&encoded, charset) }, "ư", "charset {}", charset);
    }
}

#[test]
fn ffi_vni_does_not_emit_nul_padding() {
    let encoded = unsafe { ffi_encode("ĐđưÍ", 2) };
    assert_eq!(encoded, [0xD1, 0xF1, 0xF6, 0xCD]);
}

#[test]
fn ffi_unicode_nfd_remains_distinct_from_nfc() {
    let encoded = unsafe { ffi_encode("ế", 13) };
    assert_eq!(encoded, "e\u{0302}\u{0301}".as_bytes());
    assert_eq!(unsafe { ffi_decode(&encoded, 13) }, "e\u{0302}\u{0301}");
}
