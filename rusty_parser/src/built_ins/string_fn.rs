use crate::expression::in_parenthesis_csv_expressions_non_opt;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse<I: Tokenizer + 'static>() -> impl Parser<I, Output = Expression> {
    seq2(
        keyword_dollar_sign(Keyword::String),
        in_parenthesis_csv_expressions_non_opt("Expected: expression"),
        |_, v| Expression::BuiltInFunctionCall(BuiltInFunction::String, v),
    )
}
