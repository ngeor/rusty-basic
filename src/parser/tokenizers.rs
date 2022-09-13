use crate::parser::pc::{Reader, ReaderResult};
use super::recognizers::{Recognizer, Recognition};

struct RowCol {
    row: u32,
    col: u32
}

struct Position {
    begin: RowCol,
    end: RowCol
}

struct Token {
    text: String,
    kind: u32,
    position: Position
}

struct Tokenizer<R: Reader<Item = char, Err = E>, E> {
    reader: Option<R>,
    recognizers: Vec<Box<dyn Recognizer>>,
    pos: RowCol
}

impl<R: Reader<Item = char, Err = E>, E> Tokenizer<R, E> {
    pub fn new(reader: R, recognizers: Vec<Box<dyn Recognizer>>) -> Self {
        Self {
            reader: Some(reader),
            recognizers,
            pos: RowCol {
                row: 1,
                col: 1
            }
        }
    }

    pub fn read(&mut self) -> Result<Option<Token>, E> {
        let mut buffer = String::new();
        let mut no_match_or_eof = false;
        while !no_match_or_eof {
            match self.read_char()? {
                Some(ch) => {
                    buffer.push(ch);
                    for recognizer in &self.recognizers {
                        let recognition = recognizer.recognize(&buffer);
                    }
                    todo!()
                }
                None => {
                    no_match_or_eof = true;
                }
            }
        }
        todo!()
    }

    fn read_char(&mut self) -> Result<Option<char>, E> {
        let reader = self.reader.take().unwrap();
        match reader.read() {
            Ok((reader, Some(ch))) => {
                self.reader = Some(reader);
                Ok(Some(ch))
            }
            Ok((reader, None)) => {
                self.reader = Some(reader);
                Ok(None)
            }
            Err((reader, err)) => {
                self.reader = Some(reader);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};
    use crate::parser::char_reader::CharReader;
    use crate::parser::recognizers::{digits_recognizer, letters_recognizer};
    use super::*;

    #[test]
    fn test_digits() {
        let input = "1234";
        let reader: CharReader<BufReader<Cursor<&str>>> = input.into();
        let mut tokenizer = Tokenizer::new(
            reader,
            vec![Box::new(digits_recognizer())]
        );
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 0);
        assert_eq!(token.position.begin.row, 1);
        assert_eq!(token.position.begin.col, 1);
        assert_eq!(token.position.end.row, 1);
        assert_eq!(token.position.end.col, 4);
    }

    #[test]
    fn test_letters_digits() {
        let input = "abc1234";
        let reader: CharReader<BufReader<Cursor<&str>>> = input.into();
        let mut tokenizer = Tokenizer::new(
            reader,
            vec![
                Box::new(letters_recognizer()),
                Box::new(digits_recognizer())
            ]
        );
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "abc");
        assert_eq!(token.kind, 0);
        assert_eq!(token.position.begin.row, 1);
        assert_eq!(token.position.begin.col, 1);
        assert_eq!(token.position.end.row, 1);
        assert_eq!(token.position.end.col, 4);
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 1);
        assert_eq!(token.position.begin.row, 1);
        assert_eq!(token.position.begin.col, 1);
        assert_eq!(token.position.end.row, 1);
        assert_eq!(token.position.end.col, 4);
    }
}
