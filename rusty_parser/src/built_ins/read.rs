use crate::expression::csv_expressions_first_guarded;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

pub fn parse<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    keyword(Keyword::Read)
        .then_demand(csv_expressions_first_guarded().or_syntax_error("Expected: variable"))
        .map(|args| Statement::BuiltInSubCall(BuiltInSub::Read, args))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::ParseError;

    #[test]
    fn parse_must_have_at_least_one_argument() {
        assert_parser_err!("READ", ParseError::syntax_error("Expected: variable"));
    }
}
