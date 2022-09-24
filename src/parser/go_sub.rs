use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, KeepRightTrait, Parser};
use crate::parser::name::bare_name_p;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::specific::{keyword, keyword_followed_by_whitespace_p, OrErrorTrait};
use crate::parser::{Keyword, Statement};

pub fn statement_go_sub_p() -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoSub)
        .and_demand(bare_name_p().or_syntax_error("Expected: label"))
        .fn_map(|(_, l)| Statement::GoSub(l))
}

pub fn statement_return_p() -> impl Parser<Output = Statement> {
    keyword(Keyword::Return)
        .and_opt(bare_name_p().preceded_by_req_ws())
        .keep_right()
        .fn_map(Statement::Return)
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
