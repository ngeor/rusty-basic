use crate::parser::expression::in_parenthesis_csv_expressions_non_opt;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Expression> {
    seq2(
        keyword_dollar_sign(Keyword::String),
        in_parenthesis_csv_expressions_non_opt("Expected: expression"),
        |_, v| Expression::BuiltInFunctionCall(BuiltInFunction::String_, v),
    )
}
