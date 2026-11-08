//! Embedding levels for right-to-left text: the part of the Unicode
//! Bidirectional Algorithm (UAX #9) a slide paragraph needs (`T2D1-04`).
//!
//! New here: OpenCalc draws a cell with the browser, which does its own bidi, so it
//! has none to copy. The shaper (`harfrust`) joins and orders the glyphs of one run
//! in one direction; deciding which runs are right to left, and the order runs sit
//! in on a line, is this module's.
//!
//! What is implemented: the paragraph's base level from `pPr/@rtl` (rule P3 is not
//! applied: PowerPoint takes a paragraph's direction from `rtl`, not from its first
//! letter), rules W2, W3 and W7 for numbers, N1 and N2 for neutrals, I1 and I2, and
//! L2's reordering ([`visual_order`]). Not implemented, and drawn at the paragraph's
//! level: explicit embeddings and isolates (`U+202A` to `U+202E`, `U+2066` to
//! `U+2069`), number separators (W4 to W6, so `1,000` between Arabic letters
//! reorders by its parts) and mirrored brackets (L4).

/// A character's bidi class, reduced to what the rules here distinguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// Left to right letter.
    L,
    /// Right to left letter (Hebrew; Arabic after W3).
    R,
    /// Arabic letter, before W3.
    Al,
    /// European digit.
    En,
    /// Arabic-Indic digit.
    An,
    /// Everything else: spaces, punctuation, symbols, marks.
    N,
}

fn class(ch: char) -> Class {
    match u32::from(ch) {
        0x0660..=0x0669 | 0x066B | 0x066C | 0x06DD => Class::An,
        0x0590..=0x05FF | 0x07C0..=0x085F | 0xFB1D..=0xFB4F => Class::R,
        0x0600..=0x06FF | 0x0700..=0x07BF | 0x0860..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => {
            // Arabic combining marks take the class of the letter they sit on;
            // treated as neutral, N1 gives them that letter's side.
            if is_arabic_mark(ch) {
                Class::N
            } else {
                Class::Al
            }
        }
        0x10800..=0x10FFF | 0x1E800..=0x1EFFF => Class::R,
        _ if ch.is_ascii_digit() || ('\u{06F0}'..='\u{06F9}').contains(&ch) => Class::En,
        _ if ch.is_alphabetic() => Class::L,
        _ => Class::N,
    }
}

fn is_arabic_mark(ch: char) -> bool {
    matches!(u32::from(ch), 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06ED)
}

/// Whether `text` has a character drawn right to left.
#[must_use]
pub fn has_rtl(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(class(c), Class::R | Class::Al | Class::An))
}

/// The embedding level of each character of `text`, in order: even is left to
/// right, odd right to left. `rtl` is the paragraph's base direction.
#[must_use]
pub fn levels(text: &str, rtl: bool) -> Vec<u8> {
    let base = u8::from(rtl);
    let mut classes: Vec<Class> = text.chars().map(class).collect();
    // W2: a European digit after an Arabic letter is an Arabic digit. W3: AL is R.
    // W7: a European digit after a left-to-right letter (or at the start of a
    // left-to-right paragraph) is L.
    let sos = if rtl { Class::R } else { Class::L };
    let mut last_strong = sos;
    for c in &mut classes {
        match *c {
            Class::L | Class::R => last_strong = *c,
            Class::Al => {
                last_strong = Class::Al;
                *c = Class::R;
            }
            Class::En if last_strong == Class::Al => *c = Class::An,
            Class::En if last_strong == Class::L => *c = Class::L,
            _ => {}
        }
    }
    // N1 and N2: a run of neutrals between two sides of one direction takes it
    // (digits count as right to left); otherwise the paragraph's.
    let side = |c: Class| match c {
        Class::L => Some(false),
        Class::R | Class::Al | Class::En | Class::An => Some(true),
        Class::N => None,
    };
    let mut i = 0;
    while i < classes.len() {
        if classes.get(i) != Some(&Class::N) {
            i += 1;
            continue;
        }
        let start = i;
        while classes.get(i) == Some(&Class::N) {
            i += 1;
        }
        let before = start
            .checked_sub(1)
            .and_then(|b| classes.get(b))
            .and_then(|&c| side(c))
            .unwrap_or(rtl);
        let after = classes.get(i).and_then(|&c| side(c)).unwrap_or(rtl);
        let resolved = if before == after {
            if before { Class::R } else { Class::L }
        } else if rtl {
            Class::R
        } else {
            Class::L
        };
        for c in classes.iter_mut().take(i).skip(start) {
            *c = resolved;
        }
    }
    // I1 and I2.
    classes
        .into_iter()
        .map(|c| match (base % 2, c) {
            (0, Class::R | Class::Al) => base + 1,
            (0, Class::En | Class::An) => base + 2,
            (0, _) => base,
            (_, Class::L | Class::En | Class::An) => base + 1,
            _ => base,
        })
        .collect()
}

/// The order in which pieces with these levels are drawn, left to right: rule L2,
/// reversing every maximal sequence at or above each odd level, highest first.
#[must_use]
pub fn visual_order(levels: &[u8]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..levels.len()).collect();
    let Some(&highest) = levels.iter().max() else {
        return order;
    };
    let lowest_odd = levels
        .iter()
        .copied()
        .filter(|l| l % 2 == 1)
        .min()
        .unwrap_or(highest.saturating_add(1));
    let mut level = highest;
    while level >= lowest_odd && level > 0 {
        let mut i = 0;
        while i < order.len() {
            let at = |k: usize| order.get(k).and_then(|&o| levels.get(o)).copied();
            if at(i).is_some_and(|l| l >= level) {
                let start = i;
                while at(i).is_some_and(|l| l >= level) {
                    i += 1;
                }
                if let Some(run) = order.get_mut(start..i) {
                    run.reverse();
                }
            } else {
                i += 1;
            }
        }
        level -= 1;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_is_right_to_left_and_latin_left_to_right_in_either_paragraph() {
        assert_eq!(levels("ab", false), vec![0, 0]);
        assert_eq!(levels("\u{0645}\u{0646}", false), vec![1, 1]);
        assert_eq!(levels("\u{0645}\u{0646}", true), vec![1, 1]);
        assert_eq!(levels("ab", true), vec![2, 2]);
    }

    #[test]
    fn a_space_between_arabic_words_is_right_to_left_and_digits_stay_left_to_right() {
        // م ن space 1 2
        let l = levels("\u{0645} \u{0646} 12", true);
        assert_eq!(l, vec![1, 1, 1, 1, 2, 2]);
        // In a left-to-right paragraph, a space between Latin and Arabic takes the
        // paragraph's direction.
        assert_eq!(levels("a \u{0645}", false), vec![0, 0, 1]);
    }

    #[test]
    fn reordering_reverses_right_to_left_runs_and_keeps_numbers_inside_them_forward() {
        assert_eq!(visual_order(&[0, 0, 0]), vec![0, 1, 2]);
        assert_eq!(visual_order(&[1, 1, 1]), vec![2, 1, 0]);
        // Arabic, number, Arabic in a right-to-left paragraph: the number stays
        // between them, drawn forward.
        assert_eq!(visual_order(&[1, 2, 1]), vec![2, 1, 0]);
        assert_eq!(visual_order(&[0, 1, 1, 0]), vec![0, 2, 1, 3]);
    }
}
