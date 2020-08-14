use crate::common::{PeekIterCopy, PeekOptCopy, ReadOpt};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Result};

/// Reads one character at a time out of a `BufRead`.
///
/// Returns a `Result<Option<char>>` where:
///
/// - `Ok(Some(char))` means we found a `char`
/// - `Ok(None)` means we hit EOF
/// - `Err(err)` means we encountered some IO error
#[derive(Debug)]
pub struct CharReader<T: BufRead> {
    reader: T,
    buffer: VecDeque<char>,
    read_eof: bool,
}

impl<T: BufRead> CharReader<T> {
    pub fn new(reader: T) -> CharReader<T> {
        CharReader {
            reader,
            buffer: VecDeque::new(),
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
                self.buffer.push_back(c);
            }
        }
        Ok(())
    }
}

impl<T: BufRead> ReadOpt for CharReader<T> {
    type Item = char;
    type Err = std::io::Error;

    fn read_ng(&mut self) -> Result<Option<char>> {
        if self.read_eof {
            Ok(None)
        } else {
            self.fill_buffer_if_empty()?;
            if self.buffer.is_empty() {
                self.read_eof = true;
                Ok(None)
            } else {
                Ok(self.buffer.pop_front())
            }
        }
    }
}

impl<T: BufRead> PeekOptCopy for CharReader<T> {
    fn peek_ng(&mut self) -> Result<Option<char>> {
        if self.read_eof {
            Ok(None)
        } else {
            self.fill_buffer_if_empty()?;
            if self.buffer.is_empty() {
                Ok(None)
            } else {
                Ok(Some(self.buffer[0]))
            }
        }
    }
}

impl<T: BufRead> Iterator for CharReader<T> {
    type Item = Result<char>;

    fn next(&mut self) -> Option<Result<char>> {
        self.read_ng().transpose()
    }
}

impl<T: BufRead> PeekIterCopy for CharReader<T> {
    fn peek_iter_ng(&mut self) -> Option<Result<char>> {
        self.peek_ng().transpose()
    }
}

// bytes || &str -> CharReader
impl<T> From<T> for CharReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        CharReader::new(BufReader::new(Cursor::new(input)))
    }
}

// File -> CharReader
impl From<File> for CharReader<BufReader<File>> {
    fn from(input: File) -> Self {
        CharReader::new(BufReader::new(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eof_is_twice() {
        let mut reader: CharReader<BufReader<Cursor<&str>>> = "123".into();
        assert_eq!(reader.read_ng().unwrap().unwrap(), '1');
        assert_eq!(reader.read_ng().unwrap().unwrap(), '2');
        assert_eq!(reader.read_ng().unwrap().unwrap(), '3');
        assert_eq!(reader.read_ng().unwrap(), None);
        assert_eq!(reader.read_ng().unwrap(), None);
    }

    #[test]
    fn test_iterator() {
        let input = "123";
        let reader = CharReader::from(input);
        let chars: Vec<char> = reader.map(|x| x.unwrap()).collect();
        assert_eq!(chars, vec!['1', '2', '3']);
    }
}
