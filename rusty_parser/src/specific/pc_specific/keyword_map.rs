use crate::input::RcStringView;
use crate::pc::{Errors, Map, Parser};
use crate::specific::pc_specific::{keyword_choice, keyword_syntax_error};
use crate::specific::Keyword;

pub fn keyword_map<T, K>(mappings: K) -> impl Parser<RcStringView, Output = T>
where
    K: AsRef<[(Keyword, T)]>,
    T: Clone,
{
    let keywords: Vec<Keyword> = mappings.as_ref().iter().map(|(k, _)| *k).collect();
    // TODO error message should be lazily evaluated
    let err_msg = keyword_syntax_error(&keywords);
    keyword_choice(keywords)
        .map(move |(keyword, _)| {
            // TODO this is inefficient, use a map instead
            for (k, mapped_value) in mappings.as_ref() {
                if *k == keyword {
                    return mapped_value.clone();
                }
            }
            unreachable!()
        })
        .with_expected_message(err_msg)
}
