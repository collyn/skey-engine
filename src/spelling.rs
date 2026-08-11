//! Vietnamese CVC (Consonant-Vowel-Consonant) spelling validation.
//!
//! Adapted from bamboo-core 0.3.13 spelling.rs — verified correct via
//! unit tests.  The tables are linguistic data (which consonants can
//! precede which vowels, etc.) and are not copyrightable.

// Static token: (chars, length)
type Token = ([char; 4], u8);

static FC_0: &[Token] = &[
    (['b', '\0', '\0', '\0'], 1),
    (['d', '\0', '\0', '\0'], 1),
    (['đ', '\0', '\0', '\0'], 1),
    (['g', '\0', '\0', '\0'], 1),
    (['g', 'h', '\0', '\0'], 2),
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['n', 'h', '\0', '\0'], 2),
    (['p', '\0', '\0', '\0'], 1),
    (['p', 'h', '\0', '\0'], 2),
    (['r', '\0', '\0', '\0'], 1),
    (['s', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
    (['t', 'r', '\0', '\0'], 2),
    (['v', '\0', '\0', '\0'], 1),
    (['z', '\0', '\0', '\0'], 1),
];
static FC_1: &[Token] = &[
    (['c', '\0', '\0', '\0'], 1),
    (['h', '\0', '\0', '\0'], 1),
    (['k', '\0', '\0', '\0'], 1),
    (['k', 'h', '\0', '\0'], 2),
    (['q', 'u', '\0', '\0'], 2),
    (['t', 'h', '\0', '\0'], 2),
];
static FC_2: &[Token] = &[
    (['c', 'h', '\0', '\0'], 2),
    (['g', 'i', '\0', '\0'], 2),
    (['l', '\0', '\0', '\0'], 1),
    (['n', 'g', '\0', '\0'], 2),
    (['n', 'g', 'h', '\0'], 3),
    (['x', '\0', '\0', '\0'], 1),
];
static FC_3: &[Token] = &[(['đ', '\0', '\0', '\0'], 1), (['l', '\0', '\0', '\0'], 1)];
static FC_4: &[Token] = &[(['h', '\0', '\0', '\0'], 1)];
static FC_ROWS: &[&[Token]] = &[FC_0, FC_1, FC_2, FC_3, FC_4];

static VO_0: &[Token] = &[
    (['ê', '\0', '\0', '\0'], 1),
    (['i', '\0', '\0', '\0'], 1),
    (['u', 'a', '\0', '\0'], 2),
    (['u', 'ê', '\0', '\0'], 2),
    (['u', 'y', '\0', '\0'], 2),
    (['y', '\0', '\0', '\0'], 1),
];
static VO_1: &[Token] = &[
    (['a', '\0', '\0', '\0'], 1),
    (['i', 'ê', '\0', '\0'], 2),
    (['o', 'a', '\0', '\0'], 2),
    (['u', 'e', '\0', '\0'], 2),
    (['u', 'y', 'ê', '\0'], 3),
    (['y', 'ê', '\0', '\0'], 2),
];
static VO_2: &[Token] = &[
    (['â', '\0', '\0', '\0'], 1),
    (['ă', '\0', '\0', '\0'], 1),
    (['e', '\0', '\0', '\0'], 1),
    (['o', '\0', '\0', '\0'], 1),
    (['o', 'o', '\0', '\0'], 2),
    (['ô', '\0', '\0', '\0'], 1),
    (['ơ', '\0', '\0', '\0'], 1),
    (['o', 'e', '\0', '\0'], 2),
    (['u', '\0', '\0', '\0'], 1),
    (['ư', '\0', '\0', '\0'], 1),
    (['u', 'â', '\0', '\0'], 2),
    (['u', 'ô', '\0', '\0'], 2),
    (['ư', 'ơ', '\0', '\0'], 2),
];
static VO_3: &[Token] = &[(['o', 'ă', '\0', '\0'], 2)];
static VO_4: &[Token] = &[(['u', 'ơ', '\0', '\0'], 2)];
static VO_5: &[Token] = &[
    (['a', 'i', '\0', '\0'], 2),
    (['a', 'o', '\0', '\0'], 2),
    (['a', 'u', '\0', '\0'], 2),
    (['â', 'u', '\0', '\0'], 2),
    (['a', 'y', '\0', '\0'], 2),
    (['â', 'y', '\0', '\0'], 2),
    (['e', 'o', '\0', '\0'], 2),
    (['ê', 'u', '\0', '\0'], 2),
    (['i', 'a', '\0', '\0'], 2),
    (['i', 'ê', 'u', '\0'], 3),
    (['i', 'u', '\0', '\0'], 2),
    (['o', 'a', 'i', '\0'], 3),
    (['o', 'a', 'o', '\0'], 3),
    (['o', 'a', 'y', '\0'], 3),
    (['o', 'e', 'o', '\0'], 3),
    (['o', 'i', '\0', '\0'], 2),
    (['ô', 'i', '\0', '\0'], 2),
    (['ơ', 'i', '\0', '\0'], 2),
    (['ư', 'a', '\0', '\0'], 2),
    (['u', 'â', 'y', '\0'], 3),
    (['u', 'i', '\0', '\0'], 2),
    (['ư', 'i', '\0', '\0'], 2),
    (['u', 'ô', 'i', '\0'], 3),
    (['ư', 'ơ', 'i', '\0'], 3),
    (['ư', 'ơ', 'u', '\0'], 3),
    (['ư', 'u', '\0', '\0'], 2),
    (['u', 'y', 'a', '\0'], 3),
    (['u', 'y', 'u', '\0'], 3),
    (['y', 'ê', 'u', '\0'], 3),
];
static VO_6: &[Token] = &[(['ă', '\0', '\0', '\0'], 1)];
static VO_7: &[Token] = &[(['i', '\0', '\0', '\0'], 1)];
static VO_ROWS: &[&[Token]] = &[VO_0, VO_1, VO_2, VO_3, VO_4, VO_5, VO_6, VO_7];

static LC_0: &[Token] = &[(['c', 'h', '\0', '\0'], 2), (['n', 'h', '\0', '\0'], 2)];
static LC_1: &[Token] = &[(['c', '\0', '\0', '\0'], 1), (['n', 'g', '\0', '\0'], 2)];
static LC_2: &[Token] = &[
    (['m', '\0', '\0', '\0'], 1),
    (['n', '\0', '\0', '\0'], 1),
    (['p', '\0', '\0', '\0'], 1),
    (['t', '\0', '\0', '\0'], 1),
];
static LC_3: &[Token] = &[(['k', '\0', '\0', '\0'], 1)];
static LC_4: &[Token] = &[(['c', '\0', '\0', '\0'], 1)];
static LC_ROWS: &[&[Token]] = &[LC_0, LC_1, LC_2, LC_3, LC_4];

const CV_ALLOWED_MASKS: [u16; 5] = [
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 5),
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5),
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 5),
    1 << 6,
    1 << 7,
];

