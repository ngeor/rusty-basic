use crate::common::{HasLocation, QError};
use crate::parser::pc::{BinaryParser, Parser, Reader, UnaryFnParser};
use crate::parser::pc_specific::{keyword_choice_p, keyword_followed_by_whitespace_p, PcSpecific};
use crate::parser::{ExitObject, Keyword, Statement};

pub fn statement_exit_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(Keyword::Exit)
        .and_demand(
            keyword_choice_p(&[Keyword::Function, Keyword::Sub])
                .or_syntax_error("Expected: FUNCTION or SUB"),
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
