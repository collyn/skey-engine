//! Vietnamese IME engine — Letter-based accumulator model.
//! Each keystroke produces a `Letter` struct. Modifier keys (w, tone, z)
//! mutate the last matching vowel in place. UTF-8 output via precomputed tables.

use crate::tables;

#[derive(Clone, Copy, Default)]
struct Letter {
    c: char,
    is_vowel: bool, variant: u8, tone: u8, is_dstroke: bool, upper: bool,
}

impl Letter {
    fn new(c: char, upper: bool) -> Self {
        let lc = c.to_ascii_lowercase();
        Letter { c: lc, is_vowel: tables::is_vowel(lc), variant: 0, tone: 0, is_dstroke: false, upper }
    }
}

fn emit(ls: &[Letter]) -> String {
    let mut out = String::with_capacity(ls.len() * 3);
    for lt in ls {
        if lt.is_dstroke {
            out.push_str(if lt.upper { tables::D_STROKE_UP } else { tables::D_STROKE_LO });
        } else if lt.is_vowel {
            out.push_str(tables::vowel_utf8(lt.c, lt.variant, lt.tone, lt.upper));
        } else {
            out.push(if lt.upper { lt.c.to_ascii_uppercase() } else { lt.c });
        }
    }
    out
}

fn last_vowel_index(ls: &[Letter]) -> Option<usize> { ls.iter().rposition(|lt| lt.is_vowel) }
fn last_d_index(ls: &[Letter]) -> Option<usize> {
    ls.iter().rposition(|lt| lt.c == 'd' && !lt.is_vowel && !lt.is_dstroke)
}

/// Find the last vowel that can be modified by `w`: a→ă, o→ơ, u→ư, e→ê.
/// Used by the `w` key handler so that e.g. "toi" + w finds 'o' (not 'i').
fn last_w_target_index(ls: &[Letter]) -> Option<usize> {
    ls.iter().rposition(|lt| lt.is_vowel && matches!(lt.c, 'a' | 'o' | 'u' | 'e'))
}

fn tone_vowel_index(ls: &[Letter], _modern: bool) -> Option<usize> {
    let end = last_vowel_index(ls)?;
    let mut start = end;
    while start > 0 && ls[start - 1].is_vowel { start -= 1; }
    if start < end {
        if ls[start].c == 'u' && start > 0 && ls[start - 1].c == 'q' { start += 1; }
        else if ls[start].c == 'i' && start > 0 && ls[start - 1].c == 'g' { start += 1; }
    }
    if start > end { start = end; }
    for i in (start..=end).rev() { if ls[i].variant != 0 { return Some(i); } }
    let count = end - start + 1;
    if count == 1 { return Some(start); }
    if count >= 3 {
        if ls[start].c == 'u' && start + 2 <= end && ls[start + 1].c == 'y' && ls[start + 2].c == 'e'
        { return Some(start + 2); }
        return Some(start + 1);
    }
    if end + 1 < ls.len() { return Some(end); }
    let (c1, c2) = (ls[start].c, ls[end].c);
    if (c1 == 'o' && c2 == 'a') || (c1 == 'o' && c2 == 'e') || (c1 == 'u' && c2 == 'y') {
        return Some(end); // always 2nd vowel — use typing order for òa (ofa) vs oà (oaf)
    }
    Some(start) // default open diphthong: tone on first vowel
}

fn telex_tone(c: char) -> Option<u8> {
    match c.to_ascii_lowercase() { 's'=>Some(2),'f'=>Some(1),'r'=>Some(3),'x'=>Some(4),'j'=>Some(5), _=>None }
}
fn vni_tone(c: char) -> Option<u8> {
    match c { '1'=>Some(2),'2'=>Some(1),'3'=>Some(3),'4'=>Some(4),'5'=>Some(5), _=>None }
}

pub fn convert_telex(input: &str, modern: bool, short_w: bool) -> String {
    let mut ls: Vec<Letter> = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let lc = ch.to_ascii_lowercase(); let upper = ch.is_ascii_uppercase(); let n = ls.len();
        if lc == 'd' && n > 0 && ls[n-1].c == 'd' && !ls[n-1].is_vowel && !ls[n-1].is_dstroke
        { ls[n-1].is_dstroke = true; if upper { ls[n-1].upper = true; } continue; }
        // Double-vowel circumflex: search backwards through vowel cluster.
        // Toggle behavior (Unikey): oo→ô, ooo→oo (3rd press undoes circumflex)
        if tables::is_vowel(lc) && matches!(lc, 'a'|'e'|'o') && n > 0 {
            let mut consumed = false;
            for i in (0..n).rev() {
                if ls[i].is_vowel {
                    if ls[i].c == lc {
                        if ls[i].variant == 0 { ls[i].variant = 1; if upper { ls[i].upper = true; } consumed = true; break; }
                        if ls[i].variant == 1 { ls[i].variant = 0; break; } // toggle off, then push copy
                    }
                } else { break; }
            }
            if consumed { continue; }
        }
        if lc == 'w' {
            if let Some(vi) = last_w_target_index(&ls) {
                match ls[vi].c {
                    'a' => { ls[vi].variant = 2; continue; }
                    'o' => { ls[vi].variant = 2; if vi > 0 && ls[vi-1].c == 'u' { ls[vi-1].variant = 2; } continue; }
                    'u' => { ls[vi].variant = 2; continue; }
                    'e' => { ls[vi].variant = 1; continue; }
                    _ => {}
                }
            } else if short_w {
                // short_w mode: standalone w → ư
                ls.push(Letter { c: 'u', is_vowel: true, variant: 2, tone: 0, is_dstroke: false, upper });
                continue;
            }
        }
        if let Some(t) = telex_tone(lc) {
            if let Some(vi) = tone_vowel_index(&ls, modern) {
                ls[vi].tone = t; continue; // replace existing tone
            }
        }
        if lc == 'z' {
            if let Some(vi) = tone_vowel_index(&ls, modern) {
                if ls[vi].tone != 0 { ls[vi].tone = 0; continue; }
            }
        }
        ls.push(Letter::new(ch, upper));
    }
    emit(&ls)
}

