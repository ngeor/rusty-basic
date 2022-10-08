use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use crate::parser::Keyword;
use std::str::FromStr;

pub struct KeywordChoice<'a> {
    keywords: &'a [Keyword],
}

impl Undo for (Keyword, Token) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.1.undo(tokenizer)
    }
}

impl<'a> Parser for KeywordChoice<'a> {
    type Output = (Keyword, Token);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => match self.find_keyword(&token) {
                Some(keyword) => Ok((keyword, token)),
                None => {
                    tokenizer.unread(token);
                    self.to_err()
                }
            },
            None => self.to_err(),
        }
    }
}

impl<'a> KeywordChoice<'a> {
    fn find_keyword(&self, token: &Token) -> Option<Keyword> {
        if TokenType::Keyword.matches(&token) {
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

    fn to_err(&self) -> Result<(Keyword, Token), QError> {
        Err(QError::Expected(keyword_syntax_error(self.keywords.iter())))
    }
}

pub fn keyword_choice(keywords: &[Keyword]) -> impl Parser<Output = (Keyword, Token)> + '_ {
    KeywordChoice { keywords }
}

pub fn keyword_syntax_error<'a>(keywords: impl Iterator<Item = &'a Keyword>) -> String {
    let mut s = String::new();
    for keyword in keywords {
        if !s.is_empty() {
            s.push_str(" or ");
        }
        s.push_str(keyword.as_str());
    }
    format!("Expected: {}", s)
}
