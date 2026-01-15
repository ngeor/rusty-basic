use rusty_pc::*;

use crate::core::name::bare_name_p;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::whitespace_ignoring;
use crate::{Keyword, ParseError, Statement};

pub fn statement_go_sub_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    keyword_followed_by_whitespace_p(Keyword::GoSub)
        .and_keep_right(bare_name_p().or_expected("label"))
        .map(Statement::GoSub)
}

pub fn statement_return_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
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
    use crate::error::ParseError;

    #[test]
    fn go_sub_without_label() {
        assert_parser_err!("GOSUB ", ParseError::expected("label"));
    }
}
