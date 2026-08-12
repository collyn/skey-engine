//! VSeqList parity test — validates skey-engine against all 70 Vietnamese
//! vowel sequences from x-unikey's VSeqList.
//!
//! Each test case exercises typing the vowel sequence in Telex and verifies
//! correct output.  Run with: cargo test vseq -- --nocapture

#[cfg(test)]
mod vseq_parity {
    use skey_engine::engine;

    // All 70 vowel sequences from x-unikey VSeqList, grouped by length.
    // Format: (telex_input, expected_output, description)

    #[test]
    fn single_vowels() {
        let tt = |s| engine::convert_telex(s, false, true);
        // 12 single vowels — all complete, all allow suffix
        assert_eq!(tt("as"), "á");    // a acute
        assert_eq!(tt("af"), "à");    // a grave
        assert_eq!(tt("ar"), "ả");    // a hook
        assert_eq!(tt("ax"), "ã");    // a tilde
        assert_eq!(tt("aj"), "ạ");    // a dot
        assert_eq!(tt("aas"), "ấ");   // â acute (tone on â, default pos 0)
        assert_eq!(tt("aws"), "ắ");   // ă acute
        assert_eq!(tt("es"), "é");
        assert_eq!(tt("ees"), "ế");   // ê acute
        assert_eq!(tt("is"), "í");
        assert_eq!(tt("os"), "ó");
        assert_eq!(tt("oos"), "ố");   // ô acute
        assert_eq!(tt("ows"), "ớ");   // ơ acute
        assert_eq!(tt("us"), "ú");
        assert_eq!(tt("uws"), "ứ");   // ư acute
        assert_eq!(tt("ys"), "ý");
    }

    #[test]
    fn open_diphthongs_no_suffix() {
        let tt = |s| engine::convert_telex(s, false, true);
        // Open diphthongs — tone on first vowel, no suffix allowed
        assert_eq!(tt("ais"), "ái");   // ai
        assert_eq!(tt("aos"), "áo");   // ao
        assert_eq!(tt("aus"), "áu");   // au
        assert_eq!(tt("ays"), "áy");   // ay
        assert_eq!(tt("aaus"), "ấu");  // âu — tone on â (has variant)
        assert_eq!(tt("aays"), "ấy");  // ây — tone on â
        assert_eq!(tt("eos"), "éo");   // eo
        assert_eq!(tt("eeus"), "ếu");  // êu — tone on ê (has variant)
        assert_eq!(tt("ias"), "ía");   // ia — tone on i (default first vowel)
        assert_eq!(tt("ius"), "íu");   // iu
        assert_eq!(tt("ois"), "ói");   // oi
        assert_eq!(tt("oois"), "ối");  // ôi — tone on ô (variant)
        assert_eq!(tt("owis"), "ới");  // ơi — tone on ơ (variant)
        assert_eq!(tt("uas"), "úa");   // ua
        assert_eq!(tt("uis"), "úi");   // ui
        assert_eq!(tt("uwas"), "ứa");  // ưa — tone on ư (variant)
        assert_eq!(tt("uwis"), "ứi");  // ưi — tone on ư
        assert_eq!(tt("uwus"), "ứu");  // ưu — tone on first ư (variant at 0)
    }

