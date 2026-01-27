use std::io::{ErrorKind, Read};

use crate::interpreter::io::Input;
use crate::interpreter::is_cr_lf;

pub struct ReadInputSource<T: Read> {
    read: T,
    buffer: Vec<u8>,
}

impl<T: Read> ReadInputSource<T> {
    pub fn new(read: T) -> Self {
        Self {
            read,
            buffer: vec![],
        }
    }

    #[cfg(test)]
    pub fn inner(&mut self) -> &mut T {
        &mut self.read
    }

    fn peek(&mut self) -> std::io::Result<Option<u8>> {
        if self.fill_buffer()? == 0 {
            Ok(None)
        } else {
            Ok(self.buffer.first().copied())
        }
    }

    fn read(&mut self) -> std::io::Result<Option<u8>> {
        if self.fill_buffer()? == 0 {
            Ok(None)
        } else {
            Ok(Some(self.buffer.remove(0)))
        }
    }

    fn fill_buffer(&mut self) -> std::io::Result<usize> {
        if self.buffer.is_empty() {
            let mut buf = [0; 1]; // 1 bytes buffer inefficient
            let n = self.read.read(&mut buf[..])?;
            if n > 0 {
                self.buffer.push(buf[0]);
            }

            Ok(n)
        } else {
            Ok(self.buffer.len())
        }
    }

    fn skip_while<F>(&mut self, predicate: F) -> std::io::Result<String>
    where
        F: Fn(char) -> bool,
    {
        let mut buf: Vec<u8> = vec![];
        let mut found = true;
        while found {
            found = false;
            if let Some(ch) = self.peek()?
                && predicate(ch as char)
            {
                buf.push(ch);
                self.read()?;
                found = true;
            }
        }
        Ok(String::from_utf8(buf).unwrap())
    }

    fn read_until<F>(&mut self, predicate: F) -> std::io::Result<String>
    where
        F: Fn(char) -> bool,
    {
        let mut buf: Vec<u8> = vec![];
        let mut found = true;
        while found {
            found = false;
            if let Some(ch) = self.read()? {
                if predicate(ch as char) {
                    // if it was '\r', try to also get the next '\n', if exists
                    if ch as char == '\r' {
                        if let Some(next_ch) = self.peek()?
                            && next_ch as char == '\n'
                        {
                            self.read()?;
                        }
                    }
                } else {
                    buf.push(ch);
                    found = true;
                }
            }
        }
        Ok(String::from_utf8(buf).unwrap())
    }
}

impl<T: Read> Input for ReadInputSource<T> {
    fn eof(&mut self) -> std::io::Result<bool> {
        self.peek().map(|ch| ch.is_none())
    }

    fn input(&mut self) -> std::io::Result<String> {
        if self.eof()? {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }

        // skip leading whitespace
        self.skip_while(|ch| ch == ' ')?;
        // read until comma or eol
        self.read_until(|ch| ch == ',' || is_cr_lf(ch))
            .map(|s| s.trim().to_owned())
    }

    fn line_input(&mut self) -> std::io::Result<String> {
        if self.eof()? {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }
        self.read_until(is_cr_lf)
    }
}
