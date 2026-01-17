use rusty_pc::*;

use crate::input::RcStringView;
use crate::tokens::{TokenType, any_token_of, dollar_sign};
use crate::{BuiltInFunction, ParseError, *};

pub fn parse() -> impl Parser<RcStringView, Output = Expression, Error = ParseError> {
    seq2(
        any_token_of!(TokenType::Identifier)
            .filter(|token: &Token| token.as_str().eq_ignore_ascii_case("STRING"))
            .and(dollar_sign(), IgnoringBothCombiner),
        in_parenthesis_csv_expressions_non_opt("expression"),
        |_, v| Expression::BuiltInFunctionCall(BuiltInFunction::String, v),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::create_string_tokenizer;

    #[test]
    pub fn test() {
        let input = "STRING$(42)";
        let input = create_string_tokenizer(input.to_owned());
        let result = super::parse().parse(input).ok().unwrap().1;
        assert!(matches!(
            result,
            Expression::BuiltInFunctionCall(BuiltInFunction::String, _)
        ));
    }
}
