pub mod parser {
    use crate::built_ins::parser::parse_built_in_sub_with_opt_args;
    use crate::built_ins::BuiltInSub;
    use crate::parser::pc::*;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        parse_built_in_sub_with_opt_args(Keyword::Color, BuiltInSub::Color)
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        if args.len() < 2 || args.len() > 3 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            for i in 0..args.len() {
                args.require_numeric_argument(i)?;
            }
            Ok(())
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::QBNumberCast;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let flags: i32 = interpreter.context()[0].try_cast()?;
        let is_foreground_present = flags & 0x01 != 0;
        let is_background_present = flags & 0x02 != 0;
        if is_foreground_present {
            let foreground_color: i32 = interpreter.context()[1].try_cast()?;
            if is_background_present {
                // set both
                let background_color: i32 = interpreter.context()[2].try_cast()?;
                interpreter.screen().foreground_color(foreground_color)?;
                interpreter.screen().background_color(background_color)
            } else {
                // only set foreground color
                interpreter.screen().foreground_color(foreground_color)
            }
        } else if is_background_present {
            // only set background color
            let background_color: i32 = interpreter.context()[1].try_cast()?;
            interpreter.screen().background_color(background_color)
        } else {
            // should not happen
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::*;

    #[test]
    fn parse_foreground_only() {
        let input = "COLOR 7";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![1.as_lit_expr(1, 1), 7.as_lit_expr(1, 7)]
            )
        );
    }

    #[test]
    fn parse_background_only() {
        let input = "COLOR , 7";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![2.as_lit_expr(1, 1), 7.as_lit_expr(1, 9)]
            )
        );
    }

    #[test]
    fn parse_both_colors() {
        let input = "COLOR 7, 4";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![
                    3.as_lit_expr(1, 1),
                    7.as_lit_expr(1, 7),
                    4.as_lit_expr(1, 10)
                ]
            )
        );
    }

    #[test]
    fn parse_no_args() {
        let input = "COLOR";
        assert_parser_err!(input, QError::syntax_error("Expected: whitespace"), 1, 6);
    }

    #[test]
    fn lint_wrong_foreground_type() {
        let input = "COLOR A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_wrong_background_type() {
        let input = "COLOR , A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn lint_too_many_args() {
        let input = "COLOR 1, 2, 3";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }
}
