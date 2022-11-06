use crate::pc::*;
use crate::pc_specific::*;
use crate::{ExitObject, Keyword, Statement};

pub fn statement_exit_p() -> impl Parser<Output = Statement> {
    seq3(
        keyword(Keyword::Exit),
        whitespace().no_incomplete(),
        keyword_map(&[
            (Keyword::Function, ExitObject::Function),
            (Keyword::Sub, ExitObject::Sub),
        ])
        .no_incomplete(),
        |_, _, exit_object| Statement::Exit(exit_object),
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::ParseError;

    #[test]
    fn exit_without_object() {
        assert_parser_err!(
            "EXIT ",
            ParseError::syntax_error("Expected: FUNCTION or SUB")
        );
    }
}
