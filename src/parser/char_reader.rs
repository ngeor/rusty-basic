use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

use crate::common::{HasLocation, Location, QError};
use crate::parser::pc::{Reader, ReaderResult};

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
    fn new(reader: T) -> Self {
        Self {
            reader,
            buffer: VecDeque::new(),
            read_eof: false,
        }
    }

    fn ok_some(
        reader: T,
        buffer: VecDeque<char>,
        read_eof: bool,
        ch: char,
    ) -> ReaderResult<Self, char, QError> {
        Ok((
            Self {
                reader,
                buffer,
                read_eof,
            },
            Some(ch),
        ))
    }

    fn ok_none(reader: T, buffer: VecDeque<char>) -> ReaderResult<Self, char, QError> {
        Ok((
            Self {
                reader,
                buffer,
                read_eof: true,
            },
            None,
        ))
    }

    fn err(
        reader: T,
        buffer: VecDeque<char>,
        read_eof: bool,
        err: std::io::Error,
    ) -> ReaderResult<Self, char, QError> {
        Err((
            Self {
                reader,
                buffer,
                read_eof,
            },
            err.into(),
        ))
    }
}

impl<T: BufRead> Reader for CharReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> ReaderResult<Self, char, QError> {
        let Self {
            mut reader,
            mut buffer,
            read_eof,
        } = self;
        if buffer.is_empty() {
            if read_eof {
                // empty buffer and already seen EOF
                Self::ok_none(reader, buffer)
            } else {
                // fill buffer with data from reader
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            for c in line.chars() {
                                buffer.push_back(c);
                            }
                            let ch = buffer.pop_front().unwrap();
                            Self::ok_some(reader, buffer, read_eof, ch)
                        } else {
                            Self::ok_none(reader, buffer)
                        }
                    }
                    Err(err) => Self::err(reader, buffer, read_eof, err),
                }
            }
        } else {
            // return existing data from the buffer
            let ch = buffer.pop_front().unwrap();
            Self::ok_some(reader, buffer, read_eof, ch)
        }
    }

    fn undo_item(self, ch: char) -> Self {
        let Self {
            reader,
            mut buffer,
            read_eof,
        } = self;
        buffer.push_front(ch);
        Self {
            reader,
            buffer,
            read_eof,
        }
    }
}

//
// EolReader
//

#[derive(Debug)]
struct Row {
    column_count: u32,
    ends_with_cr: bool,
}

#[derive(Debug)]
pub struct EolReader<T: BufRead> {
    char_reader: CharReader<T>,
    pos: Location,
    rows: Vec<Row>,
}

// Location tracking + treating CRLF as one char
impl<T: BufRead> EolReader<T> {
    fn new(char_reader: CharReader<T>) -> Self {
        Self {
            char_reader,
            pos: Location::start(),
            rows: vec![],
        }
    }

    fn ok_some(
        char_reader: CharReader<T>,
        pos: Location,
        rows: Vec<Row>,
        ch: char,
    ) -> ReaderResult<Self, char, QError> {
        Ok((
            Self {
                char_reader,
                pos,
                rows,
            },
            Some(ch),
        ))
    }
}

impl<T: BufRead> Reader for EolReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> ReaderResult<Self, char, QError> {
        let Self {
            char_reader,
            mut pos,
            mut rows,
        } = self;

        match char_reader.read() {
            Ok((char_reader, Some('\r'))) => {
                let is_new_row = rows.len() + 1 == (pos.row() as usize);
                if is_new_row {
                    rows.push(Row {
                        column_count: pos.col(),
                        ends_with_cr: true,
                    });
                }
                pos.inc_row();
                Self::ok_some(char_reader, pos, rows, '\r')
            }
            Ok((char_reader, Some('\n'))) => {
                let last_row_ended_with_cr = pos.col() == 1
                    && pos.row() >= 2
                    && (pos.row() as usize) - 2 < rows.len()
                    && rows[(pos.row() as usize) - 2].ends_with_cr;
                if !last_row_ended_with_cr {
                    let is_new_row = rows.len() + 1 == (pos.row() as usize);
                    if is_new_row {
                        rows.push(Row {
                            column_count: pos.col(),
                            ends_with_cr: false,
                        });
                    }
                    pos.inc_row();
                }
                Self::ok_some(char_reader, pos, rows, '\n')
            }
            Ok((char_reader, Some(ch))) => {
                pos.inc_col();
                Self::ok_some(char_reader, pos, rows, ch)
            }
            Ok((char_reader, None)) => Ok((
                Self {
                    char_reader,
                    pos,
                    rows,
                },
                None,
            )),
            Err((char_reader, err)) => Err((
                Self {
                    char_reader,
                    pos,
                    rows,
                },
                err,
            )),
        }
    }

    fn undo_item(self, x: char) -> Self {
        let Self {
            mut char_reader,
            mut pos,
            rows,
        } = self;
        char_reader = char_reader.undo_item(x);
        match x {
            '\r' => {
                pos = Location::new(pos.row() - 1, rows[(pos.row() - 2) as usize].column_count);
            }
            '\n' => {
                let last_row: &Row = &rows[(pos.row() - 2) as usize];
                if last_row.ends_with_cr {
                } else {
                    pos = Location::new(pos.row() - 1, last_row.column_count);
                }
            }
            _ => {
                pos = Location::new(pos.row(), pos.col() - 1);
            }
        }
        Self {
            char_reader,
            pos,
            rows,
        }
    }
}

