use crate::pc::*;
use crate::pc_specific::{any_token_of, TokenType};
use crate::{Keyword, ParseError};

impl Undo for (Keyword, Token) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.1.undo(tokenizer)
    }
}

/// Matches one of the given keywords.
pub fn keyword_choice<I: Tokenizer + 'static>(
    keywords: Vec<Keyword>,
) -> impl Parser<I, Output = (Keyword, Token)> {
    // TODO error message should be lazily evaluated
    let err_msg = keyword_syntax_error(&keywords);
    any_token_of(TokenType::Keyword)
        .filter_map(move |token| {
            let needle: Keyword = token.into();
            // TODO use a more efficient lookup
            if keywords.contains(&needle) {
                // TODO remove the need for cloning the token
                Some((needle, token.clone()))
            } else {
                None
            }
        })
        .map_incomplete_err(ParseError::Expected(err_msg))
}

pub fn keyword_syntax_error<K>(keywords: K) -> String
where
    K: AsRef<[Keyword]>,
{
    let mut s = String::new();
    for keyword in keywords.as_ref() {
        if !s.is_empty() {
            s.push_str(" or ");
        }
        s.push_str(&keyword.to_string());
    }
    format!("Expected: {}", s)
}
