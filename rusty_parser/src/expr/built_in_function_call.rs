use rusty_pc::Parser;

use crate::built_ins::built_in_function_call_p;
use crate::input::StringView;
use crate::pc_specific::WithPos;
use crate::{ParserError, *};

pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    built_in_function_call_p().with_pos()
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    mod len {
        use super::*;

        #[test]
        fn len_in_print_must_be_unqualified() {
            let program = r#"PRINT LEN!("hello")"#;
            assert_parser_err!(program, expected("("), 1, 10);
        }

        #[test]
        fn len_in_assignment_must_be_unqualified() {
            let program = r#"A = LEN!("hello")"#;
            assert_parser_err!(program, expected("("), 1, 8);
        }
    }
}
