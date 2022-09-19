use std::fmt::{Display, Formatter};
use std::iter;

use super::readers::CharReader;
use super::recognizers::{Recognition, Recognizer};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowCol {
    pub row: u32,
    pub col: u32,
}

impl RowCol {
    pub fn new() -> Self {
        Self { row: 1, col: 1 }
    }

    pub fn inc_row(self) -> Self {
        Self {
            row: self.row + 1,
            col: 1,
        }
    }

    pub fn inc_col(self) -> Self {
        Self {
            row: self.row,
            col: self.col + 1,
        }
    }
}

impl Display for RowCol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

pub struct Position {
    pub begin: RowCol,
    pub end: RowCol,
}

pub struct Token {
    // TODO support enum type
    pub kind: i32,
    pub text: String,
    pub position: Position,
}

pub type TokenList = Vec<Token>;

pub fn token_list_to_string(list: &[Token]) -> String {
    let mut result = String::new();
    for token in list {
        result.push_str(&token.text);
    }
    result
}

pub trait Tokenizer {
    fn read(&mut self) -> std::io::Result<Option<Token>>;
    fn unread(&mut self, token: Token);
    fn position(&self) -> RowCol;
}

pub fn create_tokenizer<R: CharReader>(
    reader: R,
    recognizers: Vec<Box<dyn Recognizer>>,
) -> impl Tokenizer {
    UndoTokenizerImpl::new(TokenizerImpl::new(reader, recognizers))
}

struct TokenizerImpl<R: CharReader> {
    reader: R,
    recognizers: Vec<Box<dyn Recognizer>>,
    pos: RowCol,
}

struct UndoTokenizerImpl<R: CharReader> {
    tokenizer: TokenizerImpl<R>,
    buffer: TokenList,
}

struct RecognizerResponses {
    responses: Vec<Recognition>,
}

impl RecognizerResponses {
    pub fn new(length: usize) -> Self {
        let responses: Vec<Recognition> = iter::repeat(Recognition::Partial).take(length).collect();
        Self { responses }
    }

    pub fn get_last_response(&self, index: usize) -> Recognition {
        self.responses[index]
    }

    pub fn set_last_response(&mut self, index: usize, value: Recognition) {
        self.responses[index] = value;
    }

    pub fn count_by_recognition(&self, value: Recognition) -> usize {
        self.responses.iter().filter(|r| **r == value).count()
    }
}

impl<R: CharReader> TokenizerImpl<R> {
    pub fn new(reader: R, recognizers: Vec<Box<dyn Recognizer>>) -> Self {
        Self {
            reader,
            recognizers,
            pos: RowCol::new(),
        }
    }

    pub fn read(&mut self) -> std::io::Result<Option<Token>> {
        let mut buffer = String::new();
        let mut no_match_or_eof = false;
        let mut recognizer_responses = RecognizerResponses::new(self.recognizers.len());
        let mut sizes: Vec<usize> = iter::repeat(0).take(self.recognizers.len()).collect();
        while !no_match_or_eof {
            match self.reader.read()? {
                Some(ch) => {
                    buffer.push(ch);
                    let mut i: usize = 0;
                    for recognizer in &self.recognizers {
                        let last_response = recognizer_responses.get_last_response(i);
                        if last_response != Recognition::Negative {
                            let recognition = recognizer.recognize(&buffer);
                            recognizer_responses.set_last_response(i, recognition);
                            if recognition == Recognition::Positive {
                                // remember the buffer size at this point
                                sizes[i] = buffer.len();
                            } else if recognition == Recognition::Negative
                                && last_response == Recognition::Partial
                            {
                                // this recognizer never met its goal
                                sizes[i] = 0;
                            }
                        }
                        i += 1;
                    }
                    no_match_or_eof = recognizer_responses
                        .count_by_recognition(Recognition::Negative)
                        == self.recognizers.len();
                }
                None => {
                    no_match_or_eof = true;
                }
            }
        }

        // find the longest win
        let mut max_positive_size: usize = 0;
        let mut max_positive_index: i32 = -1;
        for i in 0..self.recognizers.len() {
            if sizes[i] > max_positive_size {
                max_positive_size = sizes[i];
                max_positive_index = i as i32;
            }
        }

        // unread extra characters
        while buffer.len() > max_positive_size {
            let last_char = buffer.pop().unwrap();
            self.reader.unread(last_char);
        }

        if max_positive_index >= 0 {
            let begin: RowCol = self.pos;
            let mut previous_char: char = ' ';
            for ch in buffer.chars() {
                if ch == '\r' {
                    self.pos = self.pos.inc_row();
                } else if ch == '\n' {
                    if previous_char == '\r' {
                        // already increased row for '\r'
                    } else {
                        self.pos = self.pos.inc_row();
                    }
                } else {
                    self.pos = self.pos.inc_col();
                }
                previous_char = ch;
            }
            Ok(Some(Token {
                kind: max_positive_index,
                text: buffer,
                position: Position {
                    begin,
                    end: self.pos,
                },
            }))
        } else {
            Ok(None)
        }
    }
}

