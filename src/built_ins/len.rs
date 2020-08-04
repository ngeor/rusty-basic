// LEN(str_expr$) -> number of characters in string
// LEN(variable) -> number of bytes required to store a variable

use super::{BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{err, Interpreter, InterpreterError, Stdlib};
use crate::linter::{
    err_l, err_no_pos, Error, Expression, ExpressionNode, LinterError, TypeQualifier,
};
use crate::variant::Variant;
use std::convert::TryInto;

pub struct Len {}

impl BuiltInLint for Len {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else {
            let arg: &Expression = args[0].as_ref();
            match arg {
                Expression::Variable(_) => Ok(()),
                _ => {
                    let q = args[0].try_qualifier()?;
                    if q != TypeQualifier::DollarString {
                        err_l(LinterError::VariableRequired, &args[0])
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }
}

impl BuiltInRun for Len {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        let v = interpreter.pop_unnamed_val().unwrap();
        interpreter.function_result = match v {
            Variant::VSingle(_) => Variant::VInteger(4),
            Variant::VDouble(_) => Variant::VInteger(8),
            Variant::VString(v) => Variant::VInteger(v.len().try_into().unwrap()),
            Variant::VInteger(_) => Variant::VInteger(2),
            Variant::VLong(_) => Variant::VInteger(4),
            _ => {
                return err("Not supported", pos);
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::linter::LinterError;

    #[test]
    fn test_len_string() {
        let program = r#"PRINT LEN("hello")"#;
        assert_prints!(program, "5");
    }

    #[test]
    fn test_len_float_variable() {
        let program = "
            A = 3.14
            PRINT LEN(A)
            ";
        assert_prints!(program, "4");
    }

    #[test]
    fn test_len_double_variable() {
        let program = "
            A# = 3.14
            PRINT LEN(A#)
            ";
        assert_prints!(program, "8");
    }

    #[test]
    fn test_len_integer_variable() {
        let program = "
            A% = 42
            PRINT LEN(A%)
            ";
        assert_prints!(program, "2");
    }

    #[test]
    fn test_len_long_variable() {
        let program = "
            A& = 42
            PRINT LEN(A&)
            ";
        assert_prints!(program, "4");
    }

    #[test]
    fn test_len_integer_expression_error() {
        let program = "PRINT LEN(42)";
        assert_linter_err!(program, LinterError::VariableRequired, 1, 11);
    }

    #[test]
    fn test_len_integer_const_error() {
        let program = "
            CONST X = 42
            PRINT LEN(X)
            ";
        assert_linter_err!(program, LinterError::VariableRequired, 3, 23);
    }

    #[test]
    fn test_len_two_arguments_error() {
        let program = r#"PRINT LEN("a", "b")"#;
        assert_linter_err!(program, LinterError::ArgumentCountMismatch, 1, 7);
    }

    #[test]
    fn test_len_must_be_unqualified() {
        let program = r#"PRINT LEN!("hello")"#;
        assert_linter_err!(program, LinterError::SyntaxError, 1, 7);
    }
}