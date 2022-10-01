use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser::pc_specific::{keyword_syntax_error, TokenType};
use crate::parser::Keyword;

pub fn keyword_map<T>(mappings: &[(Keyword, T)]) -> KeywordMap<T>
where
    T: Clone,
{
    KeywordMap {
        mappings: mappings.to_vec(),
    }
}

pub struct KeywordMap<T> {
    mappings: Vec<(Keyword, T)>,
}

impl<T> Parser for KeywordMap<T>
where
    T: Clone,
{
    type Output = T;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(keyword_token) if keyword_token.kind == TokenType::Keyword as i32 => {
                for (keyword, mapped_value) in &self.mappings {
                    if keyword == &keyword_token {
                        return Ok(mapped_value.clone());
                    }
                }
                tokenizer.unread(keyword_token);
                self.to_err()
            }
            Some(other_token) => {
                tokenizer.unread(other_token);
                self.to_err()
            }
            None => self.to_err(),
        }
    }
}

impl<T> KeywordMap<T> {
    fn to_err(&self) -> Result<T, QError> {
        Err(QError::Expected(keyword_syntax_error(
            self.mappings.iter().map(|(k, _)| k),
        )))
    }
}