/// Combined Telex + VNI mode.
/// Accepts BOTH Telex and VNI keystrokes in one stream:
///   - VNI digits (0-9) apply VNI tone/marks to the last vowel
///   - Telex markers (w, s/f/r/x/j/z, and double-vowel ^/breve) apply Telex
///   - everything else is a normal letter
pub fn convert_teip_vni(input: &str, modern: bool, short_w: bool) -> String {
    let mut ls: Vec<Letter> = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let lc = ch.to_ascii_lowercase(); let upper = ch.is_ascii_uppercase(); let n = ls.len();

        // ── VNI digit rules ──
        if let Some(t) = vni_tone(lc) {
            if let Some(vi) = tone_vowel_index(&ls, modern) { ls[vi].tone = t; continue; }
        }
        if lc >= '0' && lc <= '9' {
            match lc {
                '6' => { if let Some(vi) = last_vowel_index(&ls) { if matches!(ls[vi].c, 'a'|'e'|'o') { ls[vi].variant = 1; continue; } } }
                '7' => { if let Some(vi) = last_vowel_index(&ls) { if matches!(ls[vi].c, 'o'|'u') { ls[vi].variant = 2; if ls[vi].c == 'o' && vi > 0 && ls[vi-1].c == 'u' { ls[vi-1].variant = 2; } continue; } } }
                '8' => { if let Some(vi) = last_vowel_index(&ls) { if ls[vi].c == 'a' { ls[vi].variant = 2; continue; } } }
                '9' => { if let Some(di) = last_d_index(&ls) { ls[di].is_dstroke = true; if upper { ls[di].upper = true; } } continue; }
                '0' => { continue; }
                _ => {} // digits 1-5 already handled by vni_tone above
            }
        }

        // ── Telex dd → đ ──
        if lc == 'd' && n > 0 && ls[n-1].c == 'd' && !ls[n-1].is_vowel && !ls[n-1].is_dstroke
        { ls[n-1].is_dstroke = true; if upper { ls[n-1].upper = true; } continue; }

        // ── Telex circumflex: double same vowel ──
        // Double-vowel circumflex: search backwards through vowel cluster.
        // Toggle behavior (Unikey): oo→ô, ooo→oo (3rd press undoes circumflex)
        if tables::is_vowel(lc) && matches!(lc, 'a'|'e'|'o') && n > 0 {
            let mut consumed = false;
            for i in (0..n).rev() {
                if ls[i].is_vowel {
                    if ls[i].c == lc {
                        if ls[i].variant == 0 { ls[i].variant = 1; if upper { ls[i].upper = true; } consumed = true; break; }
                        if ls[i].variant == 1 { ls[i].variant = 0; break; } // toggle off, then push copy
                    }
                } else { break; }
            }
            if consumed { continue; }
        }

        // ── Telex w (breve/horn) ──
        if lc == 'w' {
            if let Some(vi) = last_w_target_index(&ls) {
                match ls[vi].c {
                    'a' => { ls[vi].variant = 2; continue; }
                    'o' => { ls[vi].variant = 2; if vi > 0 && ls[vi-1].c == 'u' { ls[vi-1].variant = 2; } continue; }
                    'u' => { ls[vi].variant = 2; continue; }
                    'e' => { ls[vi].variant = 1; continue; }
                    _ => {}
                }
            } else if short_w {
                ls.push(Letter { c: 'u', is_vowel: true, variant: 2, tone: 0, is_dstroke: false, upper });
                continue;
            }
        }

        // ── Telex tone marks ──
        if let Some(t) = telex_tone(lc) {
            if let Some(vi) = tone_vowel_index(&ls, modern) {
                ls[vi].tone = t; continue; // replace existing tone
            }
        }
        if lc == 'z' {
            if let Some(vi) = tone_vowel_index(&ls, modern) {
                if ls[vi].tone != 0 { ls[vi].tone = 0; continue; }
            }
        }

        ls.push(Letter::new(ch, upper));
    }
    emit(&ls)
}

/// VIQR (Vietnamese Quoted-Readable) conversion.
///
/// Uses ASCII punctuation as diacritic markers:
///   ^ circumflex, ( breve, + horn (modifiers)
///   ' sac, ` huyen, ? hoi, ~ nga, . nang (tones)
///   dd -> đ d-stroke
pub fn convert_viqr(input: &str) -> String {
    let mut ls: Vec<Letter> = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let lc = ch.to_ascii_lowercase(); let n = ls.len();

        // Modifier characters: apply to last vowel
        match ch {
            '^' | '(' | '+' => {
                if let Some(vi) = last_vowel_index(&ls) {
                    match (ch, ls[vi].c) {
                        ('^', 'a' | 'e' | 'o') => { ls[vi].variant = 1; continue; }
                        ('(', 'a') => { ls[vi].variant = 2; continue; }
                        ('+', 'o' | 'u') => { ls[vi].variant = 2; continue; }
                        _ => {} // illegal combination: fall through to literal
                    }
                }
                // Fall through: treat as literal character
                ls.push(Letter { c: ch, is_vowel: false, variant: 0, tone: 0, is_dstroke: false, upper: false });
                continue;
            }
            _ => {}
        }

        // Tone characters
        match ch {
            '\'' => { if let Some(vi) = last_vowel_index(&ls) { ls[vi].tone = 2; continue; } }
            '`'  => { if let Some(vi) = last_vowel_index(&ls) { ls[vi].tone = 1; continue; } }
            '?'  => { if let Some(vi) = last_vowel_index(&ls) { ls[vi].tone = 3; continue; } }
            '~'  => { if let Some(vi) = last_vowel_index(&ls) { ls[vi].tone = 4; continue; } }
            '.'  => { if let Some(vi) = last_vowel_index(&ls) { ls[vi].tone = 5; continue; } }
            _ => {}
        }

        // dd → đ
        if lc == 'd' && n > 0 && ls[n-1].c == 'd' && !ls[n-1].is_vowel && !ls[n-1].is_dstroke {
            ls[n-1].is_dstroke = true;
            if ch.is_ascii_uppercase() { ls[n-1].upper = true; }
            continue;
        }

        ls.push(Letter::new(ch, ch.is_ascii_uppercase()));
    }
    emit(&ls)
}