const VC_ALLOWED_MASKS: [u16; 8] = [
    (1 << 0) | (1 << 1) | (1 << 2),  // ê,i,ua,uê,uy,y + {ch,nh, c,ng, m,n,p,t}
    (1 << 0) | (1 << 1) | (1 << 2),  // a,iê,oa,ue,uyê,yê + {ch,nh, c,ng, m,n,p,t}
    (1 << 0) | (1 << 1) | (1 << 2),  // â,ă,e,o,oo,ô,ơ,oe,u,ư,uâ,uô,ươ + {ch,nh, c,ng, m,n,p,t}
    (1 << 1) | (1 << 2),             // oă + {c,ng, m,n,p,t}
    0,                                // uơ — no suffix
    0,                                // diphthongs/triphthongs — no suffix
    1 << 3,                           // ă + {k}
    1 << 4,                           // i + {c}
];

// ── Utility ────────────────────────────────────────────────────────

pub fn lower(c: char) -> char { c.to_lowercase().next().unwrap_or(c) }

/// Returns the toneless base vowel for a character.
/// e.g. 'ấ' → 'â', 'ạ' → 'a', 'ê' → 'ê'
pub fn toneless_base(c: char) -> char {
    // Map of all Vietnamese vowel characters → their toneless base
    match lower(c) {
        'a' | 'à' | 'á' | 'ả' | 'ã' | 'ạ' => 'a',
        'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
        'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
        'e' | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'ê',
        'i' | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'o' | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
        'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
        'u' | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'ư',
        'y' | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        other => other,
    }
}

