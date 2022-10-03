pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::expression::csv_expressions_first_guarded;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword(Keyword::Read)
            .then_demand(csv_expressions_first_guarded().or_syntax_error("Expected: variable"))
            .map(|args| Statement::BuiltInSubCall(BuiltInSub::Read, args))
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        if args.is_empty() {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            for i in 0..args.len() {
                args.require_variable_of_built_in_type(i)?;
            }
            Ok(())
        }
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::parser::TypeQualifier;
    use std::convert::TryFrom;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        // variables are passed by ref, so we can assign to them
        let len = interpreter.context().variables().len();
        for i in 0..len {
            let target_type =
                TypeQualifier::try_from(interpreter.context().variables().get(i).unwrap())?;
            let data_value = interpreter.data_segment().pop()?;
            let casted_value = data_value.cast(target_type)?;
            interpreter.context_mut()[i] = casted_value;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn parse_must_have_at_least_one_argument() {
        assert_parser_err!("READ", QError::syntax_error("Expected: variable"));
    }

    #[test]
    fn single_literal_argument_not_allowed() {
        assert_linter_err!("READ 3.14", QError::VariableRequired);
    }

    #[test]
    fn double_literal_argument_not_allowed() {
        assert_linter_err!("READ 3.14#", QError::VariableRequired);
    }

    #[test]
    fn string_literal_argument_not_allowed() {
        assert_linter_err!("READ \"hello\"", QError::VariableRequired);
    }

    #[test]
    fn integer_literal_argument_not_allowed() {
        assert_linter_err!("READ 42", QError::VariableRequired);
    }

    #[test]
    fn long_literal_argument_not_allowed() {
        assert_linter_err!("READ 65536", QError::VariableRequired);
    }

    #[test]
    fn function_call_argument_not_allowed() {
        let input = r#"
        READ Hello(1)
        FUNCTION Hello(A)
        END FUNCTION
        "#;
        assert_linter_err!(input, QError::VariableRequired);
    }

    #[test]
    fn built_in_function_call_argument_not_allowed() {
        assert_linter_err!("READ LEN(A)", QError::VariableRequired);
    }

    #[test]
    fn binary_expression_argument_not_allowed() {
        assert_linter_err!("READ A + B", QError::VariableRequired);
    }

    #[test]
    fn unary_expression_argument_not_allowed() {
        assert_linter_err!("READ NOT A", QError::VariableRequired);
    }

    #[test]
    fn parenthesis_expression_argument_not_allowed() {
        assert_linter_err!("READ (A)", QError::VariableRequired);
    }

    #[test]
    fn array_variable_argument_not_allowed() {
        let input = r#"
        DIM A(1 TO 5)
        READ A
        "#;
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn user_defined_type_argument_not_allowed() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE
        DIM C AS Card
        READ C
        "#;
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn data_read_print() {
        let input = r#"
        DATA "the answer is", 42
        READ A$, B%
        PRINT A$, B%
        "#;
        assert_prints!(input, "the answer is  42");
    }

    #[test]
    fn read_into_property() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE

        DIM C AS Card

        DATA 42
        READ C.Value
        PRINT C.Value
        "#;
        assert_prints!(input, "42");
    }

    #[test]
    fn read_into_array_element() {
        let input = r#"
        DIM A(1 TO 5)
        DATA 1, 5, 9, 6, 7, 3, 2
        READ LO
        READ HI
        FOR I = LO TO HI
            READ A(I)
            PRINT A(I)
        NEXT
        "#;
        assert_prints!(input, "9", "6", "7", "3", "2");
    }

    #[test]
    fn cast_error_at_runtime() {
        let input = r#"
        DATA 42
        READ A$
        "#;
        assert_interpreter_err!(input, QError::TypeMismatch, 3, 9);
    }
}
