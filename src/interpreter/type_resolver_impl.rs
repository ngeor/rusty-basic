use crate::common::CaseInsensitiveString;
use crate::parser::{TypeQualifier, TypeResolver};

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

    pub fn set(&mut self, start: char, stop: char, qualifier: TypeQualifier) {
        let mut x: usize = char_to_alphabet_index(start);
        let y: usize = char_to_alphabet_index(stop);
        while x <= y {
            self.ranges[x] = qualifier;
            x += 1;
        }
    }
}

impl TypeResolver for TypeResolverImpl {
    fn resolve(&self, name: &CaseInsensitiveString) -> TypeQualifier {
        let x = char_to_alphabet_index(name.first_char());
        self.ranges[x]
    }
}
