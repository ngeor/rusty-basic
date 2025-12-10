use super::row_col_view::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    rc::Rc,
};

pub struct StringView {
    chars: Vec<char>,
    row_col: Vec<RowCol>,
}

impl From<&str> for StringView {
    fn from(value: &str) -> Self {
        let chars: Vec<char> = value.chars().collect();
        let row_col = create_row_col_view(&chars);
        StringView { chars, row_col }
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
    position: usize,
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
            position: 0,
        }
    }

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
