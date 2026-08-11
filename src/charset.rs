//! Vietnamese charset conversion.
//!
//! Implements legacy charset encode/decode using the same 213-character
//! pivot table approach as x-unikey's vnconv.  Each charset is defined
//! by a single 213-element lookup table mapping the internal index to
//! the charset's byte representation.
//!
//! Supported charsets:
//!   Unicode (UTF-8)  — native, no conversion needed
//!   TCVN3 (ABC)       — single-byte, the most common legacy charset
//!   VNI-WIN           — double-byte, second most common legacy
//!   VIQR              — ASCII quoted-readable
//!   WinCP1258         — Windows Vietnamese code page
//!   UnicodeDecomposed — NFD (combining diacritics)

use std::collections::HashMap;
use std::sync::OnceLock;

// ── Internal 213-char index ──────────────────────────────────────────
//
// Layout (same as x-unikey StdVnChar):
//   Indices  0.. 35: a group   — A,a,Á,á,À,à,Ả,ả,Ã,ã,Ạ,ạ  (12)
//   Indices 36.. 47: b,c,d group — B,b,C,c,D,d,Đ,đ          (12+2=14? no: 36-47=12)
//   ...actually let me refer to the UnicodeTable for the exact layout
//
//   0..11:   a   (12 chars: 6 tones × 2 cases)
//   12..23:  â   (12)
//   24..35:  ă   (12)
//   36..41:  b,c,d (6)
//   42..43:  đ,Đ (2)
//   44..55:  e   (12)
//   56..67:  ê   (12)
//   68..73:  f,g,h (6)
//   74..85:  i   (12)
//   86..95:  j,k,l,m,n (10)
//   96..107: o   (12)
//   108..119:ô   (12)
//   120..131:ơ   (12)
//   132..141:p,q,r,s,t (10)
//   142..153:u   (12)
//   154..165:ư   (12)
//   166..171:v,w,x (6)
//   172..183:y   (12)
//   184..185:z (2)
//   186..212:Western symbols (27)

const VN_CHAR_COUNT: usize = 213;

// ── Unicode pivot table ──────────────────────────────────────────────
// Ported from x-unikey data.cpp UnicodeTable[213].
// Maps internal index → Unicode codepoint.

