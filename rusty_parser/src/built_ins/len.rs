use rusty_pc::*;

use crate::input::StringView;
use crate::pc_specific::*;
use crate::{BuiltInFunction, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Expression, Error = ParserError> {
    keyword(Keyword::Len)
        .and_keep_right(in_parenthesis_csv_expressions_non_opt("variable"))
        .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
}
