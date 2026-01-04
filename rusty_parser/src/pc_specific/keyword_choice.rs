use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::WithExpected;
use crate::tokens::{TokenType, any_token_of};
use crate::{Keyword, ParseError};

/// Matches one of the given keywords.
pub fn keyword_choice(
    keywords: Vec<Keyword>,
) -> impl Parser<RcStringView, Output = (Keyword, Token), Error = ParseError> {
    // TODO error message should be lazily evaluated
    let err_msg = keyword_syntax_error(&keywords);
    any_token_of!(TokenType::Keyword)
        .filter_map(move |token| {
            let needle: Keyword = Keyword::try_from(token.as_str()).unwrap();
            // TODO use a more efficient lookup
            if keywords.contains(&needle) {
                // TODO remove the need for cloning the token
                Some((needle, token.clone()))
            } else {
                None
            }
        })
        .with_expected_message(err_msg)
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