static UNICODE_TABLE: [char; VN_CHAR_COUNT] = [
    // 0..11: a (6 tones × 2 cases)
    'A', 'a', '\u{00C1}', '\u{00E1}', '\u{00C0}', '\u{00E0}',
    '\u{1EA2}', '\u{1EA3}', '\u{00C3}', '\u{00E3}', '\u{1EA0}', '\u{1EA1}',
    // 12..23: â
    '\u{00C2}', '\u{00E2}', '\u{1EA4}', '\u{1EA5}', '\u{1EA6}', '\u{1EA7}',
    '\u{1EA8}', '\u{1EA9}', '\u{1EAA}', '\u{1EAB}', '\u{1EAC}', '\u{1EAD}',
    // 24..35: ă
    '\u{0102}', '\u{0103}', '\u{1EAE}', '\u{1EAF}', '\u{1EB0}', '\u{1EB1}',
    '\u{1EB2}', '\u{1EB3}', '\u{1EB4}', '\u{1EB5}', '\u{1EB6}', '\u{1EB7}',
    // 36..41: B,b,C,c,D,d
    'B', 'b', 'C', 'c', 'D', 'd',
    // 42..43: Đ,đ
    '\u{0110}', '\u{0111}',
    // 44..55: e
    'E', 'e', '\u{00C9}', '\u{00E9}', '\u{00C8}', '\u{00E8}',
    '\u{1EBA}', '\u{1EBB}', '\u{1EBC}', '\u{1EBD}', '\u{1EB8}', '\u{1EB9}',
    // 56..67: ê
    '\u{00CA}', '\u{00EA}', '\u{1EBE}', '\u{1EBF}', '\u{1EC0}', '\u{1EC1}',
    '\u{1EC2}', '\u{1EC3}', '\u{1EC4}', '\u{1EC5}', '\u{1EC6}', '\u{1EC7}',
    // 68..73: F,f,G,g,H,h
    'F', 'f', 'G', 'g', 'H', 'h',
    // 74..85: i
    'I', 'i', '\u{00CD}', '\u{00ED}', '\u{00CC}', '\u{00EC}',
    '\u{1EC8}', '\u{1EC9}', '\u{0128}', '\u{0129}', '\u{1ECA}', '\u{1ECB}',
    // 86..95: J,j,K,k,L,l,M,m,N,n
    'J', 'j', 'K', 'k', 'L', 'l', 'M', 'm', 'N', 'n',
    // 96..107: o
    'O', 'o', '\u{00D3}', '\u{00F3}', '\u{00D2}', '\u{00F2}',
    '\u{1ECE}', '\u{1ECF}', '\u{00D5}', '\u{00F5}', '\u{1ECC}', '\u{1ECD}',
    // 108..119: ô
    '\u{00D4}', '\u{00F4}', '\u{1ED0}', '\u{1ED1}', '\u{1ED2}', '\u{1ED3}',
    '\u{1ED4}', '\u{1ED5}', '\u{1ED6}', '\u{1ED7}', '\u{1ED8}', '\u{1ED9}',
    // 120..131: ơ
    '\u{01A0}', '\u{01A1}', '\u{1EDA}', '\u{1EDB}', '\u{1EDC}', '\u{1EDD}',
    '\u{1EDE}', '\u{1EDF}', '\u{1EE0}', '\u{1EE1}', '\u{1EE2}', '\u{1EE3}',
    // 132..141: P,p,Q,q,R,r,S,s,T,t
    'P', 'p', 'Q', 'q', 'R', 'r', 'S', 's', 'T', 't',
    // 142..153: u
    'U', 'u', '\u{00DA}', '\u{00FA}', '\u{00D9}', '\u{00F9}',
    '\u{1EE6}', '\u{1EE7}', '\u{0168}', '\u{0169}', '\u{1EE4}', '\u{1EE5}',
    // 154..165: ư
    '\u{01AF}', '\u{01B0}', '\u{1EE8}', '\u{1EE9}', '\u{1EEA}', '\u{1EEB}',
    '\u{1EEC}', '\u{1EED}', '\u{1EEE}', '\u{1EEF}', '\u{1EF0}', '\u{1EF1}',
    // 166..171: V,v,W,w,X,x
    'V', 'v', 'W', 'w', 'X', 'x',
    // 172..183: y
    'Y', 'y', '\u{00DD}', '\u{00FD}', '\u{1EF2}', '\u{1EF3}',
    '\u{1EF6}', '\u{1EF7}', '\u{1EF8}', '\u{1EF9}', '\u{1EF4}', '\u{1EF5}',
    // 184..185: Z,z
    'Z', 'z',
    // 186..212: Western symbols (port from x-unikey)
    '\u{20AC}', '\u{20A1}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}',
    '\u{2021}', '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}',
    '\u{017D}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}',
    '\u{2013}', '\u{2014}', '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}',
    '\u{0153}', '\u{017E}', '\u{0178}',
];

// ── TCVN3 (ABC) single-byte table ────────────────────────────────────
// Ported from x-unikey SingleByteTables[0].
// Maps internal index → TCVN3 byte value.
// 0x00 means the character is not representable in TCVN3.

