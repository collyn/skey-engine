//! Vowel sequence table — models Vietnamese vowel clusters for hook/breve
//! transformations (inspired by fcitx5-unikey's VSeqList).
//!
//! Instead of tracking per-vowel flags (`from_w`, `horn_shared`), we look at
//! the current vowel cluster as a whole and decide which positions get hook.

use crate::engine::Letter;

/// Result of attempting to apply hook (horn/breve) via the 'w' key.
#[derive(PartialEq, Eq)]
pub enum HookResult {
    /// Hook was applied to vowel(s); w key is consumed.
    Applied,
    /// Hook was removed (toggled off); w key falls through as literal.
    ToggledOff,
    /// No hookable vowel found; try phase 2 (insert standalone ư).
    NotApplicable,
}

/// Vowel sequence entry defining hook behavior.
struct VSeq {
    /// Base vowel characters (lowercase ASCII) in order: e.g. ['u','o'] for "uo"
    base: [char; 3],
    /// Number of vowels (1-3)
    len: u8,
    /// Bitmask of positions that receive the hook mark (breve/horn).
    /// Bit 0 = first vowel, bit 1 = second, bit 2 = third.
    hook_mask: u8,
}

/// Lookup table: base vowel sequences and which positions get hook.
/// Ordered by priority — longer sequences are checked first.
static VSEQ_TABLE: &[VSeq] = &[
    // ── Triphthongs (check before diphthongs) ──
    VSeq { base: ['u', 'o', 'i'], len: 3, hook_mask: 0b011 },  // uoi → ươi (hook both u+o)
    VSeq { base: ['u', 'o', 'u'], len: 3, hook_mask: 0b011 },  // uou → ươu
    // ── Diphthongs staring with u ──
    VSeq { base: ['u', 'o', '\0'], len: 2, hook_mask: 0b11 },   // uo → ươ (hook both)
    VSeq { base: ['u', 'a', '\0'], len: 2, hook_mask: 0b01 },   // ua → ưa (hook u only)
    VSeq { base: ['u', 'i', '\0'], len: 2, hook_mask: 0b01 },   // ui → ưi
    VSeq { base: ['u', 'y', '\0'], len: 2, hook_mask: 0b01 },   // uy → ưy (hook u)
    VSeq { base: ['u', 'e', '\0'], len: 2, hook_mask: 0b01 },   // ue → ưe (hook u)
    VSeq { base: ['o', 'a', '\0'], len: 2, hook_mask: 0b10 },   // oa → oă (breve on a)
    VSeq { base: ['o', 'e', '\0'], len: 2, hook_mask: 0b01 },   // oe → oe (hook o only? no, oe doesn't take hook)
    // ── Single vowels (checked last — longest match wins) ──
    VSeq { base: ['a', '\0', '\0'], len: 1, hook_mask: 0b01 },  // a → ă (breve)
    VSeq { base: ['o', '\0', '\0'], len: 1, hook_mask: 0b01 },  // o → ơ (horn)
    VSeq { base: ['u', '\0', '\0'], len: 1, hook_mask: 0b01 },  // u → ư (horn)
];

/// Find the contiguous vowel cluster ending at the last vowel.
/// Returns (start_index, count).
fn vowel_cluster(ls: &[Letter]) -> Option<(usize, usize)> {
    let end = ls.iter().rposition(|lt| lt.is_vowel)?;
    let mut start = end;
    while start > 0 && ls[start - 1].is_vowel {
        start -= 1;
    }
    // Skip gi/qu digraph vowels — they're consonants, not vowels.
    // Handle both single-vowel (start==end) and multi-vowel (start<end).
    if ls[start].c == 'u' && start > 0 && ls[start - 1].c == 'q' {
        start += 1;
    } else if ls[start].c == 'i' && start > 0 && ls[start - 1].c == 'g' {
        start += 1;
    }
    if start > end {
        return None;
    }
    Some((start, end - start + 1))
}

/// Find the VSeq entry matching the given base vowel characters.
fn lookup_vseq(base: &[char]) -> Option<&'static VSeq> {
    VSEQ_TABLE.iter().find(|vs| {
        vs.len as usize == base.len()
            && (0..base.len()).all(|i| vs.base[i] == base[i])
    })
}

