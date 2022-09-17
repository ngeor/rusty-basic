pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::base::parsers::Parser;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        // INPUT variable-list
        // LINE INPUT variable$
        // INPUT #file-number%, variable-list
        // LINE INPUT #file-number%, variable$
        keyword_followed_by_whitespace_p(Keyword::Input)
            .and_opt(expression::file_handle_comma_p())
            .and_demand(
                expression::expression_node_p()
                    .csv()
                    .or_syntax_error("Expected: #file-number or variable"),
            )
            .map(|((_, opt_loc_file_number), variables)| {
                let mut args: Vec<ExpressionNode> = vec![];
                if let Some(Locatable { element, pos }) = opt_loc_file_number {
                    args.push(Expression::IntegerLiteral(1.into()).at(Location::start()));
                    args.push(Expression::IntegerLiteral(element.into()).at(pos));
                } else {
                    args.push(Expression::IntegerLiteral(0.into()).at(Location::start()));
                }
                args.extend(variables);
                Statement::BuiltInSubCall(BuiltInSub::Input, args)
            })
    }
}

pub mod linter {
    use crate::common::*;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::{Expression, ExpressionNode};

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
            args.require_integer_argument(1)?;
            has_file_number = true;
        } else {
            panic!("parser sent unexpected arguments");
        }

        let starting_index = if has_file_number { 2 } else { 1 };
        if args.len() <= starting_index {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }

        for i in starting_index..args.len() {
            args.require_variable_of_built_in_type(i)?;
        }

        Ok(())
    }
}

pub mod interpreter {
    use crate::common::{FileHandle, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Input;
    use crate::parser::TypeQualifier;
    use crate::variant::Variant;
    use std::convert::TryFrom;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_file_handle = false;
        for idx in 0..interpreter.context().variables().len() {
            let v: &Variant = &interpreter.context()[idx];
            match v {
                Variant::VInteger(f) => {
                    if idx == 0 {
                        has_file_handle = *f == 1;
                    } else if idx == 1 {
                        if has_file_handle {
                            file_handle = FileHandle::try_from(*f)?;
                        } else {
                            // input integer variable
                            do_input_one_var(interpreter, idx, file_handle)?;
                        }
                    } else {
                        // input integer variable
                        do_input_one_var(interpreter, idx, file_handle)?;
                    }
                }
                _ => {
                    do_input_one_var(interpreter, idx, file_handle)?;
                }
            }
        }
        Ok(())
    }

