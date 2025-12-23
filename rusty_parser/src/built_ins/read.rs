use crate::built_ins::built_in_sub::BuiltInSub;
use crate::expression::csv_expressions_first_guarded;
use crate::pc::*;
use crate::pc_specific::*;
use crate::specific::*;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    keyword(Keyword::Read)
        .and_without_undo_keep_right(
            csv_expressions_first_guarded().or_syntax_error("Expected: variable"),
        )
        .map(|args| Statement::BuiltInSubCall(BuiltInSub::Read, args))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::error::ParseError;

    #[test]
    fn parse_must_have_at_least_one_argument() {
        assert_parser_err!("READ", ParseError::syntax_error("Expected: variable"));
    }
}
