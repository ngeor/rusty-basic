use crate::parser::expression::in_parenthesis_csv_expressions_non_opt;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Expression> {
    keyword(Keyword::Len)
        .then_demand(in_parenthesis_csv_expressions_non_opt("Expected: variable"))
        .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
}
