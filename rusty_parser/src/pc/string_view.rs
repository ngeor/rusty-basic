use super::row_col_view::*;
use std::rc::Rc;

pub struct StringView {
    chars: Vec<char>,
    row_col: Vec<RowCol>,
}

pub fn create_string_view(s: &str) -> StringView {
    let chars: Vec<char> = s.chars().collect();
    let row_col = create_row_col_view(&chars);
    StringView { chars, row_col }
}

/// A shared immutable view of the contents to be parsed.
/// Needs to be light to be cloned, as in some operations
/// we need to rollback to the previous state.
#[derive(Clone)]
pub struct RcStringView {
    buffer: Rc<StringView>,
    position: usize,
}

pub fn create_rc_string_view(s: &str) -> RcStringView {
    RcStringView {
        buffer: Rc::new(create_string_view(s)),
        position: 0,
    }
}

impl RcStringView {
    pub fn len(&self) -> usize {
        self.buffer.chars.len()
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn char_at(&self, index: usize) -> char {
        self.buffer.chars[index]
    }

    pub fn char(&self) -> char {
        self.char_at(self.position)
    }

    pub fn inc_position(self) -> Self {
        Self {
            buffer: self.buffer,
            position: self.position + 1,
        }
    }
}
