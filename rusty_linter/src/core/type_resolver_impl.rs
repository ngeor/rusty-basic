use rusty_parser::{DefType, LetterRange, TypeQualifier};

use crate::core::TypeResolver;

const LETTER_COUNT: usize = 26;

#[derive(Debug)]
pub struct TypeResolverImpl {
    ranges: [TypeQualifier; LETTER_COUNT],
}

impl Default for TypeResolverImpl {
    fn default() -> Self {
        Self::new()
    }
}

fn char_to_alphabet_index(ch: char) -> usize {
    let upper = ch.to_ascii_uppercase();
    if ('A'..='Z').contains(&upper) {
        ((upper as u8) - b'A') as usize
    } else {
        panic!("Not a latin letter {}", ch)
    }
}

impl TypeResolverImpl {
    pub fn new() -> Self {
        Self {
            ranges: [TypeQualifier::BangSingle; LETTER_COUNT],
        }
    }

    pub fn set(&mut self, def_type: &DefType) {
        let q: TypeQualifier = def_type.qualifier();
        for letter_range in def_type.ranges() {
            match *letter_range {
                LetterRange::Single(c) => self.fill_ranges(c, c, q),
                LetterRange::Range(start, stop) => self.fill_ranges(start, stop, q),
            }
        }
    }

    fn fill_ranges(&mut self, start: char, stop: char, qualifier: TypeQualifier) {
        let mut x: usize = char_to_alphabet_index(start);
        let y: usize = char_to_alphabet_index(stop);
        while x <= y {
            self.ranges[x] = qualifier;
            x += 1;
        }
    }
}

impl TypeResolver for TypeResolverImpl {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        let x = char_to_alphabet_index(ch);
        self.ranges[x]
    }
}
