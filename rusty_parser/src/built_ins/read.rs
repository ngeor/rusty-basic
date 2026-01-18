use rusty_pc::*;

use crate::input::StringView;
use crate::pc_specific::*;
use crate::{BuiltInSub, ParserError, *};

pub fn parse() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword(Keyword::Read)
        .and_keep_right(csv_expressions_first_guarded().or_expected("variable"))
        .map(|args| Statement::built_in_sub_call(BuiltInSub::Read, args))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;

    #[test]
    fn parse_must_have_at_least_one_argument() {
        assert_parser_err!("READ", expected("variable"));
    }
}
