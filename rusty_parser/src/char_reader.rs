use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

/// Reads one character at a time.
///
/// Returns a `Result<Option<char>>` where:
///
/// - `Ok(Some(char))` means we found a `char`
/// - `Ok(None)` means we hit EOF
/// - `Err(err)` means we encountered some IO error
pub trait CharReader {
    fn read(&mut self) -> std::io::Result<Option<char>>;
    fn unread(&mut self, item: char);
}

pub fn file_char_reader(input: File) -> impl CharReader {
    CharReaderImpl::new(BufReader::new(input))
}

pub fn string_char_reader(input: &str) -> impl CharReader + '_ {
    CharReaderImpl::new(BufReader::new(Cursor::new(input)))
}

struct CharReaderImpl<T: BufRead> {
    buf_read: T,
    buffer: VecDeque<char>,
}

impl<T: BufRead> CharReaderImpl<T> {
    fn new(buf_read: T) -> Self {
        Self {
            buf_read,
            buffer: VecDeque::new(),
        }
    }
}

impl<T: BufRead> CharReader for CharReaderImpl<T> {
    fn read(&mut self) -> std::io::Result<Option<char>> {
        match self.buffer.pop_front() {
            Some(ch) => Ok(Some(ch)),
            None => {
                let mut line = String::new();
                let bytes_read = self.buf_read.read_line(&mut line)?;
                if bytes_read > 0 {
                    while let Some(ch) = line.pop() {
                        self.buffer.push_front(ch);
                    }
                    Ok(self.buffer.pop_front())
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn unread(&mut self, item: char) {
        self.buffer.push_front(item)
    }
}

#[cfg(test)]
mod tests {
    use crate::char_reader::{string_char_reader, CharReader};

    #[test]
    fn test() {
        let mut input = string_char_reader("hello");
        assert_eq!(input.read().unwrap().unwrap(), 'h');
        assert_eq!(input.read().unwrap().unwrap(), 'e');
        assert_eq!(input.read().unwrap().unwrap(), 'l');
        assert_eq!(input.read().unwrap().unwrap(), 'l');
        assert_eq!(input.read().unwrap().unwrap(), 'o');
    }
}
