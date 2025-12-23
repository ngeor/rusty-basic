use crate::built_ins::built_in_function::BuiltInFunction;
use crate::expression::in_parenthesis_csv_expressions_non_opt;
use crate::pc::*;
use crate::pc_specific::*;
use crate::specific::*;

pub fn parse() -> impl Parser<RcStringView, Output = Expression> {
    seq2(
        keyword_dollar_sign(Keyword::String),
        in_parenthesis_csv_expressions_non_opt("Expected: expression"),
        |_, v| Expression::BuiltInFunctionCall(BuiltInFunction::String, v),
    )
}