/// Returns the tone number (0=none, 1=grave, 2=acute, 3=hook, 4=tilde, 5=dot)
pub fn tone_of(c: char) -> u8 {
    match lower(c) {
        'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => 1,
        'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => 2,
        'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => 3,
        'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => 4,
        'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => 5,
        _ => 0,
    }
}

/// Applies tone `t` to the toneless base character `c`.
pub fn apply_tone(c: char, t: u8) -> char {
    let base = toneless_base(c);
    let table: &[(char, [char; 6])] = &[
        ('a', ['a', 'à', 'á', 'ả', 'ã', 'ạ']),
        ('ă', ['ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ']),
        ('â', ['â', 'ầ', 'ấ', 'ẩ', 'ẫ', 'ậ']),
        ('e', ['e', 'è', 'é', 'ẻ', 'ẽ', 'ẹ']),
        ('ê', ['ê', 'ề', 'ế', 'ể', 'ễ', 'ệ']),
        ('i', ['i', 'ì', 'í', 'ỉ', 'ĩ', 'ị']),
        ('o', ['o', 'ò', 'ó', 'ỏ', 'õ', 'ọ']),
        ('ô', ['ô', 'ồ', 'ố', 'ổ', 'ỗ', 'ộ']),
        ('ơ', ['ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ']),
        ('u', ['u', 'ù', 'ú', 'ủ', 'ũ', 'ụ']),
        ('ư', ['ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự']),
        ('y', ['y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ']),
    ];
    for &(b, ref tones) in table {
        if base == b {
            let ti = (t as usize).min(5);
            let ch = tones[ti];
            return if c.is_uppercase() { ch.to_uppercase().next().unwrap_or(ch) } else { ch };
        }
    }
    c
}

/// Applies mark `m` to character `c`.  Marks: 1=hat, 2=breve, 3=horn, 4=dash.
pub fn apply_mark(c: char, m: u8) -> char {
    let base = toneless_base(lower(c));
    let result = match (base, m) {
        ('a', 1) => 'â', ('a', 2) => 'ă',
        ('e', 1) => 'ê',
        ('o', 1) => 'ô', ('o', 3) => 'ơ',
        ('u', 3) => 'ư', ('u', 1) => 'ư', // uw→ư handled by pair transform
        ('d', 4) => 'đ',
        _ => return c,
    };
    let tone = tone_of(c);
    let result = if tone > 0 { apply_tone(result, tone) } else { result };
    if c.is_uppercase() { result.to_uppercase().next().unwrap_or(result) } else { result }
}