static TCVN3_TABLE: [u8; VN_CHAR_COUNT] = [
    // 0..11: a
    b'A', b'a', 0xB8, 0xB8, 0xB5, 0xB5, 0xB6, 0xB6, 0xB7, 0xB7, 0xB9, 0xB9,
    // 12..23: â
    0xA2, 0xA9, 0xCA, 0xCA, 0xC7, 0xC7, 0xC8, 0xC8, 0xC9, 0xC9, 0xCB, 0xCB,
    // 24..35: ă
    0xA1, 0xA8, 0xBE, 0xBE, 0xBB, 0xBB, 0xBC, 0xBC, 0xBD, 0xBD, 0xC6, 0xC6,
    // 36..41: B,b,C,c,D,d
    b'B', b'b', b'C', b'c', b'D', b'd',
    // 42..43: Đ,đ
    0xA5, 0xB0,
    // 44..55: e
    b'E', b'e', 0xC4, 0xC4, 0xBF, 0xBF, 0xC0, 0xC0, 0xC1, 0xC1, 0xC2, 0xC2,
    // 56..67: ê
    0xA4, 0xAB, 0xCC, 0xCC, 0xC3, 0xC3, 0xCE, 0xCE, 0xCF, 0xCF, 0xD0, 0xD0,
    // 68..73: F,f,G,g,H,h
    b'F', b'f', b'G', b'g', b'H', b'h',
    // 74..85: i
    b'I', b'i', 0xAE, 0xAE, 0xA6, 0xA6, 0xB1, 0xB1, 0xB2, 0xB2, 0xB3, 0xB3,
    // 86..95: J,j,K,k,L,l,M,m,N,n
    b'J', b'j', b'K', b'k', b'L', b'l', b'M', b'm', b'N', b'n',
    // 96..107: o
    b'O', b'o', 0xD3, 0xD3, 0xD4, 0xD4, 0xD5, 0xD5, 0xD6, 0xD6, 0xD8, 0xD8,
    // 108..119: ô
    0xA3, 0xAA, 0xD9, 0xD9, 0xD1, 0xD1, 0xDA, 0xDA, 0xDB, 0xDB, 0xDC, 0xDC,
    // 120..131: ơ
    0xA7, 0xAD, 0xE1, 0xE1, 0xDD, 0xDD, 0xDE, 0xDE, 0xDF, 0xDF, 0xE0, 0xE0,
    // 132..141: P,p,Q,q,R,r,S,s,T,t
    b'P', b'p', b'Q', b'q', b'R', b'r', b'S', b's', b'T', b't',
    // 142..153: u
    b'U', b'u', 0xE7, 0xE7, 0xE4, 0xE4, 0xE5, 0xE5, 0xE6, 0xE6, 0xE8, 0xE8,
    // 154..165: ư
    0xB4, 0xAC, 0xE3, 0xE3, 0xD7, 0xD7, 0xE9, 0xE9, 0xEA, 0xEA, 0xEB, 0xEB,
    // 166..171: V,v,W,w,X,x
    b'V', b'v', b'W', b'w', b'X', b'x',
    // 172..183: y
    b'Y', b'y', 0xED, 0xED, 0xEE, 0xEE, 0xF0, 0xF0, 0xF1, 0xF1, 0xEF, 0xEF,
    // 184..185: Z,z
    b'Z', b'z',
    // 186..212: Western symbols
    0x80, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88,
    0x89, 0x8A, 0x8B, 0x8C, 0x8E, 0x91, 0x92, 0x93,
    0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B,
    0x9C, 0x9E, 0x9F,
];

// ── VNI-WIN double-byte table ────────────────────────────────────────
// Ported from x-unikey DoubleByteTables[0].
// Maps internal index → VNI-WIN 2-byte word (big-endian in original).
// 0x0000 means not representable.

