use crate::common::{HasLocation, Location, QError};
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

/// Reads one character at a time out of a `BufRead`.
///
/// Returns a `Result<Option<char>>` where:
///
/// - `Ok(Some(char))` means we found a `char`
/// - `Ok(None)` means we hit EOF
/// - `Err(err)` means we encountered some IO error
#[derive(Debug)]
struct CharReader<T: BufRead> {
    reader: T,
    buffer: VecDeque<char>,
    read_eof: bool,
}

impl<T: BufRead> Reader for CharReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> ReaderResult<Self, char, QError> {
        let Self {
            mut reader,
            mut buffer,
            mut read_eof,
        } = self;
        if buffer.is_empty() {
            if read_eof {
                Ok((
                    // TODO throw IO error EOF here?
                    Self {
                        reader,
                        buffer,
                        read_eof,
                    },
                    None,
                ))
            } else {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            for c in line.chars() {
                                buffer.push_back(c);
                            }
                            let ch = buffer.pop_front().unwrap();
                            Ok((
                                Self {
                                    reader,
                                    buffer,
                                    read_eof,
                                },
                                Some(ch),
                            ))
                        } else {
                            read_eof = true;
                            Ok((
                                Self {
                                    reader,
                                    buffer,
                                    read_eof,
                                },
                                None,
                            ))
                        }
                    }
                    Err(err) => Err((
                        Self {
                            reader,
                            buffer,
                            read_eof,
                        },
                        err.into(),
                    )),
                }
            }
        } else {
            let ch = buffer.pop_front().unwrap();
            Ok((
                Self {
                    reader,
                    buffer,
                    read_eof,
                },
                Some(ch),
            ))
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

impl<T: BufRead> CharReader<T> {
    fn new(reader: T) -> Self {
        Self {
            reader,
            buffer: VecDeque::new(),
            read_eof: false,
        }
    }
}

//
// EolReader
//

pub struct EolReader<T: BufRead> {
    char_reader: CharReader<T>,
    pos: Location,
    line_lengths: Vec<u32>,
}

// Location tracking + treating CRLF as one char
impl<T: BufRead> EolReader<T> {
    fn new(char_reader: CharReader<T>) -> Self {
        Self {
            char_reader,
            pos: Location::start(),
            line_lengths: vec![],
        }
    }
}

impl<T: BufRead + 'static> Reader for EolReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> ReaderResult<Self, char, QError> {
        let Self {
            char_reader,
            mut pos,
            mut line_lengths,
        } = self;
        let res = or(
            or(
                try_read('\n'),
                map(
                    // Tradeoff: CRLF becomes just CR
                    // Alternatives:
                    // - Return a String instead of a char
                    // - Return a new enum type instead of a char
                    // - Encode CRLF as a special char e.g. CR = 13 + LF = 10 -> CRLF = 23
                    opt_seq2(try_read('\r'), try_read('\n')),
                    |(cr, _)| cr,
                ),
            ),
            read(),
        )(char_reader);
        match res {
            Ok((char_reader, opt_res)) => {
                match &opt_res {
                    Some('\r') | Some('\n') => {
                        if line_lengths.len() + 1 == (pos.row() as usize) {
                            line_lengths.push(pos.col());
                        }
                        pos.inc_row();
                    }
                    Some(_) => {
                        pos.inc_col();
                    }
                    _ => {}
                }

                Ok((
                    Self {
                        char_reader,
                        pos,
                        line_lengths,
                    },
                    opt_res,
                ))
            }
            Err((char_reader, err)) => Err((
                Self {
                    char_reader,
                    pos,
                    line_lengths,
                },
                err,
            )),
        }
    }

    fn undo_item(self, x: char) -> Self {
        let Self {
            mut char_reader,
            mut pos,
            line_lengths,
        } = self;
        match x {
            '\r' | '\n' => {
                pos = Location::new(pos.row() - 1, line_lengths[(pos.row() - 2) as usize]);
                char_reader = char_reader.undo_item(x);
            }
            _ => {
                pos = Location::new(pos.row(), pos.col() - 1);
                char_reader = char_reader.undo_item(x);
            }
        }
        Self {
            char_reader,
            pos,
            line_lengths,
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
    fn test_eof_is_twice() {
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
}
