use crate::common::{HasLocation, QError};
use crate::parser::name::bare_name_p;
use crate::parser::pc::{whitespace_p, BinaryParser, Parser, Reader, UnaryFnParser, UnaryParser};
use crate::parser::pc_specific::{keyword_followed_by_whitespace_p, keyword_p, PcSpecific};
use crate::parser::{Keyword, Statement};

pub fn statement_go_sub_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(Keyword::GoSub)
        .and_demand(bare_name_p().or_syntax_error("Expected: label"))
        .map(|(_, l)| Statement::GoSub(l))
}

pub fn statement_return_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Return)
        .and_opt(whitespace_p().and(bare_name_p()).keep_right())
        .map(|(_, opt_label)| Statement::Return(opt_label))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::QError;

    #[test]
    fn go_sub_without_label() {
        assert_parser_err!("GOSUB ", QError::syntax_error("Expected: label"));
    }
}
