use criterion::{black_box, criterion_group, criterion_main, Criterion};
use skey_engine::engine;

// ── Single-shot transforms ─────────────────────────────────────────

fn bench_single_char(c: &mut Criterion) {
    c.bench_function("telex_single_a", |b| {
        b.iter(|| engine::convert_telex(black_box("a"), true, false))
    });
    c.bench_function("telex_single_as_accent", |b| {
        b.iter(|| engine::convert_telex(black_box("as"), true, false))
    });
}

fn bench_short_words(c: &mut Criterion) {
    c.bench_function("telex_tooi", |b| {
        b.iter(|| engine::convert_telex(black_box("tooi"), true, false))
    });
    c.bench_function("telex_tieengs", |b| {
        b.iter(|| engine::convert_telex(black_box("tieengs"), true, false))
    });
    c.bench_function("telex_nguwowif", |b| {
        b.iter(|| engine::convert_telex(black_box("nguwowif"), true, false))
    });
    c.bench_function("telex_dduwowcj", |b| {
        b.iter(|| engine::convert_telex(black_box("dduwowcj"), true, false))
    });
}

fn bench_long_text(c: &mut Criterion) {
    let sentence = "tooi ddeens tuwf vuofn cuar mej tooi, maf toi laf con trai duy nhaats";
    c.bench_function("telex_sentence", |b| {
        b.iter(|| engine::convert_telex(black_box(sentence), true, false))
    });
}

// ── Incremental typing simulation (stress test) ─────────────────────

fn bench_incremental_typing(c: &mut Criterion) {
    // Simulate fast sequential keystrokes — the real-world IME usage pattern.
    // Each new keystroke calls transform() with the accumulated input.
    let typing_sequences: &[&[&str]] = &[
        // gõ "xin chào"
        &["x", "xi", "xin", "xin ", "xin c", "xin ch", "xin cha", "xin chaf", "xin chafo"],
        // gõ "tiếng Việt"
        &["t", "ti", "tie", "tiee", "tieen", "tieeng", "tieengs",
          "tieengs ", "tieengs V", "tieengs Vi", "tieengs Vie", "tieengs Viee", "tieengs Viej", "tieengs Vieejt"],
        // gõ "được"
        &["d", "dd", "ddu", "dduw", "dduwo", "dduwow", "dduwowc", "dduwowcj"],
        // gõ "người"
        &["n", "ng", "ngu", "nguw", "nguwo", "nguwow", "nguwowi", "nguwowif"],
        // gõ nhanh "toàn" — tone reassignment
        &["t", "to", "tof", "tofa", "tofan"],
    ];

    let mut group = c.benchmark_group("incremental_typing");
    for (seq_idx, seq) in typing_sequences.iter().enumerate() {
        group.bench_function(format!("seq_{}", seq_idx), |b| {
            b.iter(|| {
                for input in *seq {
                    let _ = engine::convert_telex(black_box(input), false, true);
                }
            })
        });
    }
    group.finish();
}

// ── Stress: rapid single-char accumulation ──────────────────────────

fn bench_rapid_accumulation(c: &mut Criterion) {
    // Worst case: 50 single-char transforms (simulates 50 rapid keystrokes)
    let chars: Vec<String> = "tieengs vieejt dduwowcj nguwowif".chars()
        .scan(String::new(), |acc, ch| {
            acc.push(ch);
            Some(acc.clone())
        })
        .collect();

    c.bench_function("rapid_50_keystrokes", |b| {
        b.iter(|| {
            for s in &chars {
                let _ = engine::convert_telex(black_box(s.as_str()), false, true);
            }
        })
    });
}

// ── VNI method ──────────────────────────────────────────────────────

fn bench_vni(c: &mut Criterion) {
    c.bench_function("vni_tie61ng", |b| {
        b.iter(|| engine::convert_vni(black_box("tie61ng"), false, true))
    });
    c.bench_function("vni_vie65t", |b| {
        b.iter(|| engine::convert_vni(black_box("vie65t"), false, true))
    });
}

// ── TeipVni (combined mode) ─────────────────────────────────────────

fn bench_teipvni(c: &mut Criterion) {
    c.bench_function("teipvni_vie61t", |b| {
        b.iter(|| engine::convert_teip_vni(black_box("vie61t"), false, true))
    });
    c.bench_function("teipvni_tie61ng", |b| {
        b.iter(|| engine::convert_teip_vni(black_box("tie61ng"), false, true))
    });
}

// ── Charset conversion ─────────────────────────────────────────────

fn bench_charset(c: &mut Criterion) {
    let text = "tiếng Việt là ngôn ngữ của người Việt Nam, được viết bằng chữ Latinh với các dấu thanh điệu";
    c.bench_function("charset_encode_tcvn3", |b| {
        b.iter(|| skey_engine::charset::encode(black_box(text), skey_engine::charset::VietCharset::TCVN3))
    });
    c.bench_function("charset_decode_tcvn3", |b| {
        let encoded = skey_engine::charset::encode(text, skey_engine::charset::VietCharset::TCVN3);
        b.iter(|| skey_engine::charset::decode(black_box(&encoded), skey_engine::charset::VietCharset::TCVN3))
    });
    c.bench_function("charset_encode_vniwin", |b| {
        b.iter(|| skey_engine::charset::encode(black_box(text), skey_engine::charset::VietCharset::VNIWin))
    });
    c.bench_function("charset_decode_vniwin", |b| {
        let encoded = skey_engine::charset::encode(text, skey_engine::charset::VietCharset::VNIWin);
        b.iter(|| skey_engine::charset::decode(black_box(&encoded), skey_engine::charset::VietCharset::VNIWin))
    });
    c.bench_function("charset_remove_tone", |b| {
        b.iter(|| skey_engine::charset::remove_tone(black_box(text)))
    });
}

criterion_group!(
    benches,
    bench_single_char,
    bench_short_words,
    bench_long_text,
    bench_incremental_typing,
    bench_rapid_accumulation,
    bench_vni,
    bench_teipvni,
    bench_charset,
);
criterion_main!(benches);
