use super::post_conversion_linter::PostConversionLinter;
use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::linter::types::{Expression, ExpressionNode, ExpressionType, HasExpressionType};
use crate::parser::TypeQualifier;

/// Lints built-in functions and subs.
pub struct BuiltInLinter;

impl PostConversionLinter for BuiltInLinter {
    fn visit_built_in_sub_call(
        &self,
        built_in_sub: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        self.visit_expressions(args)?;
        match built_in_sub {
            BuiltInSub::Close => close::lint(args),
            BuiltInSub::Environ => environ_sub::lint(args),
            BuiltInSub::Input => input::lint(args),
            BuiltInSub::Kill => kill::lint(args),
            BuiltInSub::LineInput => line_input::lint(args),
            BuiltInSub::Name => name::lint(args),
            BuiltInSub::Open => open::lint(args),
            BuiltInSub::Print => print::lint(args),
            BuiltInSub::System => system::lint(args),
        }
    }

    fn visit_expression(&self, expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
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
        BuiltInFunction::Chr => chr::lint(args),
        BuiltInFunction::Environ => environ_fn::lint(args),
        BuiltInFunction::Eof => eof::lint(args),
        BuiltInFunction::InStr => instr::lint(args),
        BuiltInFunction::Len => len::lint(args),
        BuiltInFunction::Mid => mid::lint(args),
        BuiltInFunction::Str => str_fn::lint(args),
        BuiltInFunction::Val => val::lint(args),
    }
}

mod close {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 1 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            require_file_handle_or_integer(&args[0])
        }
    }
}

mod environ_sub {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 1 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else if !args[0].can_cast_to(TypeQualifier::DollarString) {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[0])
        } else {
            Ok(())
        }
    }
}

mod input {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() == 0 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            args.iter()
                .map(|a| match a.as_ref() {
                    Expression::Variable(_) => Ok(()),
                    _ => Err(QError::VariableRequired).with_err_at(a),
                })
                .collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_linter_err;

        #[test]
        fn test_parenthesis_variable_required() {
            let input = "INPUT (A$)";
            assert_linter_err!(input, QError::VariableRequired);
        }

        #[test]
        fn test_binary_expression_variable_required() {
            let input = "INPUT A$ + B$";
            assert_linter_err!(input, QError::VariableRequired);
        }

        #[test]
        fn test_const() {
            let input = r#"
            CONST A$ = "hello"
            INPUT A$
            "#;
            assert_linter_err!(input, QError::VariableRequired);
        }
    }
}

mod kill {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        require_single_string_argument(args)
    }
}

mod line_input {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // two possible ways:
        // 1. #file-number%, variable$
        // 2. variable$
        if args.len() == 1 {
            require_string_variable(&args[0])
        } else if args.len() == 2 {
            require_file_handle(&args[0])?;
            require_string_variable(&args[1])
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }

    fn require_string_variable(arg: &ExpressionNode) -> Result<(), QErrorNode> {
        let Locatable { element, pos } = arg;
        match element {
            Expression::Variable(n) => {
                if n.can_cast_to(TypeQualifier::DollarString) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(pos)
                }
            }
            _ => Err(QError::VariableRequired).with_err_at(pos),
        }
    }
}

mod name {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 2 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else if !args[0].can_cast_to(TypeQualifier::DollarString) {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[0])
        } else if !args[1].can_cast_to(TypeQualifier::DollarString) {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[1])
        } else {
            Ok(())
        }
    }
}

mod open {
    use super::*;
    pub fn lint(_args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // not needed because of special parsing
        Ok(())
    }
}

mod print {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        for arg in args.iter() {
            let type_definition = arg.expression_type();
            match type_definition {
                ExpressionType::UserDefined(_) => {
                    return Err(QError::TypeMismatch).with_err_at(arg);
                }
                _ => {}
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_linter_err;

        #[test]
        fn cannot_print_user_defined_type() {
            let input = "
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM c AS Card
            PRINT c";
            assert_linter_err!(input, QError::TypeMismatch, 8, 19);
        }
    }
}

mod system {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 0 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            Ok(())
        }
    }
}

mod chr {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        require_single_numeric_argument(args)
    }
}

mod environ_fn {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        require_single_string_argument(args)
    }
}

mod eof {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        require_single_numeric_argument(args)
    }
}

mod instr {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() == 2 {
            require_string_argument(args, 0)?;
            require_string_argument(args, 1)
        } else if args.len() == 3 {
            require_integer_argument(args, 0)?;
            require_string_argument(args, 1)?;
            require_string_argument(args, 2)
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
                Expression::Variable(_) => Ok(()),
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
        use super::*;
        use crate::assert_linter_err;

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

        #[test]
        fn test_len_must_be_unqualified() {
            let program = r#"PRINT LEN!("hello")"#;
            assert_linter_err!(
                program,
                QError::syntax_error("Function Len must be unqualified"),
                1,
                7
            );
        }
    }
}

mod mid {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() == 2 {
            require_string_argument(args, 0)?;
            require_integer_argument(args, 1)
        } else if args.len() == 3 {
            require_string_argument(args, 0)?;
            require_integer_argument(args, 1)?;
            require_integer_argument(args, 2)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

mod str_fn {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        require_single_numeric_argument(args)
    }
}

mod val {
    use super::*;
    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        require_single_string_argument(args)
    }
}

fn require_single_numeric_argument(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
    if args.len() != 1 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        match args[0].expression_type() {
            ExpressionType::BuiltIn(q) => {
                if q == TypeQualifier::DollarString {
                    Err(QError::ArgumentTypeMismatch).with_err_at(&args[0])
                } else {
                    Ok(())
                }
            }
            _ => Err(QError::ArgumentTypeMismatch).with_err_at(&args[0]),
        }
    }
}

fn require_single_string_argument(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
    if args.len() != 1 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        require_string_argument(args, 0)
    }
}

fn require_string_argument(args: &Vec<ExpressionNode>, idx: usize) -> Result<(), QErrorNode> {
    if !args[idx].can_cast_to(TypeQualifier::DollarString) {
        Err(QError::ArgumentTypeMismatch).with_err_at(&args[idx])
    } else {
        Ok(())
    }
}

fn require_integer_argument(args: &Vec<ExpressionNode>, idx: usize) -> Result<(), QErrorNode> {
    if !args[idx].can_cast_to(TypeQualifier::PercentInteger) {
        Err(QError::ArgumentTypeMismatch).with_err_at(&args[idx])
    } else {
        Ok(())
    }
}

fn require_file_handle(arg: &ExpressionNode) -> Result<(), QErrorNode> {
    match arg.as_ref() {
        Expression::FileHandle(_) => Ok(()),
        _ => Err(QError::ArgumentTypeMismatch).with_err_at(arg),
    }
}

fn require_file_handle_or_integer(arg: &ExpressionNode) -> Result<(), QErrorNode> {
    match arg.as_ref() {
        Expression::FileHandle(_) => Ok(()),
        _ => {
            if arg.can_cast_to(TypeQualifier::PercentInteger) {
                Ok(())
            } else {
                Err(QError::ArgumentTypeMismatch).with_err_at(arg)
            }
        }
    }
}
