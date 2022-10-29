use crate::parser::char_reader::CharReader;
use crate::parser::pc::{Recognition, Recognizer};
use crate::parser::BareName;
use rusty_common::Location;
use std::iter;

pub type TokenKind = u8;

// TODO make fields private
/// Represents a recognized token.
///
/// The [kind] field could have been a generic parameter, but that would require
/// propagating the type in the [Tokenizer] and eventually also to the parsers.
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub pos: Location,
}

pub type TokenList = Vec<Token>;

pub fn token_list_to_string(tokens: TokenList) -> String {
    tokens.into_iter().map(|token| token.text).collect()
}

pub fn token_list_to_bare_name(tokens: TokenList) -> BareName {
    BareName::new(token_list_to_string(tokens))
}

pub trait Tokenizer {
    // TODO this can also be Result<Token, ?> where ? is Fatal/NotFound, or an Iterator
    fn read(&mut self) -> std::io::Result<Option<Token>>;
    fn unread(&mut self, token: Token);
    fn position(&self) -> Location;
}

pub type RecognizerWithTypePair = (TokenKind, Box<dyn Recognizer>);
pub type RecognizersWithType = Vec<RecognizerWithTypePair>;

pub fn create_tokenizer<R: CharReader>(
    reader: R,
    recognizers: RecognizersWithType,
) -> impl Tokenizer {
    UndoTokenizerImpl::new(TokenizerImpl::new(reader, recognizers))
}

struct TokenizerImpl<R: CharReader> {
    reader: R,
    recognizers: RecognizersWithType,
    pos: Location,
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
    pub fn new(reader: R, recognizers: RecognizersWithType) -> Self {
        Self {
            reader,
            recognizers,
            pos: Location::start(),
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
                    for (i, (_, recognizer)) in self.recognizers.iter().enumerate() {
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
        let mut token_type: Option<TokenKind> = None;
        for (i, (kind, _)) in self.recognizers.iter().enumerate() {
            if sizes[i] > max_positive_size {
                max_positive_size = sizes[i];
                token_type = Some(*kind);
            }
        }

        // unread extra characters
        while buffer.len() > max_positive_size {
            let last_char = buffer.pop().unwrap();
            self.reader.unread(last_char);
        }

        if let Some(kind) = token_type {
            let begin: Location = self.pos;
            let mut previous_char: char = ' ';
            for ch in buffer.chars() {
                if ch == '\r' {
                    self.pos.inc_row();
                } else if ch == '\n' {
                    if previous_char == '\r' {
                        // already increased row for '\r'
                    } else {
                        self.pos.inc_row();
                    }
                } else {
                    self.pos.inc_col();
                }
                previous_char = ch;
            }
            Ok(Some(Token {
                kind,
                text: buffer,
                pos: begin,
            }))
        } else if buffer.is_empty() {
            Ok(None)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not recognize token!",
            ))
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

    fn position(&self) -> Location {
        match self.buffer.last() {
            Some(token) => token.pos,
            _ => self.tokenizer.pos,
        }
    }
}

#[macro_export]
macro_rules! recognizers {
    [$($token_type:expr => $recognizer:expr),+$(,)?] => {
        vec![
            $(
            (
                $token_type.into(),
                Box::new($recognizer)
            )
            ),+
        ]
    };
}

#[cfg(test)]
mod tests {
    use crate::parser::char_reader::test_helper::string_char_reader;
    use crate::parser::pc::tokenizers::{TokenizerImpl, UndoTokenizerImpl};
    use crate::parser::pc::*;

    #[test]
    fn test_digits() {
        let input = "1234";
        let reader = string_char_reader(input);
        let mut tokenizer = TokenizerImpl::new(
            reader,
            recognizers![
                0 => many_digits_recognizer()
            ],
        );
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 0);
        assert_eq!(token.pos.row(), 1);
        assert_eq!(token.pos.col(), 1);
        assert_eq!(tokenizer.pos.row(), 1);
        assert_eq!(tokenizer.pos.col(), 5);
    }

    #[test]
    fn test_letters_digits() {
        let input = "abc1234";
        let reader = string_char_reader(input);
        let mut tokenizer = TokenizerImpl::new(
            reader,
            recognizers![
                0 => many_letters_recognizer(),
                1 => many_digits_recognizer()
            ],
        );
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "abc");
        assert_eq!(token.kind, 0);
        assert_eq!(token.pos.row(), 1);
        assert_eq!(token.pos.col(), 1);
        assert_eq!(tokenizer.pos.row(), 1);
        assert_eq!(tokenizer.pos.col(), 4);
        let token = tokenizer.read().unwrap().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 1);
        assert_eq!(token.pos.row(), 1);
        assert_eq!(token.pos.col(), 4);
        assert_eq!(tokenizer.pos.row(), 1);
        assert_eq!(tokenizer.pos.col(), 8);
    }

    #[test]
    fn test_undo() {
        let input = "a1b2c3";
        let reader = string_char_reader(input);
        let mut tokenizer = UndoTokenizerImpl::new(TokenizerImpl::new(
            reader,
            recognizers![
                0 => many_letters_recognizer(),
                1 => many_digits_recognizer(),
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
