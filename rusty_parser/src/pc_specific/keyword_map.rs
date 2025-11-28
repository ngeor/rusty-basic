use crate::pc::{Parser, Tokenizer};
use crate::pc_specific::{keyword_syntax_error, TokenType};
use crate::{Keyword, ParseError};

pub fn keyword_map<I: Tokenizer + 'static, T>(
    mappings: &[(Keyword, T)],
) -> impl Parser<I, Output = T>
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

impl<I: Tokenizer + 'static, T> Parser<I> for KeywordMap<T>
where
    T: Clone,
{
    type Output = T;
    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, ParseError> {
        match tokenizer.read()? {
            Some(keyword_token) if TokenType::Keyword.matches(&keyword_token) => {
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
    fn to_err(&self) -> Result<T, ParseError> {
        Err(ParseError::Expected(keyword_syntax_error(
            self.mappings.iter().map(|(k, _)| k),
        )))
    }
}
