use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use crate::parser::Keyword;
use std::str::FromStr;

pub struct KeywordChoice<'a> {
    keywords: &'a [Keyword],
}

impl<'a> ParserBase for KeywordChoice<'a> {
    type Output = (Keyword, Token);
}

impl Undo for (Keyword, Token) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.1.undo(tokenizer)
    }
}

impl<'a> OptParser for KeywordChoice<'a> {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => match self.find_keyword(&token) {
                Some(keyword) => Ok(Some((keyword, token))),
                None => {
                    tokenizer.unread(token);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}

impl<'a> NonOptParser for KeywordChoice<'a> {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => match self.find_keyword(&token) {
                Some(keyword) => Ok((keyword, token)),
                None => Err(self.provide_error()),
            },
            None => Err(self.provide_error()),
        }
    }
}

impl<'a> KeywordChoice<'a> {
    fn find_keyword(&self, token: &Token) -> Option<Keyword> {
        if token.kind == TokenType::Keyword as i32 {
            // if it panics, it means the recognizer somehow has a bug
            let needle = Keyword::from_str(&token.text).unwrap();
            if self.keywords.contains(&needle) {
                Some(needle)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a> ErrorProvider for KeywordChoice<'a> {
    fn provide_error(&self) -> QError {
        keyword_syntax_error(self.keywords.iter())
    }
}

pub fn keyword_choice(
    keywords: &[Keyword],
) -> impl OptParser<Output = (Keyword, Token)> + NonOptParser<Output = (Keyword, Token)> + '_ {
    KeywordChoice { keywords }
}

pub fn keyword_syntax_error<'a>(keywords: impl Iterator<Item = &'a Keyword>) -> QError {
    let mut s = String::new();
    for keyword in keywords {
        if !s.is_empty() {
            s.push_str(" or ");
        }
        s.push_str(keyword.as_str());
    }
    QError::SyntaxError(format!("Expected: {}", s))
}