impl<T: BufRead> HasLocation for EolReader<T> {
    fn pos(&self) -> Location {
        self.pos
    }
}

// ========================================================
// Converters from str and File
// ========================================================

// bytes || &str -> CharReader
impl<T> From<T> for CharReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        CharReader::new(BufReader::new(Cursor::new(input)))
    }
}

// bytes || &str -> EolReader
impl<T> From<T> for EolReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        EolReader::new(CharReader::new(BufReader::new(Cursor::new(input))))
    }
}

// File -> EolReader
impl From<File> for EolReader<BufReader<File>> {
    fn from(input: File) -> Self {
        EolReader::new(CharReader::new(BufReader::new(input)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_reader_eof_is_twice() {
        let reader: CharReader<BufReader<Cursor<&str>>> = "123".into();
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '1');
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '2');
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '3');
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
        let (_, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
    }

    #[test]
    fn test_eol_cr_only() {
        let input = "ab\rc\r";
        let reader: EolReader<BufReader<Cursor<&str>>> = input.into();
        assert_eq!(reader.pos(), Location::start());

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'a');
        assert_eq!(reader.pos(), Location::new(1, 2));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'b');
        assert_eq!(reader.pos(), Location::new(1, 3));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'c');
        assert_eq!(reader.pos(), Location::new(2, 2));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(reader.pos(), Location::new(3, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
        assert_eq!(reader.pos(), Location::new(3, 1));

        let reader = reader.undo_item('\r');
        assert_eq!(reader.pos(), Location::new(2, 2));

        let reader = reader.undo_item('c');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let reader = reader.undo_item('\r');
        assert_eq!(reader.pos(), Location::new(1, 3));

        let reader = reader.undo_item('b');
        assert_eq!(reader.pos(), Location::new(1, 2));

        let reader = reader.undo_item('a');
        assert_eq!(reader.pos(), Location::new(1, 1));
    }

    #[test]
    fn test_eol_lf_only() {
        let input = "ab\nc\n";
        let reader: EolReader<BufReader<Cursor<&str>>> = input.into();
        assert_eq!(reader.pos(), Location::start());

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'a');
        assert_eq!(reader.pos(), Location::new(1, 2));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'b');
        assert_eq!(reader.pos(), Location::new(1, 3));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\n');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'c');
        assert_eq!(reader.pos(), Location::new(2, 2));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\n');
        assert_eq!(reader.pos(), Location::new(3, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
        assert_eq!(reader.pos(), Location::new(3, 1));

        let reader = reader.undo_item('\n');
        assert_eq!(reader.pos(), Location::new(2, 2));

        let reader = reader.undo_item('c');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let reader = reader.undo_item('\n');
        assert_eq!(reader.pos(), Location::new(1, 3));

        let reader = reader.undo_item('b');
        assert_eq!(reader.pos(), Location::new(1, 2));

        let reader = reader.undo_item('a');
        assert_eq!(reader.pos(), Location::new(1, 1));
    }

    #[test]
    fn test_eol_cr_lf() {
        let input = "ab\r\nc\r\n";
        let reader: EolReader<BufReader<Cursor<&str>>> = input.into();
        assert_eq!(reader.pos(), Location::start());

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'a');
        assert_eq!(reader.pos(), Location::new(1, 2));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'b');
        assert_eq!(reader.pos(), Location::new(1, 3));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\n');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), 'c');
        assert_eq!(reader.pos(), Location::new(2, 2));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(reader.pos(), Location::new(3, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '\n');
        assert_eq!(reader.pos(), Location::new(3, 1));

        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
        assert_eq!(reader.pos(), Location::new(3, 1));

        let reader = reader.undo_item('\n');
        assert_eq!(reader.pos(), Location::new(3, 1));

        let reader = reader.undo_item('\r');
        assert_eq!(reader.pos(), Location::new(2, 2));

        let reader = reader.undo_item('c');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let reader = reader.undo_item('\n');
        assert_eq!(reader.pos(), Location::new(2, 1));

        let reader = reader.undo_item('\r');
        assert_eq!(reader.pos(), Location::new(1, 3));

        let reader = reader.undo_item('b');
        assert_eq!(reader.pos(), Location::new(1, 2));

        let reader = reader.undo_item('a');
        assert_eq!(reader.pos(), Location::new(1, 1));
    }
}
