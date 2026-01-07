use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{BuiltInFunction, ParseError, *};

pub fn parse() -> impl Parser<RcStringView, Output = Expression, Error = ParseError> {
    keyword(Keyword::Len)
        .and_keep_right(in_parenthesis_csv_expressions_non_opt("variable"))
        .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
}