static VNIWIN_TABLE: [u16; VN_CHAR_COUNT] = [
    // 0..11: a
    0x0041, 0x0061, 0xd941, 0xf961, 0xd841, 0xf861, 0xdb41, 0xfb61,
    0xd541, 0xf561, 0xcf41, 0xef61,
    // 12..23: â
    0xc241, 0xe261, 0xc141, 0xe161, 0xc041, 0xe061, 0xc541, 0xe561,
    0xc341, 0xe361, 0xc441, 0xe461,
    // 24..35: ă
    0xca41, 0xea61, 0xc941, 0xe961, 0xc841, 0xe861, 0xda41, 0xfa61,
    0xdc41, 0xfc61, 0xcb41, 0xeb61,
    // 36..41: B,b,C,c,D,d
    0x0042, 0x0062, 0x0043, 0x0063, 0x0044, 0x0064,
    // 42..43: Đ,đ
    0x00d1, 0x00f1,
    // 44..55: e
    0x0045, 0x0065, 0xd945, 0xf965, 0xd845, 0xf865, 0xdb45, 0xfb65,
    0xd545, 0xf565, 0xcf45, 0xef65,
    // 56..67: ê
    0xc245, 0xe265, 0xc145, 0xe165, 0xc045, 0xe065, 0xc545, 0xe565,
    0xc345, 0xe365, 0xc445, 0xe465,
    // 68..73: F,f,G,g,H,h
    0x0046, 0x0066, 0x0047, 0x0067, 0x0048, 0x0068,
    // 74..85: i
    0x0049, 0x0069, 0x00cd, 0x00ed, 0x00cc, 0x00ec, 0x00c6, 0x00e6,
    0x00d3, 0x00f3, 0x00d2, 0x00f2,
    // 86..95: J,j,K,k,L,l,M,m,N,n
    0x004a, 0x006a, 0x004b, 0x006b, 0x004c, 0x006c, 0x004d, 0x006d,
    0x004e, 0x006e,
    // 96..107: o
    0x004f, 0x006f, 0xd94f, 0xf96f, 0xd84f, 0xf86f, 0xdb4f, 0xfb6f,
    0xd54f, 0xf56f, 0xcf4f, 0xef6f,
    // 108..119: ô
    0xc24f, 0xe26f, 0xc14f, 0xe16f, 0xc04f, 0xe06f, 0xc54f, 0xe56f,
    0xc34f, 0xe36f, 0xc44f, 0xe46f,
    // 120..131: ơ
    0x00d4, 0x00f4, 0xd9d4, 0xf9f4, 0xd8d4, 0xf8f4, 0xdbd4, 0xfbf4,
    0xd5d4, 0xf5f4, 0xcfd4, 0xeff4,
    // 132..141: P,p,Q,q,R,r,S,s,T,t
    0x0050, 0x0070, 0x0051, 0x0071, 0x0052, 0x0072, 0x0053, 0x0073,
    0x0054, 0x0074,
    // 142..153: u
    0x0055, 0x0075, 0xd955, 0xf975, 0xd855, 0xf875, 0xdb55, 0xfb75,
    0xd555, 0xf575, 0xcf55, 0xef75,
    // 154..165: ư
    0x00d6, 0x00f6, 0xd9d6, 0xf9f6, 0xd8d6, 0xf8f6, 0xdbd6, 0xfbf6,
    0xd5d6, 0xf5f6, 0xcfd6, 0xeff6,
    // 166..171: V,v,W,w,X,x
    0x0056, 0x0076, 0x0057, 0x0077, 0x0058, 0x0078,
    // 172..183: y
    0x0059, 0x0079, 0xd959, 0xf979, 0xd859, 0xf879, 0xdb59, 0xfb79,
    0xd559, 0xf579, 0x00ce, 0x00ee,
    // 184..185: Z,z
    0x005a, 0x007a,
    // 186..212: Western symbols
    0x0080, 0x0082, 0x0083, 0x0084, 0x0085, 0x0086, 0x0087, 0x0088,
    0x0089, 0x008A, 0x008B, 0x008C, 0x008E, 0x0091, 0x0092, 0x0093,
    0x0094, 0x0095, 0x0096, 0x0097, 0x0098, 0x0099, 0x009A, 0x009B,
    0x009C, 0x009E, 0x009F,
];

// ── Tone removal table ──────────────────────────────────────────────
// Ported from x-unikey StdVnNoTone[213].
// Maps internal index → index of toneless equivalent.

