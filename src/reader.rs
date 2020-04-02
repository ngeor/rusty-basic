use crate::common::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CharOrEof {
    EOF,

    Char(char),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct RowCol {
    row: u32,
    col: u32,
}

impl RowCol {
    pub fn new() -> RowCol {
        RowCol { row: 0, col: 0 }
    }

    pub fn row(&self) -> u32 {
        self.row
    }
    pub fn col(&self) -> u32 {
        self.col
    }
    pub fn inc_col(&mut self) {
        self.col += 1
    }
    pub fn inc_row(&mut self) {
        self.row += 1;
        self.col = 1;
    }
}

#[derive(Debug)]
pub struct CharOrEofReader<T> {
    reader: T,
    _buffer: Vec<CharOrEof>,
    _pos: RowCol,
}

impl<T: BufRead> CharOrEofReader<T> {
    pub fn new(reader: T) -> CharOrEofReader<T> {
        CharOrEofReader {
            reader,
            _buffer: vec![],
            _pos: RowCol::new(),
        }
    }

    pub fn read(&mut self) -> Result<CharOrEof> {
        self._fill_buffer_if_empty()?;
        Ok(self._buffer[0])
    }

    pub fn consume(&mut self) -> Result<CharOrEof> {
        if self._buffer.is_empty() {
            Err("Buffer underrun".to_string())
        } else {
            self._pos.inc_col();
            Ok(self._buffer.remove(0))
        }
    }

    pub fn read_and_consume(&mut self) -> Result<CharOrEof> {
        self._fill_buffer_if_empty()?;
        self._pos.inc_col();
        Ok(self._buffer.remove(0))
    }

    pub fn pos(&self) -> RowCol {
        self._pos
    }

    fn _fill_buffer_if_empty(&mut self) -> Result<()> {
        if self._buffer.is_empty() {
            self._fill_buffer()
        } else {
            Ok(())
        }
    }

    fn _fill_buffer(&mut self) -> Result<()> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(bytes_read) => {
                if bytes_read <= 0 {
                    self._buffer.push(CharOrEof::EOF);
                } else {
                    self._pos.inc_row();
                    for c in line.chars() {
                        self._buffer.push(CharOrEof::Char(c))
                    }
                    if self._buffer.is_empty() {
                        panic!("Should have found at least one character")
                    }
                }
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

// bytes || &str -> CharOrEofReader
impl<T> From<T> for CharOrEofReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        CharOrEofReader::new(BufReader::new(Cursor::new(input)))
    }
}

// File -> CharOrEofReader
impl From<File> for CharOrEofReader<BufReader<File>> {
    fn from(input: File) -> Self {
        CharOrEofReader::new(BufReader::new(input))
    }
}