pub fn convert_vni(input: &str, modern: bool, _short_w: bool) -> String {
    let mut ls: Vec<Letter> = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let lc = ch.to_ascii_lowercase(); let upper = ch.is_ascii_uppercase();
        if let Some(t) = vni_tone(lc) {
            if let Some(vi) = tone_vowel_index(&ls, modern) { ls[vi].tone = t; continue; }
        }
        match lc {
            '6' => { if let Some(vi) = last_vowel_index(&ls) { if matches!(ls[vi].c, 'a'|'e'|'o') { ls[vi].variant = 1; continue; } } }
            '7' => { if let Some(vi) = last_vowel_index(&ls) { if matches!(ls[vi].c, 'o'|'u') { ls[vi].variant = 2; if ls[vi].c == 'o' && vi > 0 && ls[vi-1].c == 'u' { ls[vi-1].variant = 2; } continue; } } }
            '8' => { if let Some(vi) = last_vowel_index(&ls) { if ls[vi].c == 'a' { ls[vi].variant = 2; continue; } } }
            '9' => { if let Some(di) = last_d_index(&ls) { ls[di].is_dstroke = true; if upper { ls[di].upper = true; } } continue; }
            '0' => { continue; }
            _ => {}
        }
        if lc.is_ascii_alphabetic() || (lc.is_ascii_digit() && !ls.is_empty()) { ls.push(Letter::new(ch, upper)); }
    }
    emit(&ls)
}


#[cfg(test)]
mod tests {
    use super::*;
    // modern=true, short_w=false (current default behavior)
    fn t(s: &str) -> String { convert_telex(s, true, false) }
    fn tw(s: &str) -> String { convert_telex(s, true, true) }
    // traditional tone placement (short_w enabled)
    fn tt(s: &str) -> String { convert_telex(s, false, true) }
    fn vt(s: &str) -> String { convert_vni(s, false, true) }

