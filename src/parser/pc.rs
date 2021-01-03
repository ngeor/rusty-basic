// ========================================================
// types
// ========================================================

// the R is needed in the error in order to be able to get error location

pub type ReaderResult<R, T, E> = Result<(R, Option<T>), (R, E)>;

// ========================================================
// traits
// ========================================================

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

pub trait Reader: Sized {
    type Item;
    type Err;
    fn read(self) -> ReaderResult<Self, Self::Item, Self::Err>;
    fn undo_item(self, item: Self::Item) -> Self;
}

// ========================================================
// Undo support
// ========================================================

pub mod undo {
    use super::{Reader, Undo};
    use crate::common::Locatable;

    impl<R: Reader<Item = char>> Undo<char> for R {
        fn undo(self, item: char) -> Self {
            self.undo_item(item)
        }
    }

    impl<R: Reader<Item = char>> Undo<Locatable<char>> for R {
        fn undo(self, item: Locatable<char>) -> Self {
            self.undo_item(item.element)
        }
    }

    impl<R: Reader<Item = char>> Undo<String> for R {
        fn undo(self, s: String) -> Self {
            let mut result = self;
            for ch in s.chars().rev() {
                result = result.undo_item(ch);
            }
            result
        }
    }

    impl<R: Reader<Item = char>> Undo<(String, Locatable<char>)> for R {
        fn undo(self, item: (String, Locatable<char>)) -> Self {
            let (a, b) = item;
            self.undo(b).undo(a)
        }
    }

    // undo char followed by opt ws
    impl<R: Reader<Item = char>> Undo<(char, Option<String>)> for R {
        fn undo(self, item: (char, Option<String>)) -> Self {
            let (a, b) = item;
            self.undo(b.unwrap_or_default()).undo_item(a)
        }
    }

    // undo char preceded by opt ws
    impl<B, R: Reader<Item = char> + Undo<String> + Undo<B>> Undo<(Option<String>, B)> for R {
        fn undo(self, item: (Option<String>, B)) -> Self {
            let (a, b) = item;
            self.undo(b).undo(a.unwrap_or_default())
        }
    }
}

pub fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

pub fn is_eol(ch: char) -> bool {
    ch == '\r' || ch == '\n'
}

pub fn is_eol_or_whitespace(ch: char) -> bool {
    is_eol(ch) || is_whitespace(ch)
}
