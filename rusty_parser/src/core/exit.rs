use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::whitespace;
use crate::{ExitObject, Keyword, ParseError, Statement};

pub fn statement_exit_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq3(
        keyword(Keyword::Exit),
        whitespace(),
        keyword_map(&[
            (Keyword::Function, ExitObject::Function),
            (Keyword::Sub, ExitObject::Sub),
        ]),
        |_, _, exit_object| Statement::Exit(exit_object),
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::error::ParseError;

    #[test]
    fn exit_without_object() {
        assert_parser_err!("EXIT ", ParseError::expected("FUNCTION or SUB"));
    }
}
