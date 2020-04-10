use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Result};

#[derive(Debug)]
pub struct CharOrEofReader<T> {
    reader: T,
    _buffer: Vec<Option<char>>,
}

impl<T: BufRead> CharOrEofReader<T> {
    pub fn new(reader: T) -> CharOrEofReader<T> {
        CharOrEofReader {
            reader,
            _buffer: vec![],
        }
    }

    pub fn read(&mut self) -> Result<Option<char>> {
        self._fill_buffer_if_empty()?;
        Ok(self._buffer[0])
    }

    pub fn consume(&mut self) -> Option<char> {
        if self._buffer.is_empty() {
            panic!("Buffer underrun")
        } else {
            self._buffer.remove(0)
        }
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
        let bytes_read = self.reader.read_line(&mut line)?;
        if bytes_read <= 0 {
            self._buffer.push(None);
        } else {
            for c in line.chars() {
                self._buffer.push(Some(c))
            }
            if self._buffer.is_empty() {
                panic!("Should have found at least one character")
            }
        }
        Ok(())
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
