use crate::pc::string_view::RcStringView;
use crate::pc::{Recognition, Recognizer};
use crate::BareName;
use rusty_common::Position;
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
    pub pos: Position,
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
    fn read(&mut self) -> Option<Token>;
    fn unread(&mut self, token: Token);
    fn position(&self) -> Position;
}

pub type RecognizerWithTypePair = (TokenKind, Box<dyn Recognizer>);
pub type RecognizersWithType = Vec<RecognizerWithTypePair>;

pub fn create_tokenizer(reader: RcStringView, recognizers: RecognizersWithType) -> impl Tokenizer {
    UndoTokenizerImpl::new(TokenizerImpl::new(reader, recognizers))
}

struct TokenizerImpl {
    reader: RcStringView,
    recognizers: RecognizersWithType,
    eof_pos: Option<Position>,
}

struct UndoTokenizerImpl {
    tokenizer: TokenizerImpl,
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

impl TokenizerImpl {
    pub fn new(reader: RcStringView, recognizers: RecognizersWithType) -> Self {
        Self {
            reader,
            recognizers,
            eof_pos: None,
        }
    }

    pub fn pos(&self) -> Position {
        if self.reader.position() >= self.reader.len() {
            self.eof_pos.unwrap()
        } else {
            self.reader.row_col()
        }
    }

    pub fn read(&mut self) -> Option<Token> {
        let mut buffer = String::new();
        let mut no_match_or_eof = false;
        let mut recognizer_responses = RecognizerResponses::new(self.recognizers.len());
        let mut sizes: Vec<usize> = iter::repeat(0).take(self.recognizers.len()).collect();

        // temporary solution, store views to be able to "unread"
        let mut views: Vec<RcStringView> = vec![];

        let begin = if self.reader.position() >= self.reader.len() {
            Position::start()
        } else {
            self.reader.row_col()
        };

        while !no_match_or_eof {
            let eof = self.reader.position() >= self.reader.len();
            if !eof {
                let ch = self.reader.char();

                views.push(self.reader.clone());
                self.reader = self.reader.clone().inc_position();

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
                no_match_or_eof = recognizer_responses.count_by_recognition(Recognition::Negative)
                    == self.recognizers.len();
            } else {
                no_match_or_eof = true;

                if self.eof_pos.is_none() {
                    let mut temp = begin;
                    temp.inc_col();
                    self.eof_pos = Some(temp);
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
            buffer.pop().unwrap();
            self.reader = views.pop().unwrap();
        }

        if let Some(kind) = token_type {
            Some(Token {
                kind,
                text: buffer,
                pos: begin,
            })
        } else if buffer.is_empty() {
            None
        } else {
            panic!("Could not recognize token!");
        }
    }
}

impl UndoTokenizerImpl {
    pub fn new(tokenizer: TokenizerImpl) -> Self {
        Self {
            tokenizer,
            buffer: vec![],
        }
    }
}

impl Tokenizer for UndoTokenizerImpl {
    fn read(&mut self) -> Option<Token> {
        match self.buffer.pop() {
            Some(token) => Some(token),
            None => self.tokenizer.read(),
        }
    }

    fn unread(&mut self, token: Token) {
        self.buffer.push(token)
    }

    fn position(&self) -> Position {
        match self.buffer.last() {
            Some(token) => token.pos,
            _ => self.tokenizer.pos(),
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
    use crate::pc::string_view::RcStringView;
    use crate::pc::tokenizers::{TokenizerImpl, UndoTokenizerImpl};
    use crate::pc::*;

    fn many_letters_recognizer() -> impl Recognizer {
        all_chars_are(is_letter)
    }

    #[test]
    fn test_digits() {
        let input = "1234";
        let reader: RcStringView = input.into();
        let mut tokenizer = TokenizerImpl::new(
            reader,
            recognizers![
                0 => many_digits_recognizer()
            ],
        );
        let token = tokenizer.read().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 0);
        assert_eq!(token.pos.row(), 1);
        assert_eq!(token.pos.col(), 1);
    }

    #[test]
    fn test_letters_digits() {
        let input = "abc1234";
        let reader: RcStringView = input.into();
        let mut tokenizer = TokenizerImpl::new(
            reader,
            recognizers![
                0 => many_letters_recognizer(),
                1 => many_digits_recognizer()
            ],
        );
        let token = tokenizer.read().unwrap();
        assert_eq!(token.text, "abc");
        assert_eq!(token.kind, 0);
        assert_eq!(token.pos.row(), 1);
        assert_eq!(token.pos.col(), 1);
        assert_eq!(tokenizer.pos().row(), 1);
        assert_eq!(tokenizer.pos().col(), 4);
        let token = tokenizer.read().unwrap();
        assert_eq!(token.text, "1234");
        assert_eq!(token.kind, 1);
        assert_eq!(token.pos.row(), 1);
        assert_eq!(token.pos.col(), 4);
    }

    #[test]
    fn test_undo() {
        let input = "a1b2c3";
        let reader: RcStringView = input.into();
        let mut tokenizer = UndoTokenizerImpl::new(TokenizerImpl::new(
            reader,
            recognizers![
                0 => many_letters_recognizer(),
                1 => many_digits_recognizer(),
            ],
        ));

        let token = tokenizer.read().unwrap();
        assert_eq!(token.text, "a");

        tokenizer.unread(token);

        let token1 = tokenizer.read().unwrap();
        assert_eq!(token1.text, "a");

        let token2 = tokenizer.read().unwrap();
        assert_eq!(token2.text, "1");

        tokenizer.unread(token2);
        tokenizer.unread(token1);

        let token1 = tokenizer.read().unwrap();
        assert_eq!(token1.text, "a");

        let token2 = tokenizer.read().unwrap();
        assert_eq!(token2.text, "1");

        let token2 = tokenizer.read().unwrap();
        assert_eq!(token2.text, "b");
    }
}