    #[test] fn telex_basic() {
        assert_eq!(t("as"),"á"); assert_eq!(t("af"),"à"); assert_eq!(t("ar"),"ả"); assert_eq!(t("ax"),"ã"); assert_eq!(t("aj"),"ạ");
        assert_eq!(t("aw"),"ă"); assert_eq!(t("aa"),"â"); assert_eq!(t("ee"),"ê"); assert_eq!(t("oo"),"ô"); assert_eq!(t("ow"),"ơ");
        assert_eq!(t("uw"),"ư"); assert_eq!(t("dd"),"đ");
    }
    #[test] fn telex_compound() {
        assert_eq!(t("aws"),"ắ"); assert_eq!(t("aas"),"ấ"); assert_eq!(t("ees"),"ế"); assert_eq!(t("oos"),"ố"); assert_eq!(t("ows"),"ớ"); assert_eq!(t("uws"),"ứ");
    }
    #[test] fn telex_words() {
        assert_eq!(t("xin"),"xin"); assert_eq!(t("tooi"),"tôi"); assert_eq!(t("laf"),"là"); assert_eq!(t("cos"),"có");
        assert_eq!(t("khoong"),"không"); assert_eq!(t("tieengs"),"tiếng"); assert_eq!(t("Vieetj"),"Việt");
    }
    #[test] fn telex_uow() {
        assert_eq!(t("uow"),"ươ"); assert_eq!(t("cuowcs"),"cước"); assert_eq!(t("dduwowcj"),"được"); assert_eq!(t("nguwowif"),"người");
    }
    #[test] fn telex_gi_qu() {
        assert_eq!(t("gif"),"gì"); assert_eq!(t("giowf"),"giờ"); assert_eq!(t("gioir"),"giỏi"); assert_eq!(t("gias"),"giá"); assert_eq!(t("quas"),"quá");
    }
    #[test] fn telex_tone_placement() {
        assert_eq!(t("chafo"),"chào"); assert_eq!(t("toans"),"toán"); assert_eq!(t("hoaf"),"hoà"); assert_eq!(t("thowif"),"thời");
        assert_eq!(t("cuar"),"của"); assert_eq!(t("thoix"),"thõi");
        // òa/oà both accessible via typing order:
        assert_eq!(t("ofa"),"òa");  // tone between vowels → on first vowel
        assert_eq!(t("oaf"),"oà");  // tone after all vowels → on second vowel
    }
    #[test] fn telex_english() {
        assert_eq!(t("hello"),"hello");
        assert_eq!(t("wood"),"wôd");
        assert_eq!(t("reboot"),"rebôt");
    }
    #[test] fn telex_z() { assert_eq!(t("asz"),"a"); }
    #[test] fn telex_w_standalone() {
        assert_eq!(t("w"),"w");     // without short_w: w stays as w
        assert_eq!(tw("w"),"ư");    // with short_w: standalone w → ư
        assert_eq!(tw("ws"),"ứ");
        assert_eq!(tw("wf"),"ừ");
        assert_eq!(tw("wn"),"ưn");
    }

    // ── traditional tone placement (modern=false, short_w=true) ─────────────

    #[test] fn telex_dd_traditional() {
        assert_eq!(tt("dd"),"đ"); assert_eq!(tt("DD"),"Đ");
        assert_eq!(tt("Dd"),"Đ"); assert_eq!(tt("dD"),"Đ");
    }
    #[test] fn telex_marks_traditional() {
        assert_eq!(tt("aa"),"â"); assert_eq!(tt("aw"),"ă");
        assert_eq!(tt("ee"),"ê"); assert_eq!(tt("ew"),"ê");
        assert_eq!(tt("oo"),"ô"); assert_eq!(tt("ow"),"ơ");
        assert_eq!(tt("uw"),"ư"); assert_eq!(tt("uow"),"ươ");
    }
    #[test] fn telex_w_traditional() {
        assert_eq!(tt("w"),"ư"); assert_eq!(tt("W"),"Ư");
        assert_eq!(tt("wn"),"ưn"); assert_eq!(tt("ws"),"ứ"); assert_eq!(tt("wf"),"ừ");
    }
    #[test] fn telex_compound_traditional() {
        assert_eq!(tt("aws"),"ắ"); assert_eq!(tt("aas"),"ấ"); assert_eq!(tt("ees"),"ế");
        assert_eq!(tt("oos"),"ố"); assert_eq!(tt("ows"),"ớ"); assert_eq!(tt("uws"),"ứ");
    }
    #[test] fn telex_words_traditional() {
        assert_eq!(tt("chaof"),"chào"); assert_eq!(tt("vieejt"),"việt");
        assert_eq!(tt("tieengs"),"tiếng"); assert_eq!(tt("quas"),"quá");
        assert_eq!(tt("gias"),"giá"); assert_eq!(tt("xin"),"xin");
        assert_eq!(tt("tooi"),"tôi"); assert_eq!(tt("laf"),"là"); assert_eq!(tt("cos"),"có");
        assert_eq!(tt("khoong"),"không"); assert_eq!(tt("Vieetj"),"Việt");
    }
    #[test] fn telex_tone_traditional() {
        // traditional: tone on 2nd vowel for oa/oe/uy open diphthongs
        assert_eq!(tt("toans"),"toán"); assert_eq!(tt("hoaf"),"hoà");
        assert_eq!(tt("thowif"),"thời"); assert_eq!(tt("cuar"),"của");
    }
    #[test] fn telex_gi_qu_traditional() {
        assert_eq!(tt("gif"),"gì"); assert_eq!(tt("giowf"),"giờ");
        assert_eq!(tt("gioir"),"giỏi");
    }
    #[test] fn telex_uow_traditional() {
        assert_eq!(tt("cuowcs"),"cước"); assert_eq!(tt("dduwowcj"),"được");
        assert_eq!(tt("nguwowif"),"người");
    }
    #[test] fn telex_z_traditional() { assert_eq!(tt("asz"),"a"); }
    #[test] fn telex_english_traditional() { assert_eq!(tt("hello"),"hello"); }
    #[test] fn telex_more_traditional() {
        assert_eq!(tt("hoanf"),"hoàn"); assert_eq!(tt("huyeenf"),"huyền");
        assert_eq!(tt("nguyeexn"),"nguyễn"); assert_eq!(tt("ngoaif"),"ngoài");
        assert_eq!(tt("thuowngf"),"thường");
    }
    #[test] fn vni_marks_traditional() {
        assert_eq!(vt("d9"),"đ"); assert_eq!(vt("a6"),"â"); assert_eq!(vt("a8"),"ă");
        assert_eq!(vt("e6"),"ê"); assert_eq!(vt("o6"),"ô"); assert_eq!(vt("o7"),"ơ");
        assert_eq!(vt("u7"),"ư"); assert_eq!(vt("uo7"),"ươ");
    }
    #[test] fn vni_words_traditional() {
        assert_eq!(vt("vie65t"),"việt"); assert_eq!(vt("tie61ng"),"tiếng");
        assert_eq!(vt("qua1"),"quá"); assert_eq!(vt("gia1"),"giá");
    }

    // ── Tone replacement tests ─────────────────────────────────────
    #[test] fn telex_tone_replace() {
        assert_eq!(tt("saosf"),"sào");  // sáo + f → sào (acute→grave)
        assert_eq!(tt("saojs"),"sáo"); // sạ + s → sáo (dot→acute)
        assert_eq!(tt("saojr"),"sảo"); // sạo + r → sảo (dot→hook)
        assert_eq!(tt("hoasf"),"hoà"); // hóa + f → hoà (acute→grave)
        assert_eq!(tt("toansr"),"toản");// toán + r → toản (acute→hook)
    }
    #[test] fn teipvni_tone_replace() {
        assert_eq!(tv("saosf"),"sào");  // Telex acute→grave in combined mode
        assert_eq!(tv("sao1f"),"sào");  // VNI acute + Telex grave
    }

    // ── Unikey-style triple-press toggle ────────────────────────────
    #[test] fn telex_triple_toggle() {
        assert_eq!(tt("rooot"),"root");  // oo→ô, ooo→oo
        assert_eq!(tt("roo"),"rô");
        assert_eq!(tt("aa"),"â");        // double → circumflex
        assert_eq!(tt("aaa"),"aa");      // triple → undo
        assert_eq!(tt("aaaa"),"aâ");     // quadruple → circumflex again
        assert_eq!(tt("ee"),"ê");
        assert_eq!(tt("eee"),"ee");
        assert_eq!(tt("oooo"),"oô");     // oo→ô, ooo→oo, oooo→oô
    }

    // ── Unikey-style double-vowel across y ──────────────────────────
    #[test] fn telex_double_vowel_across_y() {
        assert_eq!(tt("vaya"),"vây");   // vay + a → vây
        assert_eq!(tt("vayaj"),"vậy");  // vay + a + j → vậy
        assert_eq!(tt("vayas"),"vấy");  // vay + a + s → vấy
        assert_eq!(tt("vayaf"),"vầy");  // vay + a + f → vầy
        assert_eq!(tt("mayas"),"mấy");  // may + a + s → mấy
        assert_eq!(tt("dayaj"),"dậy");  // day + a + j → dậy
        assert_eq!(tt("quaya"),"quây"); // quay + a → quây
        assert_eq!(tt("quayar"),"quẩy");// quay + a + r → quẩy
        // au + a → âu
        assert_eq!(tt("saua"),"sâu");
        assert_eq!(tt("sauas"),"sấu");
        assert_eq!(tt("dauaf"),"dầu");
        // ee still works (immediate neighbor)
        assert_eq!(tt("ee"),"ê");
        // aay still works
        assert_eq!(tt("aay"),"ây");
    }

    // ── w-target tests: w applies to last modifiable vowel ──────────
    #[test] fn telex_w_target() {
        // toi+w: w modifies 'o' (last modifiable), not 'i'
        assert_eq!(tt("toiw"),"tơi");
        assert_eq!(tt("toiws"),"tới");  // toi + w + sắc → tới
        assert_eq!(tt("toiwf"),"tời");  // toi + w + huyền → tời
        assert_eq!(tt("toiwr"),"tởi");  // toi + w + hỏi → tởi
        assert_eq!(tt("toiwj"),"tợi");  // toi + w + nặng → tợi
        // nguoi + w + f = người (w→ơ on o, uow→ươ, f→huyền on ư)
        assert_eq!(tt("nguoiwf"),"người");
        assert_eq!(tt("nguoiws"),"ngưới");
        // muoi + w + f = mười
        assert_eq!(tt("muoiwf"),"mười");
        // tuoi + w + s = tưới
        assert_eq!(tt("tuoiws"),"tưới");
        // "tuổi" correctly typed as tuooir (oo→ô)
        assert_eq!(tt("tuooir"),"tuổi");
        assert_eq!(tt("tuooif"),"tuồi");
    }
    #[test] fn telex_toan() {
        // "toàn" has trailing consonant → tone on 2nd vowel always
        assert_eq!(tt("toanf"),"toàn");
        assert_eq!(t("toanf"),"toàn");
        assert_eq!(tt("hoanf"),"hoàn");
        assert_eq!(t("hoanf"),"hoàn");
        // open diphthong oa/oe/uy: always 2nd vowel
        // òa typed as ofa (tone between vowels), oà typed as oaf (tone after)
        assert_eq!(t("oaf"),"oà");   // tone after all vowels → 2nd
        assert_eq!(t("ofa"),"òa");   // tone between vowels → 1st
        assert_eq!(tt("oaf"),"oà");
        assert_eq!(tt("ofa"),"òa");  // same
    }

    // ── TeipVni (combined Telex + VNI) tests ─────────────────────────
    fn tv(s: &str) -> String { convert_teip_vni(s, false, true) }

    #[test] fn teipvni_telex_words() {
        assert_eq!(tv("vieejt"),"việt"); assert_eq!(tv("dd"),"đ");
        assert_eq!(tv("aa"),"â"); assert_eq!(tv("aw"),"ă");
        assert_eq!(tv("ee"),"ê"); assert_eq!(tv("oo"),"ô");
        assert_eq!(tv("ow"),"ơ"); assert_eq!(tv("uw"),"ư");
        assert_eq!(tv("uow"),"ươ"); assert_eq!(tv("w"),"ư");
    }
    #[test] fn teipvni_vni_marks() {
        assert_eq!(tv("a6"),"â"); assert_eq!(tv("a8"),"ă");
        assert_eq!(tv("e6"),"ê"); assert_eq!(tv("o6"),"ô");
        assert_eq!(tv("o7"),"ơ"); assert_eq!(tv("u7"),"ư");
        assert_eq!(tv("uo7"),"ươ"); assert_eq!(tv("d9"),"đ");
    }
    #[test] fn teipvni_numbers() {
        // VNI tones
        assert_eq!(tv("qua1"),"quá"); assert_eq!(tv("gia1"),"giá");
        assert_eq!(tv("vie65t"),"việt"); assert_eq!(tv("tie61ng"),"tiếng");
    }
    #[test] fn teipvni_mixed() {
        // Mix Telex and VNI in same stream
        assert_eq!(tv("vie6t"),"viêt"); // VNI circumflex + Telex
        assert_eq!(tv("vie6ts"),"viết"); // VNI circumflex + Telex tone
        assert_eq!(tv("tie6ngs"),"tiếng");
    }

    // ── VIQR tests ──────────────────────────────────────────────────
    fn viqr(s: &str) -> String { convert_viqr(s) }

    #[test] fn viqr_marks() {
        assert_eq!(viqr("a^"),"â"); assert_eq!(viqr("a("),"ă");
        assert_eq!(viqr("e^"),"ê"); assert_eq!(viqr("o^"),"ô");
        assert_eq!(viqr("o+"),"ơ"); assert_eq!(viqr("u+"),"ư");
        assert_eq!(viqr("dd"),"đ");
    }
    #[test] fn viqr_tones() {
        assert_eq!(viqr("a'"),"á"); assert_eq!(viqr("a`"),"à");
        assert_eq!(viqr("a?"),"ả"); assert_eq!(viqr("a~"),"ã");
        assert_eq!(viqr("a."),"ạ");
    }
    #[test] fn viqr_words() {
        assert_eq!(viqr("vie^'t"),"viết");
        assert_eq!(viqr("nu+o+'c"),"nước");
        assert_eq!(viqr("ngu+o+`i"),"người");
    }
    #[test] fn viqr_dd() {
        assert_eq!(viqr("dd"),"đ"); assert_eq!(viqr("DD"),"Đ");
    }
    #[test] fn viqr_english() { assert_eq!(viqr("hello"),"hello"); }
}

