//
// KeywordParser
//

use crate::common::QError;
use crate::parser::pc::{Parser, Seq2, Seq3, Token, Tokenizer};
use crate::parser::pc_specific::{whitespace, TokenType};
use crate::parser::Keyword;

struct KeywordParser {
    keyword: Keyword,
}

impl Parser for KeywordParser {
    type Output = Token;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.keyword == token {
                    // check for trailing dollar sign
                    match tokenizer.read()? {
                        Some(follow_up) => {
                            if follow_up.kind == TokenType::DollarSign as i32 {
                                tokenizer.unread(follow_up);
                                tokenizer.unread(token);
                                self.to_err()
                            } else {
                                tokenizer.unread(follow_up);
                                Ok(token)
                            }
                        }
                        None => Ok(token),
                    }
                } else {
                    tokenizer.unread(token);
                    self.to_err()
                }
            }
            None => self.to_err(),
        }
    }
}

impl KeywordParser {
    fn to_err(&self) -> Result<Token, QError> {
        Err(QError::Expected(format!("Expected: {}", self.keyword)))
    }
}

pub fn keyword(keyword: Keyword) -> impl Parser<Output = Token> {
    KeywordParser { keyword }
}

// TODO #[deprecated]
pub fn keyword_followed_by_whitespace_p(k: Keyword) -> impl Parser {
    Seq2::new(keyword(k), whitespace())
}

pub fn keyword_pair(first: Keyword, second: Keyword) -> impl Parser {
    Seq3::new(keyword(first), whitespace(), keyword(second))
}