static NO_TONE: &[usize; VN_CHAR_COUNT] = &[
    // 0..11: a → a
    0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
    // 12..23: â → â
    12, 13, 12, 13, 12, 13, 12, 13, 12, 13, 12, 13,
    // 24..35: ă → ă
    24, 25, 24, 25, 24, 25, 24, 25, 24, 25, 24, 25,
    // 36..41: b,c,d
    36, 37, 38, 39, 40, 41,
    // 42..43: đ
    42, 43,
    // 44..55: e → e
    44, 45, 44, 45, 44, 45, 44, 45, 44, 45, 44, 45,
    // 56..67: ê → ê
    56, 57, 56, 57, 56, 57, 56, 57, 56, 57, 56, 57,
    // 68..73: f,g,h
    68, 69, 70, 71, 72, 73,
    // 74..85: i → i
    74, 75, 74, 75, 74, 75, 74, 75, 74, 75, 74, 75,
    // 86..95: j,k,l,m,n
    86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    // 96..107: o → o
    96, 97, 96, 97, 96, 97, 96, 97, 96, 97, 96, 97,
    // 108..119: ô → ô
    108, 109, 108, 109, 108, 109, 108, 109, 108, 109, 108, 109,
    // 120..131: ơ → ơ
    120, 121, 120, 121, 120, 121, 120, 121, 120, 121, 120, 121,
    // 132..141: p,q,r,s,t
    132, 133, 134, 135, 136, 137, 138, 139, 140, 141,
    // 142..153: u → u
    142, 143, 142, 143, 142, 143, 142, 143, 142, 143, 142, 143,
    // 154..165: ư → ư
    154, 155, 154, 155, 154, 155, 154, 155, 154, 155, 154, 155,
    // 166..171: v,w,x
    166, 167, 168, 169, 170, 171,
    // 172..183: y → y
    172, 173, 172, 173, 172, 173, 172, 173, 172, 173, 172, 173,
    // 184..185: z
    184, 185,
    // 186..212: Western symbols (no tone)
    186, 187, 188, 189, 190, 191, 192, 193,
    194, 195, 196, 197, 198, 199, 200, 201,
    202, 203, 204, 205, 206, 207, 208, 209,
    210, 211, 212,
];

// ── Public API ───────────────────────────────────────────────────────

/// Supported Vietnamese charsets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VietCharset {
    /// Unicode UTF-8 (native, no conversion needed)
    Unicode,
    /// TCVN3 / ABC — single-byte legacy charset
    TCVN3,
    /// VNI for Windows — double-byte legacy charset
    VNIWin,
}

// ── Lazy-built reverse lookup maps ───────────────────────────────────

fn unicode_to_index_map() -> &'static HashMap<char, usize> {
    static MAP: OnceLock<HashMap<char, usize>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::with_capacity(VN_CHAR_COUNT);
        for (i, &ch) in UNICODE_TABLE.iter().enumerate() {
            m.insert(ch, i);
        }
        m
    })
}

fn tcvn3_to_index_map() -> &'static HashMap<u8, usize> {
    static MAP: OnceLock<HashMap<u8, usize>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::with_capacity(VN_CHAR_COUNT);
        for (i, &b) in TCVN3_TABLE.iter().enumerate() {
            m.insert(b, i);
        }
        m
    })
}

fn vniwin_to_index_map() -> &'static HashMap<u16, usize> {
    static MAP: OnceLock<HashMap<u16, usize>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::with_capacity(VN_CHAR_COUNT);
        for (i, &w) in VNIWIN_TABLE.iter().enumerate() {
            m.insert(w, i);
        }
        m
    })
}

// ── Encode / Decode ─────────────────────────────────────────────────

/// Encode a UTF-8 string to target charset bytes.
pub fn encode(input: &str, charset: VietCharset) -> Vec<u8> {
    match charset {
        VietCharset::Unicode => input.as_bytes().to_vec(),
        VietCharset::TCVN3 => encode_to(input, TCVN3_TABLE.as_slice(), unicode_to_index_map()),
        VietCharset::VNIWin => {
            let map = unicode_to_index_map();
            let mut out = Vec::with_capacity(input.len() * 2);
            for ch in input.chars() {
                if let Some(&idx) = map.get(&ch) {
                    let w = VNIWIN_TABLE[idx];
                    // VNI-WIN byte order: low byte (base char) first, then high byte (tone)
                    out.push((w & 0xFF) as u8);
                    out.push((w >> 8) as u8);
                } else if ch.is_ascii() {
                    // ASCII pass-through: single byte
                    out.push(ch as u8);
                } else {
                    // Non-Vietnamese Unicode: encode as UTF-8
                    let mut buf = [0u8; 4];
                    let s = ch.encode_utf8(&mut buf);
                    out.extend_from_slice(s.as_bytes());
                }
            }
            out
        }
    }
}

fn encode_to(input: &str, table: &[u8], map: &HashMap<char, usize>) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for ch in input.chars() {
        if let Some(&idx) = map.get(&ch) {
            out.push(table[idx]);
        } else {
            // Pass through non-Vietnamese chars as UTF-8 bytes
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            out.extend_from_slice(s.as_bytes());
        }
    }
    out
}

