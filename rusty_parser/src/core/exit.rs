use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{ExitObject, Keyword, ParserError, Statement};

pub fn statement_exit_p() -> impl Parser<RcStringView, Output = Statement, Error = ParserError> {
    seq2(
        keyword_ws_p(Keyword::Exit),
        keyword_map(&[
            (Keyword::Function, ExitObject::Function),
            (Keyword::Sub, ExitObject::Sub),
        ]),
        |_, exit_object| Statement::Exit(exit_object),
    )
}

#[cfg(test)]
mod tests {
    use crate::{ParserErrorKind, assert_parser_err};

    #[test]
    fn exit_without_object() {
        assert_parser_err!("EXIT ", ParserErrorKind::expected("FUNCTION or SUB"));
    }
}
