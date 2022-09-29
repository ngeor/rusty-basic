use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Tokenizer};
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

impl<T> ParserBase for KeywordMap<T> {
    type Output = T;
}

impl<T> OptParser for KeywordMap<T>
where
    T: Clone,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(keyword_token) if keyword_token.kind == TokenType::Keyword as i32 => {
                for (keyword, mapped_value) in &self.mappings {
                    if keyword == &keyword_token {
                        return Ok(Some(mapped_value.clone()));
                    }
                }
                tokenizer.unread(keyword_token);
                Ok(None)
            }
            Some(other_token) => {
                tokenizer.unread(other_token);
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

impl<T> NonOptParser for KeywordMap<T>
where
    T: Clone,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match OptParser::parse(self, tokenizer)? {
            Some(value) => Ok(value),
            None => Err(keyword_syntax_error(self.mappings.iter().map(|(k, _)| k))),
        }
    }
}
