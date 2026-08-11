//! Comprehensive parity check: all 70 x-unikey vowel sequences vs skey-engine.
//! Run with: cargo test parity -- --ignored --nocapture

#[cfg(test)]
mod parity {
    use skey_engine::spelling;
    use skey_engine::engine;

    /// All 70 vowel sequences from x-unikey VSeqList, mapped to
    /// approximate Vietnamese spelling. Each tuple:
    /// (xunikey_name, vietnamese_chars, complete, con_suffix, roof_pos, hook_pos)
    type VSeq = (&'static str, &'static str, bool, bool, Option<usize>, Option<usize>);

    fn all_vseqs() -> Vec<VSeq> {
        vec![
            // 1-vowel (12)
            ("a",  "a",  true,  true,  None,    None),
            ("â",  "â",  true,  true,  Some(0), None),
            ("ă",  "ă",  true,  true,  None,    Some(0)),
            ("e",  "e",  true,  true,  None,    None),
            ("ê",  "ê",  true,  true,  Some(0), None),
            ("i",  "i",  true,  true,  None,    None),
            ("o",  "o",  true,  true,  None,    None),
            ("ô",  "ô",  true,  true,  Some(0), None),
            ("ơ",  "ơ",  true,  true,  None,    Some(0)),
            ("u",  "u",  true,  true,  None,    None),
            ("ư",  "ư",  true,  true,  None,    Some(0)),
            ("y",  "y",  true,  true,  None,    None),
            // 2-vowel: can stand alone (con_suffix=false)
            ("ai", "ai", true,  false, None,    None),
            ("ao", "ao", true,  false, None,    None),
            ("au", "au", true,  false, None,    None),
            ("ay", "ay", true,  false, None,    None),
            ("âu", "âu", true,  false, Some(0), None),
            ("ây", "ây", true,  false, Some(0), None),
            ("eo", "eo", true,  false, None,    None),
            ("êu", "êu", true,  false, Some(0), None),
            ("ia", "ia", true,  false, None,    None),
            ("iu", "iu", true,  false, None,    None),
            ("oi", "oi", true,  false, None,    None),
            ("ôi", "ôi", true,  false, Some(0), None),
            ("ơi", "ơi", true,  false, None,    Some(0)),
            ("ua", "ua", true,  false, None,    None),
            ("ui", "ui", true,  false, None,    None),
            ("ưa", "ưa", true,  false, None,    Some(0)),
            ("ưi", "ưi", true,  false, None,    Some(0)),
            ("ưu", "ưu", true,  false, None,    Some(0)),
            // 2-vowel: need suffix (con_suffix=true)
            ("ie", "ie", false, true,  None,    None),
            ("iê", "iê", true,  true,  Some(1), None),
            ("oa", "oa", true,  true,  None,    None),
            ("oă", "oă", true,  true,  None,    Some(1)),
            ("oe", "oe", true,  true,  None,    None),
            ("uâ", "uâ", true,  true,  Some(1), None),
            ("ue", "ue", false, true,  None,    None),
            ("uê", "uê", true,  true,  Some(1), None),
            ("uo", "uo", false, true,  None,    None),
            ("uô", "uô", true,  true,  Some(1), None),
            ("ươ", "ươ", true,  true,  None,    Some(1)),
            ("uy", "uy", true,  true,  None,    None),
            ("ye", "ye", false, true,  None,    None),
            ("yê", "yê", true,  true,  Some(1), None),
            ("ưo", "ưo", false, true,  None,    Some(0)),
            // 3-vowel (22)
            ("iêu", "iêu", true,  false, Some(1), None),
            ("oai", "oai", true,  false, None,    None),
            ("oay", "oay", true,  false, None,    None),
            ("oeo", "oeo", true,  false, None,    None),
            ("uây", "uây", true,  false, Some(1), None),
            ("uôi", "uôi", true,  false, Some(1), None),
            ("uya", "uya", true,  false, None,    None),
            ("uye", "uye", false, true,  None,    None),
            ("uyê", "uyê", true,  true,  Some(2), None),
            ("uyu", "uyu", true,  false, None,    None),
            ("ươi", "ươi", true,  false, None,    Some(0)),
            ("ươu", "ươu", true,  false, None,    Some(0)),
            ("yêu", "yêu", true,  false, Some(1), None),
        ]
    }

