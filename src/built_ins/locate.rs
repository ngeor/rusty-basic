pub mod parser {
    use crate::built_ins::parser::parse_built_in_sub_with_opt_args;
    use crate::built_ins::BuiltInSub;
    use crate::parser::pc::*;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        parse_built_in_sub_with_opt_args(Keyword::Locate, BuiltInSub::Locate)
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() < 2 || args.len() > 4 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            for i in 0..args.len() {
                args.require_integer_argument(i)?;
            }
            Ok(())
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let mut iterator = interpreter.context().variables().iter();
        let flags: usize = iterator.next().unwrap().to_positive_int()?;
        let is_row_present = flags & 0x01 != 0;
        let is_col_present = flags & 0x02 != 0;
        let is_cursor_present = flags & 0x04 != 0;
        let row: Option<usize> = if is_row_present {
            Some(iterator.next().unwrap().to_positive_int()?)
        } else {
            None
        };
        let col: Option<usize> = if is_col_present {
            Some(iterator.next().unwrap().to_positive_int()?)
        } else {
            None
        };
        let cursor: Option<usize> = if is_cursor_present {
            Some(iterator.next().unwrap().to_non_negative_int()?)
        } else {
            None
        };
        move_to(interpreter, row, col)?;
        show_hide_cursor(interpreter, cursor)
    }

    fn move_to<S: InterpreterTrait>(
        interpreter: &S,
        row: Option<usize>,
        col: Option<usize>,
    ) -> Result<(), QError> {
        if let Some(row) = row {
            if let Some(col) = col {
                interpreter
                    .screen()
                    .move_to((row - 1) as u16, (col - 1) as u16)
            } else {
                interpreter.screen().move_to((row - 1) as u16, 0)
            }
        } else {
            if col.is_some() {
                // cannot move to a col because the current row is unknown
                Err(QError::IllegalFunctionCall)
            } else {
                Ok(())
            }
        }
    }

    fn show_hide_cursor<S: InterpreterTrait>(
        interpreter: &S,
        cursor: Option<usize>,
    ) -> Result<(), QError> {
        match cursor {
            Some(1) => interpreter.screen().show_cursor(),
            Some(0) => interpreter.screen().hide_cursor(),
            Some(_) => Err(QError::IllegalFunctionCall),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::QError;
    use crate::parser::test_utils::{
        parse_str, DemandSingleStatement, ExpressionNodeLiteralFactory,
    };
    use crate::parser::Statement;

    #[test]
    fn parse_row() {
        let input = "LOCATE 11";
        let statement = parse_str(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    1.as_lit_expr(1, 1),  // row present
                    11.as_lit_expr(1, 8), // row
                ]
            )
        );
    }

    #[test]
    fn parse_col() {
        let input = "LOCATE , 20";
        let statement = parse_str(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    2.as_lit_expr(1, 1),   // col present
                    20.as_lit_expr(1, 10), // col
                ]
            )
        );
    }

    #[test]
    fn parse_row_col() {
        let input = "LOCATE 10, 20";
        let statement = parse_str(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    3.as_lit_expr(1, 1),   // row and col present
                    10.as_lit_expr(1, 8),  // row
                    20.as_lit_expr(1, 12)  // col
                ]
            )
        );
    }

    #[test]
    fn parse_only_cursor_arg() {
        let input = "LOCATE , , 1";
        let statement = parse_str(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Locate,
                vec![
                    4.as_lit_expr(1, 1),  // cursor present
                    1.as_lit_expr(1, 12)  // cursor
                ]
            )
        );
    }

    #[test]
    fn cannot_have_trailing_comma() {
        assert_parser_err!(
            "LOCATE 1, 2,",
            QError::syntax_error("Error: trailing comma")
        );
    }

    #[test]
    fn lint_too_many_args() {
        assert_linter_err!("LOCATE 1, 2, 3, 4", QError::ArgumentCountMismatch);
    }
}
