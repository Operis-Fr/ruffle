//! Atom (grapheme) and word boundary lookup, as used by
//! `TextBlock.findNext/PreviousAtomBoundary` and
//! `TextBlock.findNext/PreviousWordBoundary`.
//!
//! All indices are UTF-16 code unit indices, as in Flash Player.

use crate::string::WStr;

fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..0xDC00).contains(&unit)
}

fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..0xE000).contains(&unit)
}

/// Decode the character starting at the given index.
/// Unpaired surrogates are decoded as U+FFFD.
fn char_at(text: &WStr, index: usize) -> char {
    let unit = text.at(index);
    if is_high_surrogate(unit)
        && let Some(low) = text.get(index + 1)
        && is_low_surrogate(low)
    {
        let c = 0x10000 + ((u32::from(unit) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
        return char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER);
    }
    char::from_u32(u32::from(unit)).unwrap_or(char::REPLACEMENT_CHARACTER)
}

/// Decode the character ending just before the given index.
/// Returns the character and its starting index.
fn char_before(text: &WStr, index: usize) -> (char, usize) {
    let unit = text.at(index - 1);
    if is_low_surrogate(unit) && index >= 2 && is_high_surrogate(text.at(index - 2)) {
        (char_at(text, index - 2), index - 2)
    } else {
        (char_at(text, index - 1), index - 1)
    }
}

/// Whether this character extends the previous atom (combining marks,
/// variation selectors, zero width joiner, emoji modifiers).
fn is_extending(c: char) -> bool {
    matches!(c as u32,
        0x0300..=0x036F
        | 0x0483..=0x0489
        | 0x0591..=0x05BD
        | 0x0610..=0x061A
        | 0x064B..=0x065F
        | 0x0670
        | 0x06D6..=0x06DC
        | 0x06DF..=0x06E4
        | 0x0E31
        | 0x0E34..=0x0E3A
        | 0x0E47..=0x0E4E
        | 0x1AB0..=0x1AFF
        | 0x1DC0..=0x1DFF
        | 0x200C..=0x200D
        | 0x20D0..=0x20FF
        | 0x302A..=0x302F
        | 0x3099..=0x309A
        | 0xFE00..=0xFE0F
        | 0xFE20..=0xFE2F
        | 0x1F3FB..=0x1F3FF
        | 0xE0100..=0xE01EF
    )
}

/// Returns the index of the atom boundary following `index`
/// (`index` must be lower than the text length).
pub fn next_atom_boundary(text: &WStr, index: usize) -> usize {
    let len = text.len();
    let mut pos = index + char_at(text, index).len_utf16();
    let mut previous_was_zwj = char_at(text, index) == '\u{200D}';
    while pos < len {
        let c = char_at(text, pos);
        if !is_extending(c) && !previous_was_zwj {
            break;
        }
        previous_was_zwj = c == '\u{200D}';
        pos += c.len_utf16();
    }
    pos.min(len)
}

/// Returns the index of the atom boundary preceding `index`
/// (`index` must be greater than 0 and at most the text length).
pub fn previous_atom_boundary(text: &WStr, index: usize) -> usize {
    let (mut c, mut pos) = char_before(text, index);
    while pos > 0 {
        let (prev, prev_pos) = char_before(text, pos);
        if !is_extending(c) && prev != '\u{200D}' {
            break;
        }
        c = prev;
        pos = prev_pos;
    }
    pos
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WordClass {
    Space,
    Word,
    /// Punctuation and symbols: each of them forms its own word.
    Other,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || is_extending(c)
}

/// Characters which don't break a word when surrounded by word characters,
/// e.g. `l'arbre`, `3.14` or `1,5`.
fn is_mid_word(c: char) -> bool {
    matches!(c, '\'' | '\u{2019}' | '.' | ',' | '\u{00B7}')
}

fn word_class_at(text: &WStr, index: usize) -> WordClass {
    let c = char_at(text, index);
    if c.is_whitespace() {
        WordClass::Space
    } else if is_word_char(c) {
        WordClass::Word
    } else if is_mid_word(c) && index > 0 && index + 1 < text.len() {
        let (before, _) = char_before(text, index);
        let after = char_at(text, index + c.len_utf16());
        if is_word_char(before) && is_word_char(after) {
            WordClass::Word
        } else {
            WordClass::Other
        }
    } else {
        WordClass::Other
    }
}

/// Returns the index of the word boundary following `index`, not including
/// the character at `index` (`index` must be lower than the text length).
pub fn next_word_boundary(text: &WStr, index: usize) -> usize {
    let len = text.len();
    let class = word_class_at(text, index);
    let mut pos = next_atom_boundary(text, index);
    if class == WordClass::Other {
        return pos;
    }
    while pos < len && word_class_at(text, pos) == class {
        pos = next_atom_boundary(text, pos);
    }
    pos
}

/// Returns the index of the word boundary preceding `index`
/// (`index` must be greater than 0 and at most the text length).
pub fn previous_word_boundary(text: &WStr, index: usize) -> usize {
    let mut pos = previous_atom_boundary(text, index);
    let class = word_class_at(text, pos);
    if class == WordClass::Other {
        return pos;
    }
    while pos > 0 {
        let prev = previous_atom_boundary(text, pos);
        if word_class_at(text, prev) != class {
            break;
        }
        pos = prev;
    }
    pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::WString;

    fn w(s: &str) -> WString {
        WString::from_utf8(s)
    }

    #[test]
    fn atoms_simple() {
        let t = w("abc");
        assert_eq!(next_atom_boundary(&t, 0), 1);
        assert_eq!(next_atom_boundary(&t, 2), 3);
        assert_eq!(previous_atom_boundary(&t, 3), 2);
        assert_eq!(previous_atom_boundary(&t, 1), 0);
    }

    #[test]
    fn atoms_combining_and_surrogates() {
        // "e" + combining acute accent, then U+1F600 (surrogate pair), then "x"
        let t = w("e\u{301}\u{1F600}x");
        assert_eq!(t.len(), 5);
        assert_eq!(next_atom_boundary(&t, 0), 2);
        assert_eq!(next_atom_boundary(&t, 2), 4);
        assert_eq!(next_atom_boundary(&t, 4), 5);
        assert_eq!(previous_atom_boundary(&t, 5), 4);
        assert_eq!(previous_atom_boundary(&t, 4), 2);
        assert_eq!(previous_atom_boundary(&t, 2), 0);
    }

    #[test]
    fn atoms_zwj_sequence() {
        // Family emoji: man ZWJ woman
        let t = w("\u{1F468}\u{200D}\u{1F469}a");
        assert_eq!(next_atom_boundary(&t, 0), 5);
        assert_eq!(previous_atom_boundary(&t, 5), 0);
    }

    #[test]
    fn words() {
        let t = w("Hello  world, l'été 3.14!\u{2029}");
        // "Hello" | "  " | "world" | "," | " " | "l'été" | " " | "3.14" | "!" | PS
        assert_eq!(next_word_boundary(&t, 0), 5);
        assert_eq!(next_word_boundary(&t, 2), 5);
        assert_eq!(next_word_boundary(&t, 5), 7);
        assert_eq!(next_word_boundary(&t, 7), 12);
        assert_eq!(next_word_boundary(&t, 12), 13);
        assert_eq!(next_word_boundary(&t, 13), 14);
        assert_eq!(next_word_boundary(&t, 14), 19);
        assert_eq!(next_word_boundary(&t, 20), 24);
        assert_eq!(next_word_boundary(&t, 24), 25);
        assert_eq!(next_word_boundary(&t, 25), 26);

        assert_eq!(previous_word_boundary(&t, 5), 0);
        assert_eq!(previous_word_boundary(&t, 3), 0);
        assert_eq!(previous_word_boundary(&t, 7), 5);
        assert_eq!(previous_word_boundary(&t, 12), 7);
        assert_eq!(previous_word_boundary(&t, 13), 12);
        assert_eq!(previous_word_boundary(&t, 19), 14);
        assert_eq!(previous_word_boundary(&t, 25), 24);
    }

    #[test]
    fn words_punctuation_at_edges() {
        let t = w("'a.");
        assert_eq!(next_word_boundary(&t, 0), 1);
        assert_eq!(next_word_boundary(&t, 1), 2);
        assert_eq!(previous_word_boundary(&t, 3), 2);
        assert_eq!(previous_word_boundary(&t, 2), 1);
    }
}
