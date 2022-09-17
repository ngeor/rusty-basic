use crate::parser::base::parsers::Parser;
use crate::parser::specific::{keyword_choice, keyword_followed_by_whitespace_p};
use crate::parser::{ExitObject, Keyword, Statement};

pub fn statement_exit_p() -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::Exit)
        .and_demand(
            keyword_choice(&[Keyword::Function, Keyword::Sub])
        )
        .map(|(_, (k, _))| Statement::Exit(keyword_to_exit_object(k)))
}

fn keyword_to_exit_object(keyword: Keyword) -> ExitObject {
    match keyword {
        Keyword::Function => ExitObject::Function,
        Keyword::Sub => ExitObject::Sub,
        _ => panic!("Unsupported keyword {}", keyword),
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::QError;

    #[test]
    fn exit_without_object() {
        assert_parser_err!("EXIT ", QError::syntax_error("Expected: FUNCTION or SUB"));
    }
}
