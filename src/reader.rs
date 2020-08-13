use crate::common::{PeekOne, ReadOne};
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Result};

#[derive(Debug)]
pub struct CharOrEofReader<T: BufRead> {
    reader: T,
    buffer: Vec<char>,
    read_eof: bool,
}

impl<T: BufRead> ReadOne for CharOrEofReader<T> {
    type Item = char;
    type Err = std::io::Error;

    fn read_ng(&mut self) -> Result<Option<char>> {
        if self.read_eof {
            Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
        } else {
            self.fill_buffer_if_empty()?;
            if self.buffer.is_empty() {
                self.read_eof = true;
                Ok(None)
            } else {
                Ok(Some(self.buffer.remove(0)))
            }
        }
    }
}

impl<T: BufRead> PeekOne for CharOrEofReader<T> {
    // TODO make a new trait where peek_ng can return the real thing for Copy types
    fn peek_ng(&mut self) -> Result<Option<&char>> {
        if self.read_eof {
            Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
        } else {
            self.fill_buffer_if_empty()?;
            if self.buffer.is_empty() {
                Ok(None)
            } else {
                Ok(Some(&self.buffer[0]))
            }
        }
    }
}

impl<T: BufRead> CharOrEofReader<T> {
    pub fn new(reader: T) -> CharOrEofReader<T> {
        CharOrEofReader {
            reader,
            buffer: vec![],
            read_eof: false,
        }
    }

    fn fill_buffer_if_empty(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            self.fill_buffer()
        } else {
            Ok(())
        }
    }

    fn fill_buffer(&mut self) -> Result<()> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;
        if bytes_read > 0 {
            for c in line.chars() {
                self.buffer.push(c)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eof_is_only_once() {
        let mut reader: CharOrEofReader<BufReader<Cursor<&str>>> = "123".into();
        assert_eq!(reader.read_ng().unwrap().unwrap(), '1');
        assert_eq!(reader.read_ng().unwrap().unwrap(), '2');
        assert_eq!(reader.read_ng().unwrap().unwrap(), '3');
        assert_eq!(reader.read_ng().unwrap(), None);
        assert_eq!(reader.read_ng().is_err(), true);
    }
}
