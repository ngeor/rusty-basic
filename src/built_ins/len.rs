pub mod parser {
    use crate::built_ins::BuiltInFunction;
    use crate::parser::base::parsers::Parser;
    use crate::parser::specific::{in_parenthesis, keyword_p};
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Expression> {
        keyword_p(Keyword::Len)
            .and_demand(
                in_parenthesis(
                    expression::lazy_expression_node_p()
                        .csv()
                        .or_syntax_error("Expected: variable"),
                )
            )
            .keep_right()
            .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
    }
}

pub mod linter {
    use crate::common::{CanCastTo, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
    use crate::parser::{
        Expression, ExpressionNode, ExpressionType, HasExpressionType, TypeQualifier,
    };

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() != 1 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            let arg: &Expression = args[0].as_ref();
            if arg.is_by_ref() {
                match arg.expression_type() {
                    // QBasic actually accepts LEN(A) where A is an array,
                    // but its results don't make much sense
                    ExpressionType::Unresolved | ExpressionType::Array(_) => {
                        Err(QError::ArgumentTypeMismatch).with_err_at(&args[0])
                    }
                    _ => Ok(()),
                }
            } else if !args[0].can_cast_to(TypeQualifier::DollarString) {
                Err(QError::VariableRequired).with_err_at(&args[0])
            } else {
                Ok(())
            }
        }
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::{AsciiSize, Variant};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let v: &Variant = &interpreter.context()[0];
        let len: i32 = v.ascii_size() as i32;
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Len, len);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

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
    fn test_len_string_literal() {
        let program = r#"PRINT LEN("hello")"#;
        assert_prints!(program, "5");
    }

    #[test]
    fn test_len_string_variable() {
        let program = r#"
        A$ = "hello"
        PRINT LEN(A$)
        "#;
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
    fn test_len_user_defined_type() {
        let program = "
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 9
        END TYPE
        DIM A AS Card
        PRINT LEN(A)
        ";
        assert_prints!(program, "11");
    }

    #[test]
    fn test_len_user_defined_type_nested_one_level() {
        let program = "
        TYPE PostCode
            Prefix AS STRING * 4
            Suffix AS STRING * 2
        END TYPE
        TYPE Address
            Street AS STRING * 50
            PostCode AS PostCode
        END TYPE
        DIM A AS Address
        PRINT LEN(A)
        ";
        assert_prints!(program, "56");
    }

    #[test]
    fn test_len_user_defined_type_nested_two_levels() {
        let program = "
        TYPE PostCode
            Prefix AS STRING * 4
            Suffix AS STRING * 2
        END TYPE
        TYPE Address
            Street AS STRING * 50
            PostCode AS PostCode
        END TYPE
        TYPE Person
            FullName AS STRING * 100
            Address AS Address
        END TYPE
        DIM A AS Person
        PRINT LEN(A)
        ";
        assert_prints!(program, "156");
    }

    #[test]
    fn test_len_user_defined_type_member() {
        let program = "
        TYPE PostCode
            Prefix AS STRING * 4
            Suffix AS STRING * 2
        END TYPE
        TYPE Address
            Street AS STRING * 50
            PostCode AS PostCode
        END TYPE
        TYPE Person
            FullName AS STRING * 100
            Address AS Address
        END TYPE
        DIM A AS Person
        PRINT LEN(A.Address)
        ";
        assert_prints!(program, "56");
    }

    #[test]
    fn test_fixed_length_string() {
        let program = r#"
        DIM X AS STRING * 5
        PRINT LEN(X)
        DIM Y AS STRING
        PRINT LEN(Y)
        "#;
        assert_prints!(program, "5", "0");
    }

    #[test]
    fn test_array_element() {
        let program = r#"
        DIM A(1 TO 2) AS INTEGER
        PRINT LEN(A(1))
        "#;
        assert_prints!(program, "2");
    }

    #[test]
    fn test_array() {
        let program = r#"
        DIM A(1 TO 2) AS INTEGER
        PRINT LEN(A)
        "#;
        // QBasic actually seems to be printing "4" regardless of the array type
        assert_linter_err!(program, QError::ArgumentTypeMismatch);
    }
}
