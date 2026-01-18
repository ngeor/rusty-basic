use rusty_pc::*;

use crate::input::StringView;
use crate::tokens::{TokenType, any_token_of, dollar_sign};
use crate::{BuiltInFunction, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Expression, Error = ParserError> {
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
        let mut input = create_string_tokenizer(input.to_owned());
        let result = super::parse().parse(&mut input).ok().unwrap();
        assert!(matches!(
            result,
            Expression::BuiltInFunctionCall(BuiltInFunction::String, _)
        ));
    }
}