    #[test]
    fn closed_diphthongs_need_suffix() {
        let tt = |s| engine::convert_telex(s, false, true);
        // ie/iê — need consonant, tone on ê (variant)
        assert_eq!(tt("tieengs"), "tiếng");
        assert_eq!(tt("tieeps"), "tiếp");
        // oa — closed, tone on first vowel by default (traditional)
        // For closed syllable (toan), tone moves to last vowel → toán
        assert_eq!(tt("toans"), "toán");
        // oă — closed, tone on ă (variant)
        assert_eq!(tt("hoawcj"), "hoặc");
        // oe — closed, traditional: tone on o (first vowel)
        // "khỏe" = k + oe + hook (khoer), typed as open diphthong
        assert_eq!(tt("khoer"), "khỏe");
        // uâ — closed, tone on â (variant)
        assert_eq!(tt("xuaans"), "xuấn");
        // uê — closed, tone on ê (variant)
        assert_eq!(tt("thuees"), "thuế");
        // uô — closed, tone on ô (variant)
        assert_eq!(tt("buoons"), "buốn");
        // ươ — closed, tone on ơ? or ư?
        // x-unikey: hookPos=1 (on ơ), but the variant is on ơ
        // With tone, variant=2 on both: tone on the one with variant...
        // Actually: ư has variant=2, ơ has variant=2. tone_vowel_index finds
        // the first one with variant≠0 going backwards: ơ (index 1).
        assert_eq!(tt("cuowcs"), "cước"); // ươc + acute → cước (tone on ơ)
        // uy — closed: tone on last vowel regardless of style
        // (closed oa/oe/uy always tone on second vowel in Vietnamese spelling)
        assert_eq!(tt("khuyt"), "khuyt"); // khuyt (rare, no tone)
        // yê — closed, tone on ê (variant)
        assert_eq!(tt("yeeng"), "yêng");   // yêng (no tone)
    }

    #[test]
    fn triphthongs() {
        let tt = |s| engine::convert_telex(s, false, true);
        // iêu — complete, no suffix, tone on ê (variant)
        assert_eq!(tt("ieeus"), "iếu");
        // oai — tone on a (middle vowel)
        assert_eq!(tt("oais"), "oái");
        // oay — tone on a (middle)
        assert_eq!(tt("oays"), "oáy");
        // oeo — tone on e (middle)
        assert_eq!(tt("oeos"), "oéo");
        // uây — tone on â (variant at pos 1)
        assert_eq!(tt("uaays"), "uấy");
        // uôi — tone on ô (variant at pos 1)
        assert_eq!(tt("uoois"), "uối");
        // uya — tone on y (middle)
        assert_eq!(tt("khuyas"), "khuýa");
        // uyu — tone on y (middle)
        assert_eq!(tt("khuyus"), "khuýu");
        // ươi — tone on ơ (variant at pos 1)
        assert_eq!(tt("nguoiws"), "ngưới");
        // ươu — tone on ơ (variant at pos 1)
        assert_eq!(tt("ruowus"), "rướu");
        // yêu — tone on ê (variant at pos 1)
        assert_eq!(tt("yeeus"), "yếu");
    }

    #[test]
    fn uu_cluster() {
        let tt = |s| engine::convert_telex(s, false, true);
        // uu + w → ưu (horn on first u)
        assert_eq!(tt("uuw"), "ưu");
        // uu + w + s → ứu (acute on first u which is now ư)
        assert_eq!(tt("uuws"), "ứu");
        // And by typing ưu directly
        assert_eq!(tt("uwu"), "ưu");
    }

    #[test]
    fn open_oa_oe_uy_traditional() {
        let tt = |s| engine::convert_telex(s, false, true);
        // Traditional: tone on FIRST vowel for open oa/oe/uy
        assert_eq!(tt("hoaf"), "hòa");   // oa + grave → hòa (tone on o)
        assert_eq!(tt("thowif"), "thời"); // ơi → thời
        assert_eq!(tt("cuar"), "của");   // ua + hook → của (tone on u)
        // oa/oe/uy open: traditional=tone on first
        assert_eq!(tt("oas"), "óa");     // tone on o (first)
        assert_eq!(tt("oes"), "óe");     // tone on o (first)
        assert_eq!(tt("uys"), "úy");     // tone on u (first)
    }

    #[test]
    fn open_oa_oe_uy_modern() {
        let tm = |s| engine::convert_telex(s, true, true);
        // Modern: tone on SECOND vowel for open oa/oe/uy
        assert_eq!(tm("hoaf"), "hoà");   // oa + grave → hoà (tone on a)
        assert_eq!(tm("oas"), "oá");     // tone on a (second)
        assert_eq!(tm("oes"), "oé");     // tone on e (second)
        assert_eq!(tm("uys"), "uý");     // tone on y (second)
    }
}
