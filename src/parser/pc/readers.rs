use std::collections::VecDeque;
use std::fs::File;
#[cfg(test)]
use std::io::Cursor;
use std::io::{BufRead, BufReader};

pub trait CharReader {
    fn read(&mut self) -> std::io::Result<Option<char>>;
    fn unread(&mut self, item: char);
}

#[cfg(test)]
pub fn string_char_reader<T>(input: T) -> impl CharReader
where
    T: AsRef<[u8]>,
{
    CharReaderImpl::new(BufReader::new(Cursor::new(input)))
}

pub fn file_char_reader(input: File) -> impl CharReader {
    CharReaderImpl::new(BufReader::new(input))
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
                    loop {
                        match line.pop() {
                            Some(ch) => self.buffer.push_front(ch),
                            None => break,
                        }
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
