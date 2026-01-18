use rusty_pc::*;

use crate::core::name::bare_name_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::whitespace_ignoring;
use crate::{Keyword, ParserError, Statement};

pub fn statement_go_sub_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword_ws_p(Keyword::GoSub)
        .and_keep_right(bare_name_p().or_expected("label"))
        .map(Statement::GoSub)
}

pub fn statement_return_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq2(
        keyword(Keyword::Return),
        whitespace_ignoring()
            .and_keep_right(bare_name_p())
            .to_option(),
        |_, name| Statement::Return(name),
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;

    #[test]
    fn go_sub_without_label() {
        assert_parser_err!("GOSUB ", expected("label"));
    }
}
