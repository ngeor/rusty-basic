use crate::linter::type_resolver::TypeResolver;
use crate::parser::{DefType, LetterRange, TypeQualifier};

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
    if upper >= 'A' && upper <= 'Z' {
        ((upper as u8) - ('A' as u8)) as usize
    } else {
        panic!("Not a latin letter {}", ch)
    }
}

impl TypeResolverImpl {
    pub fn new() -> Self {
        TypeResolverImpl {
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
    fn resolve_char(&self, ch: char) -> TypeQualifier {
        let x = char_to_alphabet_index(ch);
        self.ranges[x]
    }
}