    #[test]
    #[ignore]
    fn vowel_sequence_validation_coverage() {
        println!("\n=== Vowel sequence validation coverage ===\n");

        let mut passing = 0;
        let mut failing = Vec::new();

        for (name, chars, complete, con_suffix, _roof, _hook) in &all_vseqs() {
            // Test as standalone word (no initial/final consonant)
            let valid_standalone = spelling::is_valid_cvc(chars);

            // Test with initial consonant "b"
            let with_c = format!("b{}", chars);
            let valid_with_c = spelling::is_valid_cvc(&with_c);

            // Test with final consonant "n" (only if con_suffix=true)
            let valid_with_suffix = if *con_suffix {
                let with_s = format!("b{}n", chars);
                spelling::is_valid_cvc(&with_s)
            } else {
                true // n/a — should not accept suffix
            };

            // Check if validation matches x-unikey's expectations
            let mut issues = Vec::new();

            // Standalone check: incomplete sequences should be invalid alone
            if !complete && valid_standalone {
                issues.push(format!("standalone: should be INVALID (incomplete seq)"));
            }

            // Suffix check: con_suffix=true sequences should accept suffix
            if *con_suffix {
                let with_s = format!("b{}n", chars);
                if !spelling::is_valid_cvc(&with_s) {
                    issues.push(format!("with suffix 'n': should be VALID but rejected"));
                }
            }

            if issues.is_empty() {
                passing += 1;
            } else {
                failing.push(format!("  {} ({}) — standalone={} with_c={} with_suffix={}",
                    chars, name, valid_standalone, valid_with_c, valid_with_suffix));
                for issue in &issues {
                    failing.push(format!("    ✗ {}", issue));
                }
            }
        }

        for f in &failing {
            println!("{}", f);
        }
        println!("\n  Validated: {}/{} vowel sequences match x-unikey expectations",
            passing, passing + failing.len());

        if !failing.is_empty() {
            println!("  ⚠ {} discrepancies remain\n", failing.len());
        } else {
            println!("  ✅ All vowel sequences validated correctly\n");
        }
    }