impl<R: CharReader> UndoTokenizerImpl<R> {
    pub fn new(tokenizer: TokenizerImpl<R>) -> Self {
        Self {
            tokenizer,
            buffer: vec![],
        }
    }
}

impl<R: CharReader> Tokenizer for UndoTokenizerImpl<R> {
    fn read(&mut self) -> std::io::Result<Option<Token>> {
        match self.buffer.pop() {
            Some(token) => Ok(Some(token)),
            None => self.tokenizer.read(),
        }
    }

    fn unread(&mut self, token: Token) {
        self.buffer.push(token)
    }

    fn position(&self) -> RowCol {
        match self.buffer.last() {
            Some(token) => token.position.begin,
            _ => self.tokenizer.pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::readers::string_char_reader;
    use super::super::recognizers::{many_digits_recognizer, many_letters_recognizer};
    use super::{Tokenizer, TokenizerImpl, UndoTokenizerImpl};

    #[test]
    fn test_digits() {
        let input = "1234";
        let reader = string_char_reader(input);
        let mut tokenizer = TokenizerImpl::new(reader, vec![Box::new(many_digits_recognizer())]);
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 0);
        assert_eq!(token.position.begin.row, 1);
        assert_eq!(token.position.begin.col, 1);
        assert_eq!(token.position.end.row, 1);
        assert_eq!(token.position.end.col, 5);
    }

    #[test]
    fn test_letters_digits() {
        let input = "abc1234";
        let reader = string_char_reader(input);
        let mut tokenizer = TokenizerImpl::new(
            reader,
            vec![
                Box::new(many_letters_recognizer()),
                Box::new(many_digits_recognizer()),
            ],
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
        assert_eq!(token.position.begin.col, 4);
        assert_eq!(token.position.end.row, 1);
        assert_eq!(token.position.end.col, 8);
    }

    #[test]
    fn test_undo() {
        let input = "a1b2c3";
        let reader = string_char_reader(input);
        let mut tokenizer = UndoTokenizerImpl::new(TokenizerImpl::new(
            reader,
            vec![
                Box::new(many_letters_recognizer()),
                Box::new(many_digits_recognizer()),
            ],
        ));

        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "a");

        tokenizer.unread(token);

        let token1 = tokenizer.read().unwrap().unwrap();
        assert_eq!(token1.text, "a");

        let token2 = tokenizer.read().unwrap().unwrap();
        assert_eq!(token2.text, "1");

        tokenizer.unread(token2);
        tokenizer.unread(token1);

        let token1 = tokenizer.read().unwrap().unwrap();
        assert_eq!(token1.text, "a");

        let token2 = tokenizer.read().unwrap().unwrap();
        assert_eq!(token2.text, "1");

        let token2 = tokenizer.read().unwrap().unwrap();
        assert_eq!(token2.text, "b");
    }
}
