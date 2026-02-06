use std::fs::File;
use std::io::{BufRead, BufReader};

use rusty_common::{HasPos, Position};
use rusty_pc::InputTrait;

use super::row_col_view::*;

pub struct StringView {
    chars: Vec<char>,
    row_col: Vec<Position>,
    index: usize,
}

impl From<&str> for StringView {
    fn from(value: &str) -> Self {
        let chars: Vec<char> = value.chars().collect();
        let row_col = create_row_col_view(&chars);
        Self {
            chars,
            row_col,
            index: 0,
        }
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

impl StringView {
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn char_at(&self, index: usize) -> char {
        self.chars[index]
    }

    pub fn position(&self) -> Position {
        if self.is_eof() {
            self.eof_row_col()
        } else {
            self.row_col[self.index]
        }
    }

    fn eof_row_col(&self) -> Position {
        if self.row_col.is_empty() {
            Position::start()
        } else {
            let final_pos = self.row_col[self.row_col.len() - 1];
            final_pos.inc_col()
        }
    }
}

impl InputTrait for StringView {
    type Output = char;

    fn get_position(&self) -> usize {
        self.index
    }

    fn set_position(&mut self, position: usize) {
        self.index = position;
    }

    fn peek(&self) -> char {
        self.char_at(self.index)
    }

    fn read(&mut self) -> char {
        let c = self.char_at(self.index);
        self.index += 1;
        c
    }

    fn is_eof(&self) -> bool {
        self.index >= self.len()
    }
}

impl HasPos for StringView {
    fn pos(&self) -> Position {
        self.position()
    }
}
