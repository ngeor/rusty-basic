use crate::interpreter::io::Input;
use std::cell::RefCell;
use std::io::{ErrorKind, Read};

pub struct ReadInputSource<T: Read> {
    read: RefCell<T>,
    buffer: RefCell<Vec<u8>>,
}

impl<T: Read> ReadInputSource<T> {
    pub fn new(read: T) -> Self {
        Self {
            read: RefCell::new(read),
            buffer: RefCell::new(vec![]),
        }
    }

    #[cfg(test)]
    pub fn inner(&mut self) -> &mut T {
        self.read.get_mut()
    }

    fn peek(&self) -> std::io::Result<Option<u8>> {
        if self.fill_buffer()? == 0 {
            Ok(None)
        } else {
            Ok(self.buffer.borrow().get(0).map(|x| *x))
        }
    }

    fn read(&self) -> std::io::Result<Option<u8>> {
        if self.fill_buffer()? == 0 {
            Ok(None)
        } else {
            Ok(Some(self.buffer.borrow_mut().remove(0)))
        }
    }

    fn fill_buffer(&self) -> std::io::Result<usize> {
        if self.buffer.borrow().is_empty() {
            let mut buf = [0; 1]; // 1 bytes buffer inefficient
            let n = self.read.borrow_mut().read(&mut buf[..])?;
            if n > 0 {
                self.buffer.borrow_mut().push(buf[0]);
            }

            Ok(n)
        } else {
            Ok(self.buffer.borrow().len())
        }
    }

    fn skip_while<F>(&self, predicate: F) -> std::io::Result<String>
    where
        F: Fn(char) -> bool,
    {
        let mut buf: Vec<u8> = vec![];
        let mut found = true;
        while found {
            found = false;
            if let Some(ch) = self.peek()? {
                if predicate(ch as char) {
                    buf.push(ch);
                    self.read()?;
                    found = true;
                }
            }
        }
        Ok(String::from_utf8(buf).unwrap())
    }

    fn read_until<F>(&self, predicate: F) -> std::io::Result<String>
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
                        if let Some(next_ch) = self.peek()? {
                            if next_ch as char == '\n' {
                                self.read()?;
                            }
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
    fn eof(&self) -> std::io::Result<bool> {
        self.peek().map(|ch| ch.is_none())
    }

    fn input(&self) -> std::io::Result<String> {
        if self.eof()? {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }

        // skip leading whitespace
        self.skip_while(|ch| ch == ' ')?;
        // read until comma or eol
        self.read_until(|ch| ch == ',' || ch == '\r' || ch == '\n')
            .map(|s| s.trim().to_owned())
    }

    fn line_input(&self) -> std::io::Result<String> {
        if self.eof()? {
            return Err(std::io::Error::from(ErrorKind::UnexpectedEof));
        }
        self.read_until(|ch| ch == '\r' || ch == '\n')
    }
}
