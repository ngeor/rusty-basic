use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::parser::{
    Expression, ExpressionNode, ExpressionType, HasExpressionType, TypeQualifier, VariableInfo,
};

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
        match built_in_sub {
            BuiltInSub::Close => close::lint(args),
            BuiltInSub::Environ => environ_sub::lint(args),
            BuiltInSub::Input => input::lint(args),
            BuiltInSub::Kill => kill::lint(args),
            BuiltInSub::LineInput => line_input::lint(args),
            BuiltInSub::Name => name::lint(args),
            BuiltInSub::Open => open::lint(args),
        }
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
        BuiltInFunction::Chr => chr::lint(args),
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

mod close {
    use super::*;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            require_integer_argument(args, i)?;
        }
        Ok(())
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
    use crate::common::ToErrorEnvelopeNoPos;

    use super::*;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // the first one or two arguments stand for the file number
        // if the first argument is 0, no file handle
        // if the first argument is 1, the second is the file handle

        if args.len() <= 1 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        let mut has_file_number: bool = false;
        if let Locatable {
            element: Expression::IntegerLiteral(0),
            ..
        } = args[0]
        {
            // does not have a file number
        } else if let Locatable {
            element: Expression::IntegerLiteral(1),
            ..
        } = args[0]
        {
            // must have a file number
            if let Locatable {
                element: Expression::IntegerLiteral(_),
                ..
            } = args[1]
            {
                has_file_number = true;
            } else {
                panic!("parser sent unexpected arguments");
            }
        } else {
            panic!("parser sent unexpected arguments");
        }

        let starting_index = if has_file_number { 2 } else { 1 };
        if args.len() <= starting_index {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }

        for i in starting_index..args.len() {
            let Locatable { element, .. } = &args[i];
            match element {
                Expression::Variable(_, _) | Expression::Property(_, _, _) => {}
                _ => {
                    return Err(QError::VariableRequired).with_err_at(&args[i]);
                }
            }
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;

        use super::*;

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

mod line_input {
    use crate::common::ToErrorEnvelopeNoPos;

    use super::*;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // the first one or two arguments stand for the file number
        // if the first argument is 0, no file handle
        // if the first argument is 1, the second is the file handle

        if args.len() <= 1 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        let mut has_file_number: bool = false;
        if let Locatable {
            element: Expression::IntegerLiteral(0),
            ..
        } = args[0]
        {
            // does not have a file number
        } else if let Locatable {
            element: Expression::IntegerLiteral(1),
            ..
        } = args[0]
        {
            // must have a file number
            if let Locatable {
                element: Expression::IntegerLiteral(_),
                ..
            } = args[1]
            {
                has_file_number = true;
            } else {
                panic!("parser sent unexpected arguments");
            }
        } else {
            panic!("parser sent unexpected arguments");
        }

        let starting_index = if has_file_number { 2 } else { 1 };
        if args.len() != starting_index + 1 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }

        let Locatable {
            element: var_arg,
            pos,
        } = &args[starting_index];
        match var_arg {
            Expression::Variable(
                _,
                VariableInfo {
                    expression_type, ..
                },
            ) => match expression_type {
                ExpressionType::BuiltIn(TypeQualifier::DollarString)
                | ExpressionType::FixedLengthString(_) => {}
                _ => return Err(QError::TypeMismatch).with_err_at(*pos),
            },
            _ => return Err(QError::TypeMismatch).with_err_at(*pos),
        }

        Ok(())
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

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // must have 5 arguments:
        // filename
        // file mode
        // file access
        // file number
        // rec len
        if args.len() != 5 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        require_string_argument(args, 0)?;
        for i in 1..args.len() {
            require_integer_argument(args, i)?;
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_linter_err;

        #[test]
        fn test_open_filename_must_be_string() {
            let program = "OPEN 42 AS #1";
            assert_linter_err!(program, QError::ArgumentTypeMismatch, 1, 6);
        }

        #[test]
        fn test_rec_len_must_be_numeric() {
            let program = r#"OPEN "a.txt" AS #1 LEN = "hi""#;
            assert_linter_err!(program, QError::ArgumentTypeMismatch, 1, 26);
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
