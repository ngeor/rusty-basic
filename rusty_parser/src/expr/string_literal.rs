use rusty_pc::many::StringManyCombiner;
use rusty_pc::*;

use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::{MatchMode, TokenType, any_symbol_of, any_token_of};
use crate::{Expression, ExpressionPos, ParserError};

pub(super) fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    surround(
        string_delimiter(),
        inside_string(),
        string_delimiter(),
        SurroundMode::Mandatory,
    )
    .map(Expression::StringLiteral)
    .with_pos()
}

fn string_delimiter() -> impl Parser<StringView, Output = Token, Error = ParserError> {
    // TODO support ignoring token to avoid allocation
    any_symbol_of!('"')
}

fn inside_string() -> impl Parser<StringView, Output = String, Error = ParserError> {
    any_token_of!(
            types = TokenType::Eol ;
            symbols = '"' ;
            mode = MatchMode::Exclude)
    .many_allow_none(StringManyCombiner)
}
#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::{assert_literal_expression, *};

    #[test]
    fn test_parse_literals() {
        assert_literal_expression!(r#""hello, world""#, "hello, world");
        assert_literal_expression!(r#""hello 123 . AS""#, "hello 123 . AS");
        assert_literal_expression!("42", 42);
        assert_literal_expression!("4.2", 4.2_f32);
        assert_literal_expression!("0.5", 0.5_f32);
        assert_literal_expression!(".5", 0.5_f32);
        assert_literal_expression!("3.14#", 3.14_f64);
        assert_literal_expression!("-42", -42);
    }

    #[test]
    fn test_special_characters() {
        assert_literal_expression!(r#""┘""#, "┘");
    }
}