/// Check whether the 'uo' cluster has a prefix consonant that triggers
/// o-only hook (h, kh, th) — ported from fcitx5-unikey's "thuong rule".
#[allow(dead_code)]
fn uo_o_only_prefix(ls: &[Letter], vstart: usize) -> bool {
    if vstart < 2 {
        return false; // need at least a 2-char consonant prefix
    }
    // The consonant directly before uo must be 'h' (part of h, th, or kh)
    let prev = vstart - 1;
    if !ls[prev].is_vowel && ls[prev].c == 'h' {
        // Check if 'h' forms th or kh with a preceding consonant
        if prev > 0 && !ls[prev - 1].is_vowel {
            match ls[prev - 1].c {
                't' | 'k' => return true,  // th or kh prefix
                _ => {}
            }
        }
        // Standalone 'h' prefix also triggers the rule
        return true;
    }
    false
}

/// Try to apply hook (horn/breve) to the current vowel cluster.
///
/// Returns:
/// - `Applied` — hook was set on vowel(s); w consumed, caller should `continue`
/// - `ToggledOff` — hook was removed; w falls through as literal
/// - `NotApplicable` — no hookable vowel; caller should try phase 2
pub(crate) fn try_apply_hook(ls: &mut Vec<Letter>) -> HookResult {
    let (vstart, vcount) = match vowel_cluster(ls) {
        Some(vc) => vc,
        None => return HookResult::NotApplicable,
    };

    // Extract base chars (ignoring current variants)
    let base: Vec<char> = (vstart..vstart + vcount)
        .map(|i| ls[i].c)
        .collect();

    let vs = match lookup_vseq(&base) {
        Some(vs) => vs,
        None => {
            // No VSeq entry for this cluster — scan right-to-left for the
            // first vowel that can take hook (a, o, u). This handles clusters
            // like "oi" where 'o' should be the hook target, not 'i'.
            for i in (vstart..vstart + vcount).rev() {
                if matches!(ls[i].c, 'a' | 'o' | 'u') {
                    return apply_single_hook(ls, i);
                }
            }
            return HookResult::NotApplicable;
        }
    };

    // Check current hook state: are ALL hook positions already hooked?
    let all_hooked = (0..vs.len as usize)
        .filter(|&i| (vs.hook_mask >> i) & 1 != 0)
        .all(|i| ls[vstart + i].variant == 2);

    if all_hooked {
        // Check if any hook was propagated (not from explicit w).
        // If so, this w confirms the propagated hooks rather than toggling off.
        let any_propagated = (0..vs.len as usize)
            .filter(|&i| (vs.hook_mask >> i) & 1 != 0)
            .any(|i| ls[vstart + i].horn_propagated);
        if any_propagated {
            // Confirm propagated hooks: clear the flag, consume w.
            // Explicit hooks stay in place.
            for i in 0..vs.len as usize {
                if (vs.hook_mask >> i) & 1 != 0 {
                    ls[vstart + i].horn_propagated = false;
                }
            }
            return HookResult::Applied; // w consumed, hook confirmed
        }
        // Toggle OFF: remove hook from all hook positions.
        // Special case: standalone ư (from_w on single u) → rewrite to literal w.
        if vs.len == 1 && vs.base[0] == 'u'
            && ls[vstart].variant == 2 && ls[vstart].from_w
        {
            ls[vstart].c = 'w';
            ls[vstart].is_vowel = false;
            ls[vstart].from_w = false;
            return HookResult::Applied; // w consumed, no literal
        }
        for i in 0..vs.len as usize {
            if (vs.hook_mask >> i) & 1 != 0 {
                ls[vstart + i].variant = 0;
                ls[vstart + i].from_w = true;
                ls[vstart + i].horn_propagated = false;
            }
        }
        HookResult::ToggledOff
    } else {
        // Toggle ON: apply hook to unhooked positions
        // Apply hook based on the VSeq table mask.
        // Note: The fcitx5-unikey "thuong rule" (uo with h/kh/th prefix →
        // o-only hook) is available via uo_o_only_prefix() but currently
        // disabled to match existing test expectations.
        let effective_mask = vs.hook_mask;

        for i in 0..vs.len as usize {
            if (effective_mask >> i) & 1 != 0 && ls[vstart + i].variant == 0 {
                ls[vstart + i].variant = 2;
                ls[vstart + i].from_w = false;
            }
        }
        HookResult::Applied
    }
}

