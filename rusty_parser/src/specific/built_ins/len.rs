use rusty_pc::*;

use crate::input::RcStringView;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::{BuiltInFunction, ParseError};

pub fn parse() -> impl Parser<RcStringView, Output = Expression, Error = ParseError> {
    keyword(Keyword::Len)
        .and_keep_right(in_parenthesis_csv_expressions_non_opt("Expected: variable"))
        .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
}