pub fn is_vowel(c: char) -> bool {
    matches!(lower(c),
        'a' | 'à' | 'á' | 'ả' | 'ã' | 'ạ'
        | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ'
        | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ'
        | 'e' | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ'
        | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ'
        | 'i' | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị'
        | 'o' | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ'
        | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ'
        | 'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ'
        | 'u' | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ'
        | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự'
        | 'y' | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ')
}

// ── Bitmask lookup ─────────────────────────────────────────────────

fn lookup_mask(rows: &[&[Token]], input: &[char], full: bool, complete: bool) -> u16 {
    let input_len = input.len() as u8;
    let mut ret = 0u16;
    for (index, tokens) in rows.iter().enumerate() {
        for (t_chars, t_len) in *tokens {
            if *t_len < input_len { continue; }
            if full && *t_len > input_len { continue; }
            let mut is_match = true;
            for i in 0..input.len() {
                let ic = input[i];
                let tc = t_chars[i];
                if ic != tc && (complete || toneless_base(tc) != toneless_base(ic)) {
                    is_match = false;
                    break;
                }
            }
            if is_match {
                ret |= 1u16 << index;
                break;
            }
        }
    }
    ret
}

fn is_valid_cv(fc_mask: u16, vo_mask: u16) -> bool {
    let mut mask = fc_mask;
    while mask != 0 {
        let idx = mask.trailing_zeros() as usize;
        if idx < CV_ALLOWED_MASKS.len() && (CV_ALLOWED_MASKS[idx] & vo_mask) != 0 {
            return true;
        }
        mask &= mask - 1;
    }
    false
}

fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
    let mut mask = vo_mask;
    while mask != 0 {
        let idx = mask.trailing_zeros() as usize;
        if idx < VC_ALLOWED_MASKS.len() && (VC_ALLOWED_MASKS[idx] & lc_mask) != 0 {
            return true;
        }
        mask &= mask - 1;
    }
    false
}

// ── CVC split ──────────────────────────────────────────────────────

/// Splits a Vietnamese syllable string into first consonant, vowel, last consonant.
fn split_cvc(s: &str) -> (Vec<char>, Vec<char>, Vec<char>) {
    let chars: Vec<char> = s.chars().map(|c| toneless_base(lower(c))).collect();
    if chars.is_empty() {
        return (vec![], vec![], vec![]);
    }

    // Scan from end to find vowel → lc boundary
    let mut lc_start = chars.len();
    while lc_start > 0 {
        if is_vowel(chars[lc_start - 1]) { break; }
        lc_start -= 1;
    }

    // Scan from vowel boundary back to find fc start
    let mut vo_start = lc_start;
    while vo_start > 0 {
        if !is_vowel(chars[vo_start - 1]) { break; }
        vo_start -= 1;
    }

    let mut fc: Vec<char> = chars[..vo_start].to_vec();
    let mut vo: Vec<char> = chars[vo_start..lc_start].to_vec();
    let lc: Vec<char> = chars[lc_start..].to_vec();

    // "gi" + vowel: move 'i' from vo to fc (gi is a consonant digraph FC_2)
    // gi + i (without a separate following vowel) is not valid Vietnamese:
    //   "gì"  — ok, no consonant suffix
    //   "gim" — invalid, gi + i + consonant
    if !vo.is_empty() && fc.len() == 1 && fc[0] == 'g' && vo[0] == 'i' {
        fc.push(vo.remove(0)); // move 'i' from vowel to first consonant
        if vo.is_empty() && !lc.is_empty() {
            return (fc, vo, lc); // invalid: gi+i+consonant, caught below
        }
    }

    // "qu" + vowel: move 'u' from vo to fc (qu is a consonant digraph FC_1)
    // qu + u (without a separate following vowel) is not valid Vietnamese
    if !vo.is_empty() && fc.len() == 1 && fc[0] == 'q' && vo[0] == 'u' {
        fc.push(vo.remove(0)); // move 'u' from vowel to first consonant
        if vo.is_empty() && !lc.is_empty() {
            return (fc, vo, lc); // invalid: qu+u+consonant, caught below
        }
    }

    (fc, vo, lc)
}

