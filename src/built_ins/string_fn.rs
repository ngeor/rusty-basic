pub mod parser {
    use crate::built_ins::BuiltInFunction;
    use crate::parser::base::parsers::Parser;
    use crate::parser::*;
    use crate::parser::specific::{item_p, keyword_p};

    pub fn parse() -> impl Parser<Output = Expression> {
        keyword_p(Keyword::String_)
            .and(item_p('$'))
            .and_demand(
                in_parenthesis_p(
                    expression::lazy_expression_node_p()
                        .csv()
                        .or_syntax_error("Expected: expression"),
                )
                .or_syntax_error("Expected: ("),
            )
            .keep_right()
            .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::String_, v))
    }
}

pub mod linter {
    use crate::common::{CanCastTo, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::{ExpressionNode, TypeQualifier};

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 2 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            args.require_integer_argument(0)?;
            if args[1].can_cast_to(TypeQualifier::PercentInteger)
                || args[1].can_cast_to(TypeQualifier::DollarString)
            {
                Ok(())
            } else {
                Err(QError::ArgumentTypeMismatch).with_err_at(&args[1])
            }
        }
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;
    use crate::variant::{QBNumberCast, Variant};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let count: usize = interpreter.context()[0].to_non_negative_int()?;
        let v = &interpreter.context()[1];
        let s = run_with_variant(count, v)?;
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::String_, s);
        Ok(())
    }

    fn run_with_variant(count: usize, v: &Variant) -> Result<String, QError> {
        if let Variant::VString(s) = v {
            run_with_string_argument(count, s)
        } else {
            let ascii: i32 = v.try_cast()?;
            run_with_ascii_code_argument(count, ascii)
        }
    }

    fn run_with_string_argument(count: usize, s: &str) -> Result<String, QError> {
        let first_char = s.chars().next().ok_or(QError::IllegalFunctionCall)?;
        run_with_char(count, first_char)
    }

    fn run_with_ascii_code_argument(count: usize, ascii: i32) -> Result<String, QError> {
        if ascii >= 0 && ascii <= 255 {
            let u: u8 = ascii as u8;
            run_with_char(count, u as char)
        } else {
            Err(QError::IllegalFunctionCall)
        }
    }

    fn run_with_char(count: usize, ch: char) -> Result<String, QError> {
        Ok(std::iter::repeat(ch).take(count).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn string_without_args() {
        assert_linter_err!("PRINT STRING$()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn string_with_only_one_arg() {
        assert_linter_err!("PRINT STRING$(5)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_with_three_arguments() {
        assert_linter_err!("PRINT STRING$(1, 2, 3)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_with_string_first_argument() {
        assert_linter_err!(
            r#"PRINT STRING$("oops", "oops")"#,
            QError::ArgumentTypeMismatch
        );
    }

    #[test]
    fn string_with_ascii_code() {
        assert_prints!("PRINT STRING$(3, 33)", "!!!");
    }

    #[test]
    fn string_with_string_argument() {
        assert_prints!(r#"PRINT STRING$(4, "hello")"#, "hhhh");
    }

    #[test]
    fn string_with_empty_string_argument() {
        assert_interpreter_err!(r#"PRINT STRING$(5, "")"#, QError::IllegalFunctionCall, 1, 7);
    }

    #[test]
    fn string_with_zero_count() {
        assert_prints!(r#"PRINT STRING$(0, "hello")"#, "");
    }

    #[test]
    fn string_with_negative_count() {
        assert_interpreter_err!(
            r#"PRINT STRING$(-1, "hello")"#,
            QError::IllegalFunctionCall,
            1,
            7
        );
    }

    #[test]
    fn string_with_negative_ascii_code() {
        assert_interpreter_err!("PRINT STRING$(10, -1)", QError::IllegalFunctionCall, 1, 7);
    }

    #[test]
    fn string_with_too_big_ascii_code() {
        assert_interpreter_err!("PRINT STRING$(10, 256)", QError::IllegalFunctionCall, 1, 7);
    }
}