#[test]
fn full_parity() {
    // This test verifies full parity for all 73 reference cases.
    // traditional settings: modern=false (traditional tone placement), short_w=true

    struct Case { input: &'static str, expected: &'static str, method: u8 }
    // method: 1=Telex, 2=VNI, 3=VIQR, 4=TeipVni
    let cases = [
        // ── Telex ──
        Case{input:"dd",expected:"đ",method:1},Case{input:"DD",expected:"Đ",method:1},
        Case{input:"Dd",expected:"Đ",method:1},Case{input:"dD",expected:"Đ",method:1},
        Case{input:"aa",expected:"â",method:1},Case{input:"aw",expected:"ă",method:1},
        Case{input:"ee",expected:"ê",method:1},Case{input:"ew",expected:"ê",method:1},
        Case{input:"oo",expected:"ô",method:1},Case{input:"ow",expected:"ơ",method:1},
        Case{input:"uw",expected:"ư",method:1},Case{input:"uow",expected:"ươ",method:1},
        Case{input:"w",expected:"ư",method:1},Case{input:"W",expected:"Ư",method:1},
        Case{input:"wn",expected:"ưn",method:1},Case{input:"ws",expected:"ứ",method:1},
        Case{input:"wf",expected:"ừ",method:1},
        Case{input:"aws",expected:"ắ",method:1},Case{input:"aas",expected:"ấ",method:1},
        Case{input:"ees",expected:"ế",method:1},Case{input:"oos",expected:"ố",method:1},
        Case{input:"ows",expected:"ớ",method:1},Case{input:"uws",expected:"ứ",method:1},
        Case{input:"chaof",expected:"chào",method:1},Case{input:"vieejt",expected:"việt",method:1},
        Case{input:"tieengs",expected:"tiếng",method:1},Case{input:"quas",expected:"quá",method:1},
        Case{input:"gias",expected:"giá",method:1},Case{input:"xin",expected:"xin",method:1},
        Case{input:"tooi",expected:"tôi",method:1},Case{input:"laf",expected:"là",method:1},
        Case{input:"cos",expected:"có",method:1},Case{input:"khoong",expected:"không",method:1},
        Case{input:"Vieetj",expected:"Việt",method:1},
        Case{input:"toans",expected:"toán",method:1},Case{input:"hoaf",expected:"hoà",method:1},
        Case{input:"thowif",expected:"thời",method:1},Case{input:"cuar",expected:"của",method:1},
        Case{input:"gif",expected:"gì",method:1},Case{input:"giowf",expected:"giờ",method:1},
        Case{input:"gioir",expected:"giỏi",method:1},
        Case{input:"cuowcs",expected:"cước",method:1},Case{input:"dduwowcj",expected:"được",method:1},
        Case{input:"nguwowif",expected:"người",method:1},
        Case{input:"asz",expected:"a",method:1},Case{input:"hello",expected:"hello",method:1},
        Case{input:"hoanf",expected:"hoàn",method:1},Case{input:"huyeenf",expected:"huyền",method:1},
        Case{input:"nguyeexn",expected:"nguyễn",method:1},Case{input:"ngoaif",expected:"ngoài",method:1},
        Case{input:"thuowngf",expected:"thường",method:1},
        // ── VNI ──
        Case{input:"d9",expected:"đ",method:2},Case{input:"a6",expected:"â",method:2},
        Case{input:"a8",expected:"ă",method:2},Case{input:"e6",expected:"ê",method:2},
        Case{input:"o6",expected:"ô",method:2},Case{input:"o7",expected:"ơ",method:2},
        Case{input:"u7",expected:"ư",method:2},Case{input:"uo7",expected:"ươ",method:2},
        Case{input:"vie65t",expected:"việt",method:2},Case{input:"tie61ng",expected:"tiếng",method:2},
        Case{input:"qua1",expected:"quá",method:2},Case{input:"gia1",expected:"giá",method:2},
        // ── TeipVni ──
        Case{input:"vie61t",expected:"viết",method:4},Case{input:"vieejt",expected:"việt",method:4},
        Case{input:"tie61ng",expected:"tiếng",method:4},
        Case{input:"a6",expected:"â",method:4},Case{input:"dd",expected:"đ",method:4},
        // ── VIQR ──
        Case{input:"dd",expected:"đ",method:3},Case{input:"DD",expected:"Đ",method:3},
        Case{input:"vie^'t",expected:"viết",method:3},Case{input:"nu+o+'c",expected:"nước",method:3},
        Case{input:"ngu+o+`i",expected:"người",method:3},
    ];

    let mut failures = Vec::new();
    for c in &cases {
        let result = match c.method {
            1 => convert_telex(c.input, false, true),
            2 => convert_vni(c.input, false, true),
            3 => convert_viqr(c.input),
            4 => convert_teip_vni(c.input, false, true),
            _ => unreachable!(),
        };
        if result != c.expected {
            failures.push(format!("[m{}] '{}' -> '{}' (expected '{}')", c.method, c.input, result, c.expected));
        }
    }
    assert!(failures.is_empty(), "{} full parity failures:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn comprehensive_vietnamese() {
    // Comprehensive test covering all Vietnamese syllable patterns.
    // Uses traditional settings: modern=false, short_w=true.
    // method: 1=Telex, 2=VNI, 3=VIQR, 4=TeipVni.
    // Method 0 = Telex (traditional) for brevity.

    let cases: &[(&str, &str, u8)] = &[
        // ═══ SINGLE VOWELS — 5 tones each ═══
        ("as","á",1),("af","à",1),("ar","ả",1),("ax","ã",1),("aj","ạ",1),
        ("es","é",1),("ef","è",1),("er","ẻ",1),("ex","ẽ",1),("ej","ẹ",1),
        ("is","í",1),("if","ì",1),("ir","ỉ",1),("ix","ĩ",1),("ij","ị",1),
        ("os","ó",1),("of","ò",1),("or","ỏ",1),("ox","õ",1),("oj","ọ",1),
        ("us","ú",1),("uf","ù",1),("ur","ủ",1),("ux","ũ",1),("uj","ụ",1),
        ("ys","ý",1),("yf","ỳ",1),("yr","ỷ",1),("yx","ỹ",1),("yj","ỵ",1),

        // ═══ MARKED VOWELS (breve/circumflex/horn) + tones ═══
        // ă = aw
        ("aw","ă",1),("aws","ắ",1),("awf","ằ",1),("awr","ẳ",1),("awx","ẵ",1),("awj","ặ",1),
        // â = aa
        ("aa","â",1),("aas","ấ",1),("aaf","ầ",1),("aar","ẩ",1),("aax","ẫ",1),("aaj","ậ",1),
        // ê = ee / ew
        ("ee","ê",1),("ees","ế",1),("eef","ề",1),("eer","ể",1),("eex","ễ",1),("eej","ệ",1),
        ("ew","ê",1),("ews","ế",1),("ewf","ề",1),("ewr","ể",1),
        // ô = oo
        ("oo","ô",1),("oos","ố",1),("oof","ồ",1),("oor","ổ",1),("oox","ỗ",1),("ooj","ộ",1),
        // ơ = ow
        ("ow","ơ",1),("ows","ớ",1),("owf","ờ",1),("owr","ở",1),("owx","ỡ",1),("owj","ợ",1),
        // ư = uw
        ("uw","ư",1),("uws","ứ",1),("uwf","ừ",1),("uwr","ử",1),("uwx","ữ",1),("uwj","ự",1),

        // ═══ dd → đ ═══
        ("dd","đ",1),

        // ═══ STANDALONE w → ư ═══
        ("w","ư",1),("ws","ứ",1),("wf","ừ",1),("wr","ử",1),("wx","ữ",1),("wj","ự",1),
        ("wn","ưn",1),("wt","ưt",1),

        // ═══ DIPHTHONGS — closed (trailing consonant) ═══
        // ai, ao, au
        ("ais","ái",1),("aif","ài",1),("air","ải",1),("aix","ãi",1),("aij","ại",1),
        ("aos","áo",1),("aof","ào",1),("aor","ảo",1),("aox","ão",1),("aoj","ạo",1),
        ("aus","áu",1),("auf","àu",1),("aur","ảu",1),("aux","ãu",1),("auj","ạu",1),
        // ây, ay
        ("aays","ấy",1),("aayf","ầy",1),("aayr","ẩy",1),("aayx","ẫy",1),("aayj","ậy",1),
        ("ays","áy",1),("ayf","ày",1),("ayr","ảy",1),("ayx","ãy",1),("ayj","ạy",1),
        // eo, êu
        ("eos","éo",1),("eof","èo",1),("eor","ẻo",1),("eox","ẽo",1),("eoj","ẹo",1),
        ("eeus","ếu",1),("eeuf","ều",1),("eeur","ểu",1),("eeux","ễu",1),("eeuj","ệu",1),
        // oi, ôi, ơi, ui, ưi, ưu
        ("ois","ói",1),("oif","òi",1),("oir","ỏi",1),("oix","õi",1),("oij","ọi",1),
        ("oois","ối",1),("ooif","ồi",1),("ooir","ổi",1),("ooix","ỗi",1),("ooij","ội",1),
        ("owis","ới",1),("owif","ời",1),("owir","ởi",1),("owix","ỡi",1),("owij","ợi",1),
        ("uis","úi",1),("uif","ùi",1),("uir","ủi",1),("uix","ũi",1),("uij","ụi",1),
        ("uwis","ứi",1),("uwif","ừi",1),("uwir","ửi",1),("uwix","ữi",1),("uwij","ựi",1),
        ("uwus","ứu",1),("uwuf","ừu",1),("uwur","ửu",1),("uwux","ữu",1),("uwuj","ựu",1),

        // ═══ oa, oe, uy — closed (trailing consonant, tone on 2nd vowel) ═══
        ("oans","oán",1),("oanf","oàn",1),("oanr","oản",1),("oanx","oãn",1),("oanj","oạn",1),
        ("oats","oát",1),("oatf","oàt",1),("oatr","oảt",1),
        ("oens","oén",1),("oenf","oèn",1),
        ("uyns","uýn",1),("uynf","uỳn",1),("uynr","uỷn",1),

        // ═══ oa, oe, uy — open (traditional: tone on 2nd vowel) ═══
        ("oas","oá",1),("oaf","oà",1),("oar","oả",1),("oax","oã",1),("oaj","oạ",1),
        ("oes","oé",1),("oef","oè",1),
        ("uys","uý",1),("uyf","uỳ",1),("uyr","uỷ",1),("uyx","uỹ",1),("uyj","uỵ",1),

        // ═══ TRIPHTHONGS ═══
        // oai, oay (tone on middle vowel)
        ("oais","oái",1),("oaif","oài",1),("oair","oải",1),("oaix","oãi",1),("oaij","oại",1),
        ("oays","oáy",1),("oayf","oày",1),("oayr","oảy",1),("oayx","oãy",1),("oayj","oạy",1),
        // uây (tone on middle vowel â)
        ("uaays","uấy",1),("uaayf","uầy",1),("uaayr","uẩy",1),("uaayx","uẫy",1),("uaayj","uậy",1),
        // uyê (tone on ê — 3rd vowel, the one with variant)
        ("uyeef","uyề",1),("uyees","uyế",1),("uyeer","uyể",1),("uyeex","uyễ",1),("uyeej","uyệ",1),
        // iê, yê
        ("ieef","iề",1),("iees","iế",1),("ieer","iể",1),("ieex","iễ",1),("ieej","iệ",1),
        ("yeef","yề",1),("yees","yế",1),("yeer","yể",1),("yeex","yễ",1),("yeej","yệ",1),
        // uôi = u + ô + i (oo→ô, tone on ô which has variant≠0)
        ("uoois","uối",1),("uooif","uồi",1),("uooir","uổi",1),("uooix","uỗi",1),("uooij","uội",1),
        // ươi = u + ơ + i (w targets o, uow spread → ươ, tone on ơ which has variant≠0)
        ("uoiws","ưới",1),("uoiwf","ười",1),("uoiwr","ưởi",1),("uoiwx","ưỡi",1),("uoiwj","ượi",1),

        // ═══ GI / QU initial consonant digraphs ═══
        ("gif","gì",1),("gis","gí",1),("gir","gỉ",1),("gix","gĩ",1),("gij","gị",1),
        ("gias","giá",1),("giaf","già",1),("giar","giả",1),("giax","giã",1),("giaj","giạ",1),
        ("gioir","giỏi",1),("gioif","giòi",1),("giois","giói",1),
        ("quas","quá",1),("quaf","quà",1),("quar","quả",1),("quax","quã",1),("quaj","quạ",1),
        ("quys","quý",1),("quyf","quỳ",1),("quyr","quỷ",1),("quyx","quỹ",1),("quyj","quỵ",1),

        // ═══ REAL WORDS ═══
        ("chaof","chào",1),("vieejt","việt",1),("tieengs","tiếng",1),
        ("khoong","không",1),("cuar","của",1),("cuowcs","cước",1),
        ("dduwowcj","được",1),("nguwowif","người",1),("thuowngf","thường",1),
        ("hoanf","hoàn",1),("huyeenf","huyền",1),("nguyeexn","nguyễn",1),
        ("ngoaif","ngoài",1),("ngoafi","ngoài",1),
        ("vaya","vây",1),("vayaj","vậy",1),("vayas","vấy",1),("mayas","mấy",1),
        ("toans","toán",1),("thowif","thời",1),
        ("xin","xin",1),("tooi","tôi",1),("laf","là",1),("cos","có",1),
        ("banj","bạn",1),("minhf","mình",1),("nawm","năm",1),("thangs","tháng",1),
        ("truwowngf","trường",1),("luaatj","luật",1),

        // ═══ CONSONANT CLUSTERS ═══
        ("nhas","nhá",1),("nghir","nghỉ",1),("gheps","ghép",1),
        ("truws","trứ",1),("thas","thá",1),("phos","phó",1),
        ("khas","khá",1),("chays","cháy",1),("ngays","ngáy",1),

        // ═══ FINAL CONSONANT VARIETY ═══
        ("ddejp","đẹp",1),("maats","mất",1),("bacs","bác",1),
        ("ngachs","ngách",1),("banhf","bành",1),
        ("tins","tín",1),("metj","mẹt",1),

        // ═══ z (tone removal) ═══
        ("asz","a",1),("awsjz","ă",1),("aasz","â",1),

        // ═══ ENGLISH PASS-THROUGH (words without Telex tone/mark keys s/f/r/x/j/w/z) ═══
        ("hello","hello",1),("bacon","bacon",1),("thing","thing",1),("climb","climb",1),

        // ═══ CAPITALIZATION ═══
        ("As","Á",1),("Af","À",1),("Aas","Ấ",1),("Aws","Ắ",1),
        ("Dd","Đ",1),("DD","Đ",1),
        ("W","Ư",1),("Ws","Ứ",1),
        ("Vieetj","Việt",1),("Tooi","Tôi",1),

        // ═══ VNI method ═══
        ("d9","đ",2),("a6","â",2),("a8","ă",2),("e6","ê",2),
        ("o6","ô",2),("o7","ơ",2),("u7","ư",2),("uo7","ươ",2),
        ("vie65t","việt",2),("tie61ng","tiếng",2),
        ("qua1","quá",2),("gia1","giá",2),

        // ═══ TeipVni combined ═══
        ("vie61t","viết",4),("vieejt","việt",4),
        ("tie61ng","tiếng",4),("a6","â",4),("dd","đ",4),

        // ═══ VIQR method ═══
        ("dd","đ",3),("DD","Đ",3),
        ("vie^'t","viết",3),("nu+o+'c","nước",3),("ngu+o+`i","người",3),
    ];

    let mut failures = Vec::new();
    for &(input, expected, method) in cases {
        let result = match method {
            1 => convert_telex(input, false, true),
            2 => convert_vni(input, false, true),
            3 => convert_viqr(input),
            4 => convert_teip_vni(input, false, true),
            _ => unreachable!(),
        };
        if result != expected {
            failures.push(format!("[m{}] '{}' -> '{}' (expected '{}')", method, input, result, expected));
        }
    }
    assert!(failures.is_empty(),
        "{} comprehensive failures (traditional mode):\n{}",
        failures.len(), failures.join("\n"));
}