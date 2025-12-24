use crate::pc::*;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::BuiltInFunction;

pub fn parse() -> impl Parser<RcStringView, Output = Expression> {
    seq2(
        keyword_dollar_sign(Keyword::String),
        in_parenthesis_csv_expressions_non_opt("Expected: expression"),
        |_, v| Expression::BuiltInFunctionCall(BuiltInFunction::String, v),
    )
}