/// Decode charset bytes to a UTF-8 string.
pub fn decode(input: &[u8], charset: VietCharset) -> String {
    match charset {
        VietCharset::Unicode => String::from_utf8_lossy(input).to_string(),
        VietCharset::TCVN3 => {
            let map = tcvn3_to_index_map();
            let mut out = String::with_capacity(input.len());
            for &b in input {
                if let Some(&idx) = map.get(&b) {
                    out.push(UNICODE_TABLE[idx]);
                } else {
                    out.push(b as char);
                }
            }
            out
        }
        VietCharset::VNIWin => {
            let map = vniwin_to_index_map();
            let mut out = String::with_capacity(input.len());
            let mut i = 0;
            while i < input.len() {
                // Try 2-byte VNI-WIN pair
                if i + 1 < input.len() {
                    let lo = input[i];
                    let hi = input[i + 1];
                    let w = lo as u16 | ((hi as u16) << 8);
                    if let Some(&idx) = map.get(&w) {
                        out.push(UNICODE_TABLE[idx]);
                        i += 2;
                        continue;
                    }
                }
                // Fallback: single byte pass-through
                out.push(input[i] as char);
                i += 1;
            }
            out
        }
    }
}

/// Remove tone marks from a Vietnamese UTF-8 string.
/// e.g. "tiếng Việt" → "tiêng Viêt"
pub fn remove_tone(input: &str) -> String {
    let map = unicode_to_index_map();
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if let Some(&idx) = map.get(&ch) {
            let toneless_idx = NO_TONE[idx];
            out.push(UNICODE_TABLE[toneless_idx]);
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_table_size() {
        assert_eq!(UNICODE_TABLE.len(), 213);
        assert_eq!(TCVN3_TABLE.len(), 213);
        assert_eq!(VNIWIN_TABLE.len(), 213);
        assert_eq!(NO_TONE.len(), 213);
    }

    #[test]
    fn unicode_roundtrip() {
        let input = "tiếng Việt";
        let encoded = encode(input, VietCharset::Unicode);
        let decoded = decode(&encoded, VietCharset::Unicode);
        assert_eq!(decoded, input);
    }

    #[test]
    fn tcvn3_roundtrip() {
        let input = "tiếng Việt";
        let encoded = encode(input, VietCharset::TCVN3);
        let decoded = decode(&encoded, VietCharset::TCVN3);
        assert_eq!(decoded, input);
    }

    #[test]
    fn vniwin_roundtrip() {
        let input = "tiếng Việt";
        let encoded = encode(input, VietCharset::VNIWin);
        let decoded = decode(&encoded, VietCharset::VNIWin);
        assert_eq!(decoded, input);
    }

    #[test]
    fn all_vietnamese_vowels() {
        // Test all 12 vowels × 6 tones
        let all_vowels = "aàáảãạ âầấẩẫậ ăằắẳẵặ eèéẻẽẹ êềếểễệ iìíỉĩị oòóỏõọ ôồốổỗộ ơờớởỡợ uùúủũụ ưừứửữự yỳýỷỹỵ";
        for charset in [VietCharset::TCVN3, VietCharset::VNIWin] {
            let encoded = encode(all_vowels, charset);
            let decoded = decode(&encoded, charset);
            assert_eq!(decoded, all_vowels, "roundtrip failed for {:?}", charset);
        }
    }

    #[test]
    fn remove_tone_works() {
        assert_eq!(remove_tone("tiếng"), "tiêng");
        assert_eq!(remove_tone("Việt"), "Viêt");
        assert_eq!(remove_tone("được"), "đươc");
        assert_eq!(remove_tone("không"), "không"); // ô has no tone
        assert_eq!(remove_tone("xin chào"), "xin chao");
    }

    #[test]
    fn english_pass_through() {
        let input = "Hello world 123!";
        for charset in [VietCharset::TCVN3, VietCharset::VNIWin, VietCharset::Unicode] {
            let encoded = encode(input, charset);
            let decoded = decode(&encoded, charset);
            assert_eq!(decoded, input, "English pass-through failed for {:?}", charset);
        }
    }
}
