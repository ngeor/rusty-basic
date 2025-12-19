use crate::expression::in_parenthesis_csv_expressions_non_opt;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse() -> impl Parser<RcStringView, Output = Expression> {
    keyword(Keyword::Len)
        .and_without_undo_keep_right(in_parenthesis_csv_expressions_non_opt("Expected: variable"))
        .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
}
