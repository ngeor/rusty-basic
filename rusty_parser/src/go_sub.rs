use crate::name::bare_name_with_dots;
use crate::pc::*;
use crate::pc_specific::*;
use crate::{Keyword, Statement};

pub fn statement_go_sub_p() -> impl Parser<RcStringView, Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoSub)
        .and_without_undo_keep_right(bare_name_with_dots().or_syntax_error("Expected: label"))
        .map(Statement::GoSub)
}

pub fn statement_return_p() -> impl Parser<RcStringView, Output = Statement> {
    seq2(
        keyword(Keyword::Return),
        whitespace()
            .and_keep_right(bare_name_with_dots())
            .to_option(),
        |_, name| Statement::Return(name),
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::ParseError;

    #[test]
    fn go_sub_without_label() {
        assert_parser_err!("GOSUB ", ParseError::syntax_error("Expected: label"));
    }
}
