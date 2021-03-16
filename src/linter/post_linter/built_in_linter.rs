use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::parser::{Expression, ExpressionNode, ExpressionType, TypeQualifier, VariableInfo};

use super::post_conversion_linter::PostConversionLinter;

/// Lints built-in functions and subs.
pub struct BuiltInLinter;

impl PostConversionLinter for BuiltInLinter {
    fn visit_built_in_sub_call(
        &mut self,
        built_in_sub: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)?;
        crate::built_ins::linter::lint_sub_call(built_in_sub, args)
    }

    fn visit_expression(&mut self, expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
        let pos = expr_node.pos();
        let e = expr_node.as_ref();
        match e {
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                lint(built_in_function, args).patch_err_pos(pos)
            }
            Expression::BinaryExpression(_, left, right, _) => {
                self.visit_expression(left)?;
                self.visit_expression(right)
            }
            Expression::UnaryExpression(_, child) => self.visit_expression(child),
            _ => Ok(()),
        }
    }
}

fn lint(built_in: &BuiltInFunction, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
    match built_in {
        BuiltInFunction::Chr => crate::built_ins::chr::linter::lint(args),
        BuiltInFunction::Environ => environ_fn::lint(args),
        BuiltInFunction::Eof => eof::lint(args),
        BuiltInFunction::InStr => instr::lint(args),
        BuiltInFunction::LBound | BuiltInFunction::UBound => lbound::lint(args),
        BuiltInFunction::Len => len::lint(args),
        BuiltInFunction::Mid => mid::lint(args),
        BuiltInFunction::Str => str_fn::lint(args),
        BuiltInFunction::Val => val::lint(args),
    }
}

mod lbound {
    use super::*;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.is_empty() || args.len() > 2 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }

        // Can have at one or two arguments. First must be the array name, without parenthesis.
        // Second, optional, is an integer specifying the array dimension >=1 (default is 1).
        let Locatable {
            element: first,
            pos: first_pos,
        } = args.get(0).unwrap();
        if let Expression::Variable(
            _,
            VariableInfo {
                expression_type: ExpressionType::Array(_),
                ..
            },
        ) = first
        {
            if args.len() == 2 {
                if args[1].can_cast_to(TypeQualifier::PercentInteger) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(args[1].pos())
                }
            } else {
                Ok(())
            }
        } else {
            Err(QError::ArgumentTypeMismatch).with_err_at(first_pos)
        }
    }
}

mod environ_fn {
    use super::*;
    use crate::linter::arg_validation::ArgValidation;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_string_argument()
    }
}

mod eof {
    use super::*;
    use crate::linter::arg_validation::ArgValidation;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_numeric_argument()
    }
}

mod instr {
    use super::*;
    use crate::linter::arg_validation::ArgValidation;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() == 2 {
            args.require_string_argument(0)?;
            args.require_string_argument(1)
        } else if args.len() == 3 {
            args.require_integer_argument(0)?;
            args.require_string_argument(1)?;
            args.require_string_argument(2)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

mod len {
    use super::*;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 1 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            let arg: &Expression = args[0].as_ref();
            match arg {
                Expression::Variable(_, _) | Expression::Property(_, _, _) => Ok(()),
                _ => {
                    if !args[0].can_cast_to(TypeQualifier::DollarString) {
                        Err(QError::VariableRequired).with_err_at(&args[0])
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;

        use super::*;

        #[test]
        fn test_len_integer_expression_error() {
            let program = "PRINT LEN(42)";
            assert_linter_err!(program, QError::VariableRequired, 1, 11);
        }

        #[test]
        fn test_len_integer_const_error() {
            let program = "
            CONST X = 42
            PRINT LEN(X)
            ";
            assert_linter_err!(program, QError::VariableRequired, 3, 23);
        }

        #[test]
        fn test_len_two_arguments_error() {
            let program = r#"PRINT LEN("a", "b")"#;
            assert_linter_err!(program, QError::ArgumentCountMismatch, 1, 7);
        }
    }
}

mod mid {
    use super::*;
    use crate::linter::arg_validation::ArgValidation;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() == 2 {
            args.require_string_argument(0)?;
            args.require_integer_argument(1)
        } else if args.len() == 3 {
            args.require_string_argument(0)?;
            args.require_integer_argument(1)?;
            args.require_integer_argument(2)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

mod str_fn {
    use super::*;
    use crate::linter::arg_validation::ArgValidation;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_numeric_argument()
    }
}

mod val {
    use super::*;
    use crate::linter::arg_validation::ArgValidation;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_string_argument()
    }
}
