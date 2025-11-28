use crate::expression::in_parenthesis_csv_expressions_non_opt;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse<I: Tokenizer + 'static>() -> impl Parser<I, Output = Expression> {
    keyword(Keyword::Len)
        .then_demand(in_parenthesis_csv_expressions_non_opt("Expected: variable"))
        .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
}
