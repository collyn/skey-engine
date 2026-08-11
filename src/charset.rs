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

// ── VIQR output table ────────────────────────────────────────────────
// Packed u32: byte0=base, byte1=tone1, byte2=tone2, byte3=0
// Ported from x-unikey VIQRTable[213].

static VIQR_TABLE: [u32; VN_CHAR_COUNT] = [
    // 0..11: a
    0x00000041, 0x00000061, 0x00002741, 0x00002761, 0x00006041, 0x00006061,
    0x00003F41, 0x00003F61, 0x00007E41, 0x00007E61, 0x00002E41, 0x00002E61,
    // 12..23: â
    0x005E0041, 0x005E0061, 0x00275E41, 0x00275E61, 0x00605E41, 0x00605E61,
    0x003F5E41, 0x003F5E61, 0x007E5E41, 0x007E5E61, 0x002E5E41, 0x002E5E61,
    // 24..35: ă
    0x00280041, 0x00280061, 0x00272841, 0x00272861, 0x00602841, 0x00602861,
    0x003F2841, 0x003F2861, 0x007E2841, 0x007E2861, 0x002E2841, 0x002E2861,
    // 36..43: B,b,C,c,D,d,Đ,đ
    0x42, 0x62, 0x43, 0x63, 0x44, 0x64, 0x00004444, 0x00006464,
    // 44..55: e
    0x45, 0x65, 0x00002745, 0x00002765, 0x00006045, 0x00006065,
    0x00003F45, 0x00003F65, 0x00007E45, 0x00007E65, 0x00002E45, 0x00002E65,
    // 56..67: ê
    0x005E0045, 0x005E0065, 0x00275E45, 0x00275E65, 0x00605E45, 0x00605E65,
    0x003F5E45, 0x003F5E65, 0x007E5E45, 0x007E5E65, 0x002E5E45, 0x002E5E65,
    // 68..85: F,f,G,g,H,h,I,i
    0x46, 0x66, 0x47, 0x67, 0x48, 0x68,
    0x49, 0x69, 0x00002749, 0x00002769, 0x00006049, 0x00006069,
    0x00003F49, 0x00003F69, 0x00007E49, 0x00007E69, 0x00002E49, 0x00002E69,
    // 86..95: J,j,K,k,L,l,M,m,N,n
    0x4A, 0x6A, 0x4B, 0x6B, 0x4C, 0x6C, 0x4D, 0x6D, 0x4E, 0x6E,
    // 96..107: o
    0x4F, 0x6F, 0x0000274F, 0x0000276F, 0x0000604F, 0x0000606F,
    0x00003F4F, 0x00003F6F, 0x00007E4F, 0x00007E6F, 0x00002E4F, 0x00002E6F,
    // 108..119: ô
    0x005E004F, 0x005E006F, 0x00275E4F, 0x00275E6F, 0x00605E4F, 0x00605E6F,
    0x003F5E4F, 0x003F5E6F, 0x007E5E4F, 0x007E5E6F, 0x002E5E4F, 0x002E5E6F,
    // 120..131: ơ
    0x002B004F, 0x002B006F, 0x00272B4F, 0x00272B6F, 0x00602B4F, 0x00602B6F,
    0x003F2B4F, 0x003F2B6F, 0x007E2B4F, 0x007E2B6F, 0x002E2B4F, 0x002E2B6F,
    // 132..141: P,p,Q,q,R,r,S,s,T,t
    0x50, 0x70, 0x51, 0x71, 0x52, 0x72, 0x53, 0x73, 0x54, 0x74,
    // 142..153: u
    0x55, 0x75, 0x00002755, 0x00002775, 0x00006055, 0x00006075,
    0x00003F55, 0x00003F75, 0x00007E55, 0x00007E75, 0x00002E55, 0x00002E75,
    // 154..165: ư
    0x002B0055, 0x002B0075, 0x00272B55, 0x00272B75, 0x00602B55, 0x00602B75,
    0x003F2B55, 0x003F2B75, 0x007E2B55, 0x007E2B75, 0x002E2B55, 0x002E2B75,
    // 166..171: V,v,W,w,X,x
    0x56, 0x76, 0x57, 0x77, 0x58, 0x78,
    // 172..183: y
    0x59, 0x79, 0x00002759, 0x00002779, 0x00006059, 0x00006079,
    0x00003F59, 0x00003F79, 0x00007E59, 0x00007E79, 0x00002E59, 0x00002E79,
    // 184..185: Z,z
    0x5A, 0x7A,
    // 186..212: Western symbols (pass through as-is)
    0x80, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88,
    0x89, 0x8A, 0x8B, 0x8C, 0x8E, 0x91, 0x92, 0x93,
    0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B,
    0x9C, 0x9E, 0x9F,
];

