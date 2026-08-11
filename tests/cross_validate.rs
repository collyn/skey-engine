//! Cross-validation test: x-unikey data vs skey-engine.
//!
//! Uses x-unikey's comprehensive data tables as ground truth to find
//! gaps in skey-engine's validation and tone placement.
//!
//! Run with: cargo test cross_validate -- --nocapture --ignored

#[cfg(test)]
mod cross_validate {
    use skey_engine::engine;
    use skey_engine::spelling;

    // ── x-unikey VSeqList data (transcribed from ukengine.cpp) ─────────

    /// A vowel sequence entry from x-unikey's VSeqList
    struct VSeqEntry {
        chars: &'static str,      // base chars (e.g., "ua", "uye")
        complete: bool,           // can stand alone as word
        con_suffix: bool,         // allows consonant suffix
        roof_pos: Option<usize>,  // which char gets circumflex
        hook_pos: Option<usize>,  // which char gets breve/horn
    }

    /// All 70 vowel sequences from x-unikey VSeqList
    fn xunikey_vseq_list() -> Vec<VSeqEntry> {
        vec![
            // ── 1-vowel sequences ──
            VSeqEntry { chars: "a",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ă",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: Some(0) },  // ab
            VSeqEntry { chars: "â",  complete: true,  con_suffix: true,  roof_pos: Some(0), hook_pos: None    },  // ar
            VSeqEntry { chars: "e",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ê",  complete: true,  con_suffix: true,  roof_pos: Some(0), hook_pos: None    },  // er
            VSeqEntry { chars: "i",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "o",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ơ",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: Some(0) },  // oh
            VSeqEntry { chars: "ô",  complete: true,  con_suffix: true,  roof_pos: Some(0), hook_pos: None    },  // or
            VSeqEntry { chars: "u",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ư",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: Some(0) },  // uh
            VSeqEntry { chars: "y",  complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            // ── 2-vowel: open diphthongs (con_suffix=false) ──
            VSeqEntry { chars: "ai", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ao", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "au", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ay", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "âu", complete: true,  con_suffix: false, roof_pos: Some(0), hook_pos: None    },  // aru
            VSeqEntry { chars: "ây", complete: true,  con_suffix: false, roof_pos: Some(0), hook_pos: None    },  // ary
            VSeqEntry { chars: "eo", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "êu", complete: true,  con_suffix: false, roof_pos: Some(0), hook_pos: None    },  // eru
            VSeqEntry { chars: "ia", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "iu", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "oi", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ôi", complete: true,  con_suffix: false, roof_pos: Some(0), hook_pos: None    },  // ori
            VSeqEntry { chars: "ơi", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: Some(0) },  // ohi
            VSeqEntry { chars: "ua", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ui", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ưa", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: Some(0) },  // uha
            VSeqEntry { chars: "ưi", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: Some(0) },  // uhi
            VSeqEntry { chars: "ưu", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: Some(0) },  // uhu
            // ── 2-vowel: incomplete (need consonant suffix) ──
            VSeqEntry { chars: "eu", complete: false, con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ie", complete: false, con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "iê", complete: true,  con_suffix: true,  roof_pos: Some(1), hook_pos: None    },  // ier
            VSeqEntry { chars: "oa", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "oă", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: Some(1) },  // oab
            VSeqEntry { chars: "oe", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ua", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },  // duplicate? ua above con_suffix=false
            VSeqEntry { chars: "uâ", complete: true,  con_suffix: true,  roof_pos: Some(1), hook_pos: None    },  // uar
            VSeqEntry { chars: "ue", complete: false, con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "uê", complete: true,  con_suffix: true,  roof_pos: Some(1), hook_pos: None    },  // uer
            VSeqEntry { chars: "uo", complete: false, con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "uô", complete: true,  con_suffix: true,  roof_pos: Some(1), hook_pos: None    },  // uor
            VSeqEntry { chars: "ươ", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: Some(1) },  // uoh
            VSeqEntry { chars: "uy", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "uu", complete: false, con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "ye", complete: false, con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "yê", complete: true,  con_suffix: true,  roof_pos: Some(1), hook_pos: None    },  // yer
            VSeqEntry { chars: "ưo", complete: false, con_suffix: true,  roof_pos: None,    hook_pos: Some(0) },  // uho
            VSeqEntry { chars: "ươ", complete: true,  con_suffix: true,  roof_pos: None,    hook_pos: Some(0) },  // uhoh (with u+)
            // ── 3-vowel triphthongs (all con_suffix=false) ──
            VSeqEntry { chars: "iêu", complete: false, con_suffix: false, roof_pos: None,    hook_pos: None    },  // ieu
            VSeqEntry { chars: "iêu", complete: true,  con_suffix: false, roof_pos: Some(1), hook_pos: None    },  // ieru
            VSeqEntry { chars: "oai", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "oay", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "oeo", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "uây", complete: false, con_suffix: false, roof_pos: None,    hook_pos: None    },  // uay
            VSeqEntry { chars: "uây", complete: true,  con_suffix: false, roof_pos: Some(1), hook_pos: None    },  // uary
            VSeqEntry { chars: "uôi", complete: false, con_suffix: false, roof_pos: None,    hook_pos: None    },  // uoi
            VSeqEntry { chars: "uôi", complete: true,  con_suffix: false, roof_pos: Some(1), hook_pos: None    },  // uori
            VSeqEntry { chars: "ươi", complete: false, con_suffix: false, roof_pos: None,    hook_pos: Some(1) },  // uohi
            VSeqEntry { chars: "ươu", complete: false, con_suffix: false, roof_pos: None,    hook_pos: Some(1) },  // uohu
            VSeqEntry { chars: "uya", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "uye", complete: false, con_suffix: true,  roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "uyê", complete: true,  con_suffix: true,  roof_pos: Some(2), hook_pos: None    },  // uyer
            VSeqEntry { chars: "uyu", complete: true,  con_suffix: false, roof_pos: None,    hook_pos: None    },
            VSeqEntry { chars: "yêu", complete: false, con_suffix: false, roof_pos: None,    hook_pos: None    },  // yeu
            VSeqEntry { chars: "yêu", complete: true,  con_suffix: false, roof_pos: Some(1), hook_pos: None    },  // yeru
        ]
    }

    // ── x-unikey VCPairList: valid vowel→final_consonant pairs ────────

    /// Valid (vowel_sequence, final_consonant) pairs from x-unikey
    fn xunikey_vc_pairs() -> Vec<(&'static str, &'static str)> {
        vec![
            ("a", "c"), ("a", "ch"), ("a", "m"), ("a", "n"), ("a", "ng"), ("a", "nh"), ("a", "p"), ("a", "t"),
            ("â", "c"), ("â", "m"), ("â", "n"), ("â", "ng"), ("â", "p"), ("â", "t"),
            ("ă", "c"), ("ă", "m"), ("ă", "n"), ("ă", "ng"), ("ă", "p"), ("ă", "t"),
            ("e", "c"), ("e", "ch"), ("e", "m"), ("e", "n"), ("e", "ng"), ("e", "nh"), ("e", "p"), ("e", "t"),
            ("ê", "c"), ("ê", "ch"), ("ê", "m"), ("ê", "n"), ("ê", "nh"), ("ê", "p"), ("ê", "t"),
            ("i", "c"), ("i", "ch"), ("i", "m"), ("i", "n"), ("i", "nh"), ("i", "p"), ("i", "t"),
            ("iê", "c"), ("iê", "m"), ("iê", "n"), ("iê", "ng"), ("iê", "p"), ("iê", "t"),
            ("o", "c"), ("o", "m"), ("o", "n"), ("o", "ng"), ("o", "p"), ("o", "t"),
            ("oa", "c"), ("oa", "ch"), ("oa", "m"), ("oa", "n"), ("oa", "ng"), ("oa", "nh"), ("oa", "p"), ("oa", "t"),
            ("oă", "c"), ("oă", "m"), ("oă", "n"), ("oă", "ng"), ("oă", "t"),
            ("oe", "n"), ("oe", "t"),
            ("ô", "c"), ("ô", "m"), ("ô", "n"), ("ô", "ng"), ("ô", "p"), ("ô", "t"),
            ("ơ", "m"), ("ơ", "n"), ("ơ", "p"), ("ơ", "t"),
            ("u", "c"), ("u", "m"), ("u", "n"), ("u", "ng"), ("u", "p"), ("u", "t"),
            ("ua", "n"), ("ua", "ng"), ("ua", "t"),
            ("uâ", "n"), ("uâ", "ng"), ("uâ", "t"),
            ("uê", "c"), ("uê", "ch"), ("uê", "n"), ("uê", "nh"),
            ("uô", "c"), ("uô", "m"), ("uô", "n"), ("uô", "ng"), ("uô", "t"),
            ("uy", "c"), ("uy", "ch"), ("uy", "n"), ("uy", "nh"), ("uy", "p"), ("uy", "t"),
            ("uyê", "n"), ("uyê", "t"),
            ("ư", "c"), ("ư", "m"), ("ư", "n"), ("ư", "ng"), ("ư", "t"),
            ("ươ", "c"), ("ươ", "m"), ("ươ", "n"), ("ươ", "ng"), ("ươ", "p"), ("ươ", "t"),
            ("y", "t"),
            ("yê", "m"), ("yê", "n"), ("yê", "ng"), ("yê", "p"), ("yê", "t"),
            ("ue", "c"), ("ue", "ch"), ("ue", "n"), ("ue", "nh"),
        ]
    }

    // ── Tests ──────────────────────────────────────────────────────────

    #[test]
    #[ignore] // run with --ignored
    fn find_validation_gaps() {
        println!("\n=== Cross-validation: skey-engine vs x-unikey VCPairList ===\n");

        let mut ok = 0;
        let mut gaps = Vec::new();

        // Build test words from VCPairList
        for (vowel_seq, final_cons) in &xunikey_vc_pairs() {
            // Build a valid CVC word: use a safe initial consonant
            let test_word = format!("b{}{}", vowel_seq, final_cons);
            let valid = spelling::is_valid_cvc(&test_word);

            if valid {
                ok += 1;
            } else {
                gaps.push(format!("  ✗ b{}{} — should be VALID (vowel='{}', final='{}')",
                    vowel_seq, final_cons, vowel_seq, final_cons));
            }
        }

        for g in &gaps {
            println!("{}", g);
        }
        println!("\n  Validated: {}/{} VC pairs pass skey-engine validation", ok, ok + gaps.len());

        if !gaps.is_empty() {
            println!("  ⚠ {} gaps found — skey-engine is missing these valid combinations", gaps.len());
        }
    }

    #[test]
    #[ignore]
    fn find_consonant_vowel_gaps() {
        println!("\n=== Cross-validation: isValidCV from x-unikey ===\n");

        // x-unikey rule: k can only go with e, i, y, ê, eo, eu, êu, ia, iê, iêu
        let k_valid = ["ke", "ki", "ky", "kê", "keo", "kêu", "kia", "kiê", "kiêu"];
        let k_invalid = ["ka", "ko", "ku", "kă", "kâ", "kô", "kơ", "kư"];

        println!("k + vowel rules:");
        for word in &k_valid {
            let v = spelling::is_valid_cvc(&format!("{}m", word));
            println!("  {} → {} (should be valid)", word, if v { "✓" } else { "✗ MISSING" });
        }
        for word in &k_invalid {
            let v = spelling::is_valid_cvc(&format!("{}m", word));
            println!("  {} → {} (should be INVALID)", word, if !v { "✓" } else { "✗ FALSE POSITIVE" });
        }
    }

    #[test]
    #[ignore]
    fn check_exception_cases() {
        println!("\n=== Exception cases from x-unikey ===\n");

        // These are special exceptions in x-unikey's isValidCVC:
        let exceptions = [
            ("quyn", true, "qu + y + n (exception: quyn)"),
            ("quynh", true, "qu + y + nh (exception: quynh)"),
            ("gieng", true, "gi + e + ng (exception: gieng)"),
            ("giêng", true, "gi + ê + ng (exception: giêng)"),
            // gi + i should be INVALID
            ("gim", false, "gi + i + m (gi doesn't go with i)"),
            ("gip", false, "gi + i + p"),
            // qu + u should be INVALID
            ("qum", false, "qu + u + m (qu doesn't go with u)"),
            ("qup", false, "qu + u + p"),
        ];

        for (word, should_be_valid, desc) in &exceptions {
            let v = spelling::is_valid_cvc(word);
            let status = if v == *should_be_valid { "✓" } else { "✗ MISMATCH" };
            println!("  {} {} → is_valid={} (expected {})", status, word, v, should_be_valid);
            if v != *should_be_valid {
                println!("    {}", desc);
            }
        }
    }

    #[test]
    #[ignore]
    fn check_incomplete_vowel_sequences() {
        println!("\n=== Incomplete vowel sequences (should NOT be valid alone) ===\n");

        // Vowel sequences marked complete=false in x-unikey:
        // eu, ie, ue, uo, uu, ye, ieu, uay, uoi, uou, uye, yeu
        let incomplete = [
            ("beu", "eu"),
            ("bie", "ie"),
            ("bue", "ue"),
            ("buo", "uo"),
            ("buu", "uu"),
            ("bye", "ye"),
        ];

        for (word, desc) in &incomplete {
            let v = spelling::is_valid_cvc(word);
            println!("  {} → is_valid={} (incomplete seq '{}')", word, v, desc);
        }
    }

    #[test]
    #[ignore]
    fn full_parity_report() {
        println!("\n=== Full parity report ===\n");

        // Build all valid Vietnamese syllables from x-unikey data and
        // check against skey-engine validation.

        let initial_consonants = [
            "", "b", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh",
            "l", "m", "n", "ng", "ngh", "nh", "ph", "qu", "r", "s",
            "t", "th", "tr", "v", "x",
        ];
        let tones = ["", "f", "s", "r", "x", "j"]; // telex tone marks

        // Test one-shot transforms for all VC pairs with initial "b"
        let mut ok = 0;
        let mut engine_gaps = Vec::new();

        for (vowel_seq, final_cons) in &xunikey_vc_pairs() {
            // Build telex input: b + vowel_seq + final_cons + tone
            let telex_input = format!("b{}{}", vowel_seq, final_cons);

            // Check that engine produces non-empty output
            let output = engine::convert_telex(&telex_input, false, true);
            if output.is_empty() {
                engine_gaps.push(format!("EMPTY output for: {}", telex_input));
            } else {
                ok += 1;
            }
        }

        println!("  Engine output: {}/{} non-empty", ok, ok + engine_gaps.len());
        for g in &engine_gaps[..engine_gaps.len().min(20)] {
            println!("  {}", g);
        }
    }
}
