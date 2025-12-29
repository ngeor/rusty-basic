use crate::input::RcStringView;
use crate::pc::*;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::BuiltInSub;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    keyword(Keyword::Read)
        .and_keep_right(csv_expressions_first_guarded().or_syntax_error("Expected: variable"))
        .map(|args| Statement::built_in_sub_call(BuiltInSub::Read, args))
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
