use crate::parser::expression::csv_expressions_first_guarded;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

pub fn parse() -> impl Parser<Output = Statement> {
    keyword(Keyword::Read)
        .then_demand(csv_expressions_first_guarded().or_syntax_error("Expected: variable"))
        .map(|args| Statement::BuiltInSubCall(BuiltInSub::Read, args))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::QError;

    #[test]
    fn parse_must_have_at_least_one_argument() {
        assert_parser_err!("READ", QError::syntax_error("Expected: variable"));
    }
}
