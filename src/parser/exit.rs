use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{ExitObject, Keyword, Statement};

pub fn statement_exit_p() -> impl OptParser<Output = Statement> {
    Seq3::new(
        keyword(Keyword::Exit),
        whitespace(),
        keyword_map(&[
            (Keyword::Function, ExitObject::Function),
            (Keyword::Sub, ExitObject::Sub),
        ]),
    )
    .map(|(_, _, exit_object)| Statement::Exit(exit_object))
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