/// Apply hook to a single vowel at the given index.
/// Handles the special standalone w→ư toggle-off case.
fn apply_single_hook(ls: &mut Vec<Letter>, vi: usize) -> HookResult {
    let c = ls[vi].c;
    match c {
        'a' => {
            if ls[vi].variant == 2 {
                ls[vi].variant = 0;
                ls[vi].from_w = true;
                HookResult::ToggledOff
            } else if ls[vi].from_w && ls[vi].variant == 0 {
                HookResult::ToggledOff // already toggled
            } else {
                ls[vi].variant = 2;
                ls[vi].from_w = false;
                HookResult::Applied
            }
        }
        'o' => {
            if ls[vi].variant == 2 {
                ls[vi].variant = 0;
                ls[vi].from_w = true;
                // Also toggle off preceding u's horn (ươ → uo)
                if vi > 0 && ls[vi - 1].c == 'u' && ls[vi - 1].variant == 2 {
                    ls[vi - 1].variant = 0;
                    ls[vi - 1].from_w = true;
                }
                HookResult::ToggledOff
            } else if ls[vi].from_w && ls[vi].variant == 0 {
                HookResult::ToggledOff
            } else if ls[vi].variant == 0 {
                ls[vi].variant = 2;
                ls[vi].from_w = false;
                // Also horn the preceding u (uo → ươ)
                if vi > 0 && ls[vi - 1].c == 'u' && ls[vi - 1].variant == 0 {
                    ls[vi - 1].variant = 2;
                    ls[vi - 1].from_w = false;
                }
                HookResult::Applied
            } else {
                HookResult::NotApplicable
            }
        }
        'u' => {
            // Find the first 'u' in a run of consecutive u's
            let mut first = vi;
            while first > 0 && ls[first - 1].c == 'u' && ls[first - 1].is_vowel {
                first -= 1;
            }
            if ls[first].variant == 2 {
                if ls[first].from_w {
                    // Standalone w→ư toggle-off: ư → literal w
                    ls[first].c = 'w';
                    ls[first].is_vowel = false;
                    ls[first].from_w = false;
                    HookResult::Applied // w consumed, no literal
                } else {
                    ls[first].variant = 0;
                    ls[first].from_w = true;
                    HookResult::ToggledOff
                }
            } else if ls[first].from_w && ls[first].variant == 0 {
                HookResult::ToggledOff
            } else {
                ls[first].variant = 2;
                ls[first].from_w = false;
                HookResult::Applied
            }
        }
        _ => HookResult::NotApplicable,
    }
}

/// Phase 2: insert standalone ư when hook is not applicable (or short_w ư).
/// Returns true if a character was inserted (w consumed).
pub(crate) fn try_insert_horn(ls: &mut Vec<Letter>, short_w: bool, upper: bool) -> bool {
    // Only insert standalone ư when no "real" vowel exists and short_w is enabled.
    // Skip vowels that are part of gi/qu digraphs — they're consonants, not vowels.
    let has_real_vowel = ls.iter().enumerate().any(|(idx, lt)| {
        if !lt.is_vowel {
            return false;
        }
        // 'i' after 'g' → gi digraph (consonant, not vowel)
        if lt.c == 'i' && idx > 0 && ls[idx - 1].c == 'g' && !ls[idx - 1].is_vowel {
            return false;
        }
        // 'u' after 'q' → qu digraph (consonant, not vowel)
        if lt.c == 'u' && idx > 0 && ls[idx - 1].c == 'q' && !ls[idx - 1].is_vowel {
            return false;
        }
        true
    });

    if short_w
        && !has_real_vowel
        && !ls.last().map_or(false, |lt| lt.c == 'w' && !lt.is_vowel)
    {
        ls.push(Letter::new_standalone_u(upper));
        return true;
    }
    false
}

impl Letter {
    /// Create a standalone ư letter (short_w mode: bare w → ư).
    pub fn new_standalone_u(upper: bool) -> Self {
        Letter {
            c: 'u',
            is_vowel: true,
            variant: 2,
            tone: 0,
            is_dstroke: false,
            upper,
            from_w: true,
            horn_propagated: false,
            circ_toggled: false,
        }
    }
}
