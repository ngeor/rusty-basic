use crate::parser::{DefType, HasQualifier, LetterRange, NameTrait, TypeQualifier, TypeResolver};

#[derive(Debug)]
pub struct TypeResolverImpl {
    ranges: [TypeQualifier; 26],
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
        panic!(format!("Not a latin letter {}", ch))
    }
}

impl TypeResolverImpl {
    pub fn new() -> Self {
        TypeResolverImpl {
            ranges: [TypeQualifier::BangSingle; 26],
        }
    }

    fn _set(&mut self, start: char, stop: char, qualifier: TypeQualifier) {
        let mut x: usize = char_to_alphabet_index(start);
        let y: usize = char_to_alphabet_index(stop);
        while x <= y {
            self.ranges[x] = qualifier;
            x += 1;
        }
    }

    pub fn set(&mut self, x: &DefType) {
        let q: TypeQualifier = x.qualifier();
        for r in x.ranges() {
            match *r {
                LetterRange::Single(c) => self._set(c, c, q),
                LetterRange::Range(start, stop) => self._set(start, stop, q),
            }
        }
    }
}

impl TypeResolver for TypeResolverImpl {
    fn resolve<T: NameTrait>(&self, name: &T) -> TypeQualifier {
        match name.opt_qualifier() {
            Some(q) => q,
            None => {
                let bare_name = name.bare_name();
                let x = char_to_alphabet_index(bare_name.first_char());
                self.ranges[x]
            }
        }
    }
}
