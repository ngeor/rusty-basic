pub mod parser {
    use crate::built_ins::BuiltInFunction;
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse<R>() -> impl Parser<R, Output = Expression>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::Len)
            .and_demand(
                in_parenthesis_p(
                    expression::lazy_expression_node_p()
                        .csv()
                        .or_syntax_error("Expected: variable"),
                )
                .or_syntax_error("Expected: ("),
            )
            .keep_right()
            .map(|v| Expression::BuiltInFunctionCall(BuiltInFunction::Len, v))
    }
}

pub mod linter {
    use crate::common::{CanCastTo, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
    use crate::parser::{Expression, ExpressionNode, TypeQualifier};

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
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::{Locatable, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::parser::{ElementType, UserDefinedType, UserDefinedTypes};
    use crate::variant::Variant;
    use std::convert::TryInto;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let v: &Variant = &interpreter.context()[0];
        let len: i32 = match v {
            Variant::VSingle(_) => 4,
            Variant::VDouble(_) => 8,
            Variant::VString(v) => v.len().try_into().unwrap(),
            Variant::VInteger(_) => 2,
            Variant::VLong(_) => 4,
            Variant::VUserDefined(user_defined_value) => {
                let user_defined_type = interpreter
                    .user_defined_types()
                    .get(user_defined_value.type_name())
                    .unwrap();
                let sum: u32 =
                    len_of_user_defined_type(user_defined_type, interpreter.user_defined_types());
                sum as i32
            }
            Variant::VArray(_) => {
                return Err(QError::ArgumentTypeMismatch);
            }
        };
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Len, len);
        Ok(())
    }

    fn len_of_user_defined_type(
        user_defined_type: &UserDefinedType,
        types: &UserDefinedTypes,
    ) -> u32 {
        let mut sum: u32 = 0;
        for Locatable { element, .. } in user_defined_type.elements() {
            sum += match element.element_type() {
                ElementType::Single => 4,
                ElementType::Double => 8,
                ElementType::Integer => 2,
                ElementType::Long => 4,
                ElementType::FixedLengthString(_, l) => *l as u32,
                ElementType::UserDefined(Locatable {
                    element: type_name, ..
                }) => {
                    len_of_user_defined_type(types.get(type_name).expect("type not found"), types)
                }
            };
        }
        sum
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
}
