use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

use rusty_common::{HasPos, Position};
use rusty_pc::text::CharInput;

use super::row_col_view::*;

pub struct StringView {
    chars: Vec<char>,
    row_col: Vec<Position>,
}

impl StringView {
    pub fn eof_row_col(&self) -> Position {
        if self.row_col.is_empty() {
            Position::start()
        } else {
            let final_pos = self.row_col[self.row_col.len() - 1];
            final_pos.inc_col()
        }
    }
}

impl From<&str> for StringView {
    fn from(value: &str) -> Self {
        let chars: Vec<char> = value.chars().collect();
        let row_col = create_row_col_view(&chars);
        Self { chars, row_col }
    }
}

impl From<String> for StringView {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl TryFrom<File> for StringView {
    type Error = std::io::Error;

    fn try_from(value: File) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(value);
        let mut buf = String::new();
        loop {
            let bytes_read = reader.read_line(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
        }
        Ok(buf.into())
    }
}

/// A shared immutable view of the contents to be parsed.
/// Needs to be light to be cloned, as in some operations
/// we need to rollback to the previous state.
#[derive(Clone)]
pub struct RcStringView {
    buffer: Rc<StringView>,
    index: usize,
}

impl<S> From<S> for RcStringView
where
    S: Into<StringView>,
{
    fn from(value: S) -> Self {
        Self::new(value.into())
    }
}

impl TryFrom<File> for RcStringView {
    type Error = std::io::Error;

    fn try_from(value: File) -> Result<Self, Self::Error> {
        let string_view = StringView::try_from(value)?;
        Ok(string_view.into())
    }
}

impl RcStringView {
    pub fn new(buffer: StringView) -> Self {
        Self {
            buffer: Rc::new(buffer),
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.chars.len()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn char_at(&self, index: usize) -> char {
        self.buffer.chars[index]
    }

    pub fn position(&self) -> Position {
        if self.is_eof() {
            self.buffer.eof_row_col()
        } else {
            self.buffer.row_col[self.index]
        }
    }
}

impl CharInput for RcStringView {
    fn char(&self) -> char {
        self.char_at(self.index)
    }

    fn inc_position_by(self, amount: usize) -> Self {
        Self {
            index: self.index + amount,
            ..self
        }
    }

    fn is_eof(&self) -> bool {
        self.index >= self.len()
    }
}

impl HasPos for RcStringView {
    fn pos(&self) -> Position {
        self.position()
    }
}