    fn do_input_one_var<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
        file_handle: FileHandle,
    ) -> Result<(), QError> {
        let raw_input: String = raw_input(interpreter, file_handle)?;
        let q: TypeQualifier = qualifier(interpreter, idx)?;
        let new_value: Variant = match q {
            TypeQualifier::BangSingle => Variant::from(parse_single_input(raw_input)?),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(parse_int_input(raw_input)?),
            _ => todo!("INPUT type {} not supported yet", q),
        };
        interpreter.context_mut()[idx] = new_value;
        Ok(())
    }

    fn raw_input<S: InterpreterTrait>(
        interpreter: &mut S,
        file_handle: FileHandle,
    ) -> Result<String, QError> {
        if file_handle.is_valid() {
            let file_input = interpreter
                .file_manager()
                .try_get_file_info_input(&file_handle)?;
            file_input.input().map_err(QError::from)
        } else {
            interpreter.stdin().input().map_err(QError::from)
        }
    }

    fn qualifier<S: InterpreterTrait>(
        interpreter: &S,
        idx: usize,
    ) -> Result<TypeQualifier, QError> {
        TypeQualifier::try_from(&interpreter.context()[idx])
    }

    fn parse_single_input(s: String) -> Result<f32, QError> {
        if s.is_empty() {
            Ok(0.0)
        } else {
            s.parse::<f32>()
                .map_err(|e| format!("Could not parse {} as float: {}", s, e).into())
        }
    }

    fn parse_int_input(s: String) -> Result<i32, QError> {
        if s.is_empty() {
            Ok(0)
        } else {
            s.parse::<i32>()
                .map_err(|e| format!("Could not parse {} as int: {}", s, e).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_built_in_sub_call;
    use crate::assert_has_variable;
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::{interpret, interpret_with_raw_input};
    use crate::parser::test_utils::*;
    use crate::parser::*;
    use crate::variant::Variant;

    #[test]
    fn test_parse_one_variable() {
        let input = "INPUT A$";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::Input,
            Expression::IntegerLiteral(0), // no file number
            Expression::var_unresolved("A$")
        );
    }

    #[test]
    fn test_parse_two_variables() {
        let input = "INPUT A$, B";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::Input,
            Expression::IntegerLiteral(0), // no file number
            Expression::var_unresolved("A$"),
            Expression::var_unresolved("B")
        );
    }

    #[test]
    fn test_no_whitespace_after_input() {
        let input = "INPUT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: whitespace after INPUT")
        );
    }

    #[test]
    fn test_no_variable() {
        let input = "INPUT ";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: #file-number or variable")
        );
    }

    #[test]
    fn test_file_hash_one_variable_space_after_comma() {
        let input = "INPUT #1, A";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::Input,
            Expression::IntegerLiteral(1), // has file number
            Expression::IntegerLiteral(1), // file number
            Expression::var_unresolved("A")
        );
    }

    #[test]
    fn test_file_hash_one_variable_no_comma() {
        let input = "INPUT #2,A";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::Input,
            Expression::IntegerLiteral(1), // has file number
            Expression::IntegerLiteral(2), // file number
            Expression::var_unresolved("A")
        );
    }

    #[test]
    fn test_file_hash_one_variable_space_before_comma() {
        let input = "INPUT #3 ,A";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::Input,
            Expression::IntegerLiteral(1), // has file number
            Expression::IntegerLiteral(3), // file number
            Expression::var_unresolved("A")
        );
    }

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

    fn assert_input<T>(
        raw_input: &str,
        variable_literal: &str,
        qualified_variable: &str,
        expected_value: T,
    ) where
        Variant: From<T>,
    {
        let input = format!("INPUT {}", variable_literal);
        let interpreter = interpret_with_raw_input(input, raw_input);
        assert_has_variable!(interpreter, qualified_variable, expected_value);
    }

    mod unqualified_var {
        use super::*;

        #[test]
        fn test_input_empty() {
            assert_input("\r\n", "N", "N!", 0.0_f32);
        }

        #[test]
        fn test_input_zero() {
            assert_input("0", "N", "N!", 0.0_f32);
        }

        #[test]
        fn test_input_single() {
            assert_input("1.1", "N", "N!", 1.1_f32);
        }

        #[test]
        fn test_input_negative() {
            assert_input("-1.2345", "N", "N!", -1.2345_f32);
        }

        #[test]
        fn test_input_explicit_positive() {
            assert_input("+3.14", "N", "N!", 3.14_f32);
        }

        #[test]
        fn test_space_is_trimmed() {
            assert_input("  42  ", "N", "N!", 42.0_f32);
        }
    }

    mod string_var {
        use super::*;
        use crate::interpreter::interpreter_trait::InterpreterTrait;

        #[test]
        fn test_input_one_variable_hello_no_cr_lf() {
            let input = r#"
            INPUT A$
            PRINT A$ + "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "hello");
            assert_eq!(interpreter.stdout().output(), "hello.");
        }

        #[test]
        fn test_input_one_variable_hello_cr_lf() {
            let input = r#"
            INPUT A$
            PRINT A$ + "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "hello\r\n");
            assert_eq!(interpreter.stdout().output_exact(), "hello.\r\n");
        }

        #[test]
        fn test_input_two_variables_hello_world_comma_no_cr_lf() {
            let input = r#"
            INPUT A$
            PRINT A$ + "."
            INPUT A$
            PRINT A$ + "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "hello, world");
            assert_eq!(interpreter.stdout().output_exact(), "hello.\r\nworld.\r\n");
        }

        #[test]
        fn test_space_is_trimmed() {
            let input = r#"
            INPUT A$
            PRINT A$ ; "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "  hello  ");
            assert_eq!(interpreter.stdout().output(), "hello.");
        }
    }

    mod int_var {
        use super::*;

        #[test]
        fn test_input_42() {
            assert_input("42", "A%", "A%", 42);
        }
    }

    #[test]
    fn test_input_dim_extended_builtin() {
        let input = "
        DIM X AS INTEGER
        INPUT X
        PRINT X
        ";
        let mut interpreter = interpret_with_raw_input(input, "42");
        assert_eq!(interpreter.stdout().output(), "42");
    }

    #[test]
    fn test_input_dim_user_defined() {
        let input = "
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 9
        END TYPE

        DIM X AS Card
        INPUT X.Value
        INPUT X.Suit
        PRINT X.Value
        PRINT X.Suit
        ";
        let mut interpreter = interpret_with_raw_input(input, "2, diamonds are forever");
        assert_eq!(interpreter.stdout().output_exact(), " 2 \r\ndiamonds \r\n");
    }

    #[test]
    fn test_input_inside_sub() {
        let input = "
        TYPE Card
            Value AS INTEGER
        END TYPE

        DIM X AS Card
        Test X.Value
        PRINT X.Value

        SUB Test(X%)
            INPUT X%
        END SUB
        ";
        let mut interpreter = interpret_with_raw_input(input, "42");
        assert_eq!(interpreter.stdout().output(), "42");
    }

    #[test]
    fn test_input_assign_to_function_directly() {
        let input = "
        X = Test
        PRINT X

        FUNCTION Test
            INPUT Test
        END FUNCTION
        ";
        let mut interpreter = interpret_with_raw_input(input, "3.14");
        assert_eq!(interpreter.stdout().output(), "3.14");
    }

    #[test]
    fn test_input_string_from_file() {
        // arrange
        std::fs::write("test1.txt", "Hello, world").unwrap();

        // act
        let input = r#"
        OPEN "test1.txt" FOR INPUT AS #1
        OPEN "test2.txt" FOR OUTPUT AS #2
        INPUT #1, A$
        PRINT #2, A$
        INPUT #1, A$
        PRINT #2, A$
        CLOSE
        "#;
        interpret(input);

        // assert
        let bytes = std::fs::read("test2.txt").expect("Should have created output file");
        let s: String = String::from_utf8(bytes).expect("Should be valid utf-8");
        std::fs::remove_file("test1.txt").unwrap_or_default();
        std::fs::remove_file("test2.txt").unwrap_or_default();
        assert_eq!(s, "Hello\r\nworld\r\n");
    }

    #[test]
    fn test_input_string_from_file_eof() {
        std::fs::write("test_input_string_from_file_eof.txt", "Hello, world").unwrap();

        let input = r#"
        OPEN "test_input_string_from_file_eof.txt" FOR INPUT AS #1
        INPUT #1, A$
        INPUT #1, A$
        INPUT #1, A$ ' should EOF here
        CLOSE
        "#;

        assert_interpreter_err!(input, QError::InputPastEndOfFile, 5, 9);
        std::fs::remove_file("test_input_string_from_file_eof.txt").unwrap_or_default();
    }
}