/// Public CVC validation entry point.
///
/// Returns `true` if `s` can be parsed as a valid Vietnamese syllable.
/// `s` should already be in composed form (with diacritics applied).
pub fn is_valid_cvc(s: &str) -> bool {
    if s.is_empty() || s.len() <= 1 {
        return s.len() <= 1; // empty or single char is always "valid" (partial input)
    }

    let (fc, vo, lc) = split_cvc(s);

    // gi + i (+ consonant) → invalid: gi digraph requires a different vowel
    // qu + u (+ consonant) → invalid: qu digraph requires a different vowel
    if vo.is_empty() && !lc.is_empty()
        && ((fc.len() == 2 && fc[0] == 'g' && fc[1] == 'i')
            || (fc.len() == 2 && fc[0] == 'q' && fc[1] == 'u'))
    {
        return false;
    }

    // k only combines with e, ê, i, y (and their extended sequences: eo, êu, ia,
    // iê, iêu).  Pattern from x-unikey's isValidCV.
    if fc.len() == 1 && fc[0] == 'k' && !vo.is_empty() {
        let first_vowel = vo[0];
        if !matches!(first_vowel, 'e' | 'ê' | 'i' | 'y') {
            return false;
        }
    }

    // Debug: uncomment to trace CVC split
    // eprintln!("is_valid_cvc({s:?}): fc={fc:?} vo={vo:?} lc={lc:?}");

    let fc_mask = if !fc.is_empty() {
        let m = lookup_mask(FC_ROWS, &fc, !vo.is_empty(), true);
        if m == 0 { return false; }
        m
    } else { 0 };

    let vo_mask = if !vo.is_empty() {
        let m = lookup_mask(VO_ROWS, &vo, !lc.is_empty(), false);
        if m == 0 { return false; }
        m
    } else { 0 };

    let lc_mask = if !lc.is_empty() {
        let m = lookup_mask(LC_ROWS, &lc, false, true);
        if m == 0 { return false; }
        m
    } else { 0 };

    if vo_mask == 0 {
        return fc_mask != 0;
    }
    if fc_mask != 0 {
        if !is_valid_cv(fc_mask, vo_mask) { return false; }
        if lc_mask == 0 { return true; }
    }
    if lc_mask != 0 {
        return is_valid_vc(vo_mask, lc_mask);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_syllables() {
        assert!(is_valid_cvc("ba"));
        assert!(is_valid_cvc("con"));
        assert!(is_valid_cvc("không"));
        assert!(is_valid_cvc("người"));
        assert!(is_valid_cvc("được"));
        assert!(is_valid_cvc("trường"));
        assert!(is_valid_cvc("buồn"));
        assert!(is_valid_cvc("đẹp"));
        assert!(is_valid_cvc("bâ"));
        assert!(is_valid_cvc("mê"));
        assert!(is_valid_cvc("că"));
        assert!(is_valid_cvc("mơ"));
        assert!(is_valid_cvc("ngoăn"));
    }

    #[test]
    fn invalid_english_words() {
        assert!(!is_valid_cvc("wôd"));      // wood
        assert!(!is_valid_cvc("rebôt"));    // reboot
        assert!(!is_valid_cvc("ôk"));       // ook
        assert!(!is_valid_cvc("vâi"));      // vaai
        assert!(!is_valid_cvc("alô"));      // aloo
        assert!(!is_valid_cvc("world"));    // world
        assert!(!is_valid_cvc("wỏld"));     // world (with tone)
    }

    #[test]
    fn đ_abbreviations() {
        assert!(!is_valid_cvc("vcđ"));
        assert!(!is_valid_cvc("đc"));
        assert!(!is_valid_cvc("nđm"));
        assert!(!is_valid_cvc("avcđ"));
        assert!(!is_valid_cvc("bcđ"));
    }
}

#[test]
fn debug_gi_split() {
    let (fc, vo, lc) = split_cvc("giờ");
    eprintln!("giờ: fc={fc:?} vo={vo:?} lc={lc:?}");
    eprintln!("valid: {}", is_valid_cvc("giờ"));
}
