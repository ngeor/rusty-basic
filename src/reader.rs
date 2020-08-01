use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Result};

#[derive(Debug)]
pub struct CharOrEofReader<T: BufRead> {
    reader: T,
    buffer: Vec<Option<char>>,
    seen_eof: bool,
}

impl<T: BufRead> CharOrEofReader<T> {
    pub fn new(reader: T) -> CharOrEofReader<T> {
        CharOrEofReader {
            reader,
            buffer: vec![],
            seen_eof: false
        }
    }

    pub fn peek(&mut self) -> Result<Option<char>> {
        self.fill_buffer_if_empty()?;
        Ok(self.buffer[0])
    }

    pub fn read(&mut self) -> Result<Option<char>> {
        self.fill_buffer_if_empty()?;
        Ok(self.buffer.remove(0))
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
        if bytes_read <= 0 {
            if self.seen_eof {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
            } else {
                self.buffer.push(None);
                self.seen_eof = true;
            }
        } else {
            for c in line.chars() {
                self.buffer.push(Some(c))
            }
            if self.buffer.is_empty() {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
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
        assert_eq!(reader.read().unwrap().unwrap(), '1');
        assert_eq!(reader.read().unwrap().unwrap(), '2');
        assert_eq!(reader.read().unwrap().unwrap(), '3');
        assert_eq!(reader.read().unwrap(), None);
        assert_eq!(reader.read().is_err(), true);
    }
}
