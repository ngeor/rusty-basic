use std::collections::HashMap;

use rusty_pc::{Map, Parser};

use crate::input::RcStringView;
use crate::pc_specific::KeywordParser;
use crate::tokens::any_token;
use crate::{Keyword, ParseError};

/// A parser that parses one of the given keywords and returns the corresponding associated value.
pub fn keyword_map<T>(
    mappings: &[(Keyword, T)],
) -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: Clone,
{
    let keyword_to_value: HashMap<Keyword, T> = mappings.iter().cloned().collect();
    KeywordParser::new(any_token(), keyword_to_value.keys().copied())
        .map(move |keyword| keyword_to_value.get(&keyword).unwrap().clone())
}