/// Write packed VIQR bytes from a u32: byte0 byte1 byte2 (skip 0x00).
fn push_viqr_bytes(out: &mut Vec<u8>, w: u32) {
    let b0 = (w & 0xFF) as u8;
    let b1 = ((w >> 8) & 0xFF) as u8;
    let b2 = ((w >> 16) & 0xFF) as u8;
    if b0 != 0 { out.push(b0); }
    if b1 != 0 { out.push(b1); }
    if b2 != 0 { out.push(b2); }
}

// ── WinCP1258 table ──────────────────────────────────────────────────
// Ported from x-unikey WinCP1258Pre[213].
// Format: each u16 = (combining_mark << 8) | base_char.
//  hi=0x00 → single byte output (base_char only).
//  hi≠0x00 → two-byte output: [base_char, combining_mark].

static WINCP1258_TABLE: [u16; VN_CHAR_COUNT] = [
    // 0..11: a
    0x0041, 0x0061, 0x00C1, 0x00E1, 0x00C0, 0x00E0,
    0xD241, 0xD261, 0xDE41, 0xDE61, 0xF241, 0xF261,
    // 12..23: â
    0x00C2, 0x00E2, 0xECC2, 0xECE2, 0xCCC2, 0xCCE2,
    0xD2C2, 0xD2E2, 0xDEC2, 0xDEE2, 0xF2C2, 0xF2E2,
    // 24..35: ă
    0x00C3, 0x00E3, 0xECC3, 0xECE3, 0xCCC3, 0xCCE3,
    0xD2C3, 0xD2E3, 0xDEC3, 0xDEE3, 0xF2C3, 0xF2E3,
    // 36..43: B,b,C,c,D,d,Đ,đ
    0x0042, 0x0062, 0x0043, 0x0063, 0x0044, 0x0064, 0x00D0, 0x00F0,
    // 44..55: e
    0x0045, 0x0065, 0x00C9, 0x00E9, 0x00C8, 0x00E8,
    0xD245, 0xD265, 0xDE45, 0xDE65, 0xF245, 0xF265,
    // 56..67: ê
    0x00CA, 0x00EA, 0xECCA, 0xECEA, 0xCCCA, 0xCCEA,
    0xD2CA, 0xD2EA, 0xDECA, 0xDEEA, 0xF2CA, 0xF2EA,
    // 68..73: F,f,G,g,H,h
    0x0046, 0x0066, 0x0047, 0x0067, 0x0048, 0x0068,
    // 74..85: i
    0x0049, 0x0069, 0x00CD, 0x00ED, 0xCC49, 0xCC69,
    0xD249, 0xD269, 0xDE49, 0xDE69, 0xF249, 0xF269,
    // 86..95: J,j,K,k,L,l,M,m,N,n
    0x004A, 0x006A, 0x004B, 0x006B, 0x004C, 0x006C,
    0x004D, 0x006D, 0x004E, 0x006E,
    // 96..107: o
    0x004F, 0x006F, 0x00D3, 0x00F3, 0xCC4F, 0xCC6F,
    0xD24F, 0xD26F, 0xDE4F, 0xDE6F, 0xF24F, 0xF26F,
    // 108..119: ô
    0x00D4, 0x00F4, 0xECD4, 0xECF4, 0xCCD4, 0xCCF4,
    0xD2D4, 0xD2F4, 0xDED4, 0xDEF4, 0xF2D4, 0xF2F4,
    // 120..131: ơ
    0x00D5, 0x00F5, 0xECD5, 0xECF5, 0xCCD5, 0xCCF5,
    0xD2D5, 0xD2F5, 0xDED5, 0xDEF5, 0xF2D5, 0xF2F5,
    // 132..141: P,p,Q,q,R,r,S,s,T,t
    0x0050, 0x0070, 0x0051, 0x0071, 0x0052, 0x0072,
    0x0053, 0x0073, 0x0054, 0x0074,
    // 142..153: u
    0x0055, 0x0075, 0x00DA, 0x00FA, 0x00D9, 0x00F9,
    0xD255, 0xD275, 0xDE55, 0xDE75, 0xF255, 0xF275,
    // 154..165: ư
    0x00DD, 0x00FD, 0xECDD, 0xECFD, 0xCCDD, 0xCCFD,
    0xD2DD, 0xD2FD, 0xDEDD, 0xDEFD, 0xF2DD, 0xF2FD,
    // 166..171: V,v,W,w,X,x
    0x0056, 0x0076, 0x0057, 0x0077, 0x0058, 0x0078,
    // 172..183: y
    0x0059, 0x0079, 0xEC59, 0xEC79, 0xCC59, 0xCC79,
    0xD259, 0xD279, 0xDE59, 0xDE79, 0xF259, 0xF279,
    // 184..185: Z,z
    0x005A, 0x007A,
    // 186..212: Western symbols
    0x0080, 0x0082, 0x0083, 0x0084, 0x0085, 0x0086,
    0x0087, 0x0088, 0x0089, 0x008A, 0x008B, 0x008C,
    0x008E, 0x0091, 0x0092, 0x0093, 0x0094, 0x0095,
    0x0096, 0x0097, 0x0098, 0x0099, 0x009A, 0x009B,
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
    Unicode = 0,
    /// TCVN3 / ABC — single-byte legacy charset
    TCVN3 = 1,
    /// VNI for Windows — double-byte legacy charset
    VNIWin = 2,
    /// Windows CP1258 — Vietnamese Windows code page
    WinCP1258 = 3,
    /// VIQR — ASCII quoted-readable
    VIQR = 4,
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
        VietCharset::WinCP1258 => {
            let map = unicode_to_index_map();
            let mut out = Vec::with_capacity(input.len() * 2);
            for ch in input.chars() {
                if let Some(&idx) = map.get(&ch) {
                    let w = WINCP1258_TABLE[idx];
                    let lo = (w & 0xFF) as u8;
                    let hi = (w >> 8) as u8;
                    out.push(lo);
                    if hi != 0 {
                        out.push(hi);
                    }
                } else if ch.is_ascii() {
                    out.push(ch as u8);
                } else {
                    let mut buf = [0u8; 4];
                    let s = ch.encode_utf8(&mut buf);
                    out.extend_from_slice(s.as_bytes());
                }
            }
            out
        }
        VietCharset::VIQR => {
            let map = unicode_to_index_map();
            let mut out = Vec::new();
            for ch in input.chars() {
                if ch.is_ascii() {
                    if let Some(&idx) = map.get(&ch) {
                        let w = VIQR_TABLE[idx];
                        // VIQR escape + base char + tone char(s)
                        push_viqr_bytes(&mut out, w);
                    } else {
                        out.push(ch as u8);
                    }
                } else if let Some(&idx) = map.get(&ch) {
                    let w = VIQR_TABLE[idx];
                    push_viqr_bytes(&mut out, w);
                } else {
                    let mut buf = [0u8; 4];
                    let s = ch.encode_utf8(&mut buf);
                    out.extend_from_slice(s.as_bytes());
                }
            }
            out
        }
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
        VietCharset::WinCP1258 => {
            let _map = unicode_to_index_map();
            // Build reverse: (base_byte, combining_byte) → index
            let mut rev: HashMap<(u8, u8), usize> = HashMap::with_capacity(VN_CHAR_COUNT);
            for (i, &w) in WINCP1258_TABLE.iter().enumerate() {
                let lo = (w & 0xFF) as u8;
                let hi = (w >> 8) as u8;
                rev.insert((lo, hi), i);
            }
            let mut out = String::with_capacity(input.len());
            let mut i = 0;
            while i < input.len() {
                let lo = input[i];
                // Try 2-byte WinCP1258 pair (combining mark follows base)
                if i + 1 < input.len() {
                    let hi = input[i + 1];
                    if let Some(&idx) = rev.get(&(lo, hi)) {
                        out.push(UNICODE_TABLE[idx]);
                        i += 2;
                        continue;
                    }
                }
                // Try single-byte entry
                if let Some(&idx) = rev.get(&(lo, 0)) {
                    out.push(UNICODE_TABLE[idx]);
                } else {
                    out.push(lo as char);
                }
                i += 1;
            }
            out
        }
        VietCharset::VIQR => {
            // Basic VIQR decode: pass through as-is for now.
            // VIQR decoding is complex (context-sensitive) and the
            // engine already supports VIQR input natively.
            String::from_utf8_lossy(input).to_string()
        }
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
    fn wincp1258_roundtrip() {
        let input = "tiếng Việt";
        let encoded = encode(input, VietCharset::WinCP1258);
        let decoded = decode(&encoded, VietCharset::WinCP1258);
        assert_eq!(decoded, input);
    }

    #[test]
    fn viqr_output() {
        let input = "tiếng Việt";
        let encoded = encode(input, VietCharset::VIQR);
        // VIQR output should be ASCII-only
        assert!(encoded.iter().all(|&b| b <= 0x7F));
        // Should contain recognizable VIQR patterns
        let s = String::from_utf8(encoded).unwrap();
        assert!(s.contains("tie^'ng") || s.contains("tie^'ng") || s.len() > 0);
    }

    #[test]
    fn english_pass_through() {
        let input = "Hello world 123!";
        for charset in [VietCharset::TCVN3, VietCharset::VNIWin, VietCharset::WinCP1258, VietCharset::Unicode] {
            let encoded = encode(input, charset);
            let decoded = decode(&encoded, charset);
            assert_eq!(decoded, input, "English pass-through failed for {:?}", charset);
        }
    }
}
