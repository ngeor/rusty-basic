use crate::parser::name::bare_name_with_dots;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{Keyword, Statement};

pub fn statement_go_sub_p() -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoSub)
        .then_demand(bare_name_with_dots().or_syntax_error("Expected: label"))
        .map(Statement::GoSub)
}

pub fn statement_return_p() -> impl Parser<Output = Statement> {
    seq2(
        keyword(Keyword::Return),
        whitespace()
            .and(bare_name_with_dots())
            .keep_right()
            .allow_none(),
        |_, name| Statement::Return(name),
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use rusty_common::*;

    #[test]
    fn go_sub_without_label() {
        assert_parser_err!("GOSUB ", QError::syntax_error("Expected: label"));
    }
}