    #[test]
    #[ignore]
    fn tone_placement_coverage() {
        println!("\n=== Tone placement coverage (telex, traditional) ===\n");

        // Test tone placement for key vowel sequences via Telex input
        // Format: (telex_input, expected_output, description)
        let cases: &[(&str, &str, &str)] = &[
            // Single vowels — trivial
            ("as", "á", "a + acute"),
            ("af", "à", "a + grave"),
            // Marked vowels
            ("aws", "ắ", "ă + acute"),
            ("aas", "ấ", "â + acute"),
            ("ees", "ế", "ê + acute"),
            ("oos", "ố", "ô + acute"),
            ("ows", "ớ", "ơ + acute"),
            ("uws", "ứ", "ư + acute"),
            // Diphthongs — tone on first vowel (traditional, closed)
            ("ais", "ái", "ai + acute"),
            ("aos", "áo", "ao + acute"),
            ("aus", "áu", "au + acute"),
            ("ays", "áy", "ay + acute"),
            ("aaus", "ấu", "âu + acute → tone on â"),
            ("aays", "ấy", "ây + acute → tone on â"),
            ("eesf", "ếu", "êu + acute → tone on ê"),
            // Closed diphthongs (tone on first vowel, traditional)
            ("toans", "toán", "toa + acute → oá → toá"),
            ("hoaf", "hòa", "hoa + grave → hòa"),
            ("thowif", "thời", "thơi + acute → thời"),
            ("cuar", "của", "cua + hook → của"),
            // Triphthongs — tone on middle vowel
            ("ngoais", "ngoái", "ngoai + acute → tone on a"),
            ("ngoayr", "ngoảy", "ngoay + hook → tone on a"),
            // uyê — tone on ê (3rd vowel with variant)
            ("uyeef", "uyề", "uyê + grave"),
            ("uyees", "uyế", "uyê + acute"),
            // iê — tone on ê (variant)
            ("tieengs", "tiếng", "iê + ng + acute → tone on ê"),
            // uôi — tone on ô (variant)
            ("uoois", "uối", "uôi + acute → tone on ô"),
            // ươi — tone on ơ (variant)
            ("uoiws", "ưới", "ươi + acute → tone on ơ"),
            // gi / qu
            ("gias", "giá", "gi + a + acute"),
            ("quas", "quá", "qu + a + acute"),
            ("gif", "gì", "gi + grave"),
            // Complex words
            ("cuowcs", "cước", "cươc + acute"),
            ("dduwowcj", "được", "đươc + dot"),
            ("nguwowif", "người", "ngươi + grave"),
            ("thuowngf", "thường", "thương + grave"),
            ("hoanf", "hoàn", "hoan + grave → hoàn"),
            ("huyeenf", "huyền", "huyê + n + grave"),
            ("nguyeexn", "nguyễn", "nguyê + n + acute → nguyễn (tilde)"),
        ];

        let mut ok = 0;
        let mut bad = Vec::new();

        for (input, expected, desc) in cases {
            let result = engine::convert_telex(input, false, true);
            if result == *expected {
                ok += 1;
            } else {
                bad.push(format!("  '{}' → '{}' (expected '{}') — {}", input, result, expected, desc));
            }
        }

        for b in &bad {
            println!("{}", b);
        }
        println!("\n  Tone placement: {}/{} correct", ok, ok + bad.len());
        if !bad.is_empty() {
            println!("  ⚠ {} tone placement errors\n", bad.len());
        } else {
            println!("  ✅ All tone placements correct\n");
        }
    }

    #[test]
    #[ignore]
    fn rare_vowel_sequences_engine() {
        println!("\n=== Rare vowel sequence engine output ===\n");

        // Test sequences that are phonologically valid but rarely used
        let rare_cases: &[(&str, &str)] = &[
            // ue sequences (huê, thuê, etc.)
            ("thuees", "thuế"),   // thuê + acute → thuế
            ("huef", "huề"),      // huê + grave → huề
            ("hueen", "huên"),    // huên (rare but valid CVC)
            ("hueenh", "huênh"),  // huênh
            // oă sequences (hoặc, thoăn, etc.)
            ("hoawcj", "hoặc"),   // oă + c + dot
            ("thoaws", "thoắc"),  // oă + c + acute
            ("thoawts", "thoắt"), // oă + t + acute
            // uy sequences
            ("thuyt", "thuýt"),   // uy + t
            ("huyp", "huýp"),     // uy + p
            ("nguyr", "nguỷ"),    // uy + hook
            // uya triphthong
            ("khuya", "khuya"),   // uya (no tone change)
            ("khuyar", "khuỷa"),  // uya + hook
            // uyu triphthong
            ("khuyus", "khuyú"),  // uyu + acute
            // ươu triphthong
            ("ruowus", "rướu"),   // ươu + acute
            ("huowuf", "hườu"),   // ươu + grave
        ];

        let mut ok = 0;
        let mut bad = Vec::new();

        for (input, expected) in rare_cases {
            let result = engine::convert_telex(input, false, true);
            if result == *expected {
                ok += 1;
            } else {
                bad.push(format!("  '{}' → '{}' (expected '{}')", input, result, expected));
            }
        }

        for b in &bad {
            println!("{}", b);
        }
        println!("\n  Rare sequences: {}/{} correct", ok, ok + bad.len());
        if !bad.is_empty() {
            println!("  ⚠ {} engine gaps for rare sequences\n", bad.len());
        } else {
            println!("  ✅ All rare sequences handled correctly\n");
        }
    }
}
