pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::base::and_pc::AndDemandTrait;
    use crate::parser::base::parsers::{AndOptTrait, FnMapTrait, Parser};
    use crate::parser::specific::{keyword_pair_p, whitespace, OrSyntaxErrorTrait};
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword_pair_p(Keyword::Line, Keyword::Input)
            .and_demand(whitespace())
            .and_opt(expression::file_handle_comma_p())
            .and_demand(
                expression::expression_node_p()
                    .or_syntax_error("Expected: #file-number or variable"),
            )
            .fn_map(|((_, opt_loc_file_handle), variable)| {
                let mut args: Vec<ExpressionNode> = vec![];
                // add dummy arguments to encode the file number
                if let Some(Locatable { element, pos }) = opt_loc_file_handle {
                    args.push(Expression::IntegerLiteral(1.into()).at(Location::start()));
                    args.push(Expression::IntegerLiteral(element.into()).at(pos));
                } else {
                    args.push(Expression::IntegerLiteral(0.into()).at(Location::start()));
                }
                // add the LINE INPUT variable
                args.push(variable);
                Statement::BuiltInSubCall(BuiltInSub::LineInput, args)
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

        args.require_string_ref(starting_index)
    }
}

pub mod interpreter {
    use crate::common::{FileHandle, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Input;
    use crate::variant::Variant;
    use std::convert::TryFrom;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_file_handle = false;
        for idx in 0..interpreter.context().variables().len() {
            let v = &interpreter.context()[idx];
            match v {
                Variant::VInteger(f) => {
                    if idx == 0 {
                        has_file_handle = *f == 1;
                    } else if idx == 1 {
                        if has_file_handle {
                            file_handle = FileHandle::try_from(*f)?;
                        } else {
                            // input integer variable?
                            panic!("Linter should have caught this");
                        }
                    } else {
                        panic!("Linter should have caught this");
                    }
                }
                Variant::VString(_) => {
                    line_input_one(interpreter, idx, &file_handle)?;
                }
                _ => panic!("Linter should have caught this"),
            }
        }

        Ok(())
    }

    fn line_input_one<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
        file_handle: &FileHandle,
    ) -> Result<(), QError> {
        if file_handle.is_valid() {
            line_input_one_file(interpreter, idx, file_handle)
        } else {
            line_input_one_stdin(interpreter, idx)
        }
    }

    fn line_input_one_file<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
        file_handle: &FileHandle,
    ) -> Result<(), QError> {
        let file_input = interpreter
            .file_manager()
            .try_get_file_info_input(file_handle)?;
        let s = file_input.line_input()?;
        interpreter.context_mut()[idx] = Variant::VString(s);
        Ok(())
    }

    fn line_input_one_stdin<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
    ) -> Result<(), QError> {
        let s = interpreter.stdin().input()?;
        interpreter.context_mut()[idx] = Variant::VString(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_built_in_sub_call;
    use crate::assert_interpreter_err;
    use crate::assert_parser_err;
    use crate::assert_prints;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::parser::test_utils::*;
    use crate::parser::*;

    #[test]
    fn test_parse_one_variable() {
        let input = "LINE INPUT A$";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::LineInput,
            Expression::IntegerLiteral(0), // no file number
            Expression::var_unresolved("A$")
        );
    }

    #[test]
    fn test_parse_two_variables() {
        let input = "LINE INPUT A$, B";
        assert_parser_err!(input, QError::syntax_error("No separator: ,"));
    }

    #[test]
    fn test_no_whitespace_after_input() {
        let input = "LINE INPUT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: whitespace after LINE INPUT")
        );
    }

    #[test]
    fn test_no_variable() {
        let input = "LINE INPUT ";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: #file-number or variable")
        );
    }

    #[test]
    fn test_file_hash_one_variable_space_after_comma() {
        let input = "LINE INPUT #1, A";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::LineInput,
            Expression::IntegerLiteral(1), // has file number
            Expression::IntegerLiteral(1), // file number
            Expression::var_unresolved("A")
        );
    }

    #[test]
    fn test_file_hash_one_variable_no_comma() {
        let input = "LINE INPUT #2,A";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::LineInput,
            Expression::IntegerLiteral(1), // has file number
            Expression::IntegerLiteral(2), // file number
            Expression::var_unresolved("A")
        );
    }

    #[test]
    fn test_file_hash_one_variable_space_before_comma() {
        let input = "LINE INPUT #1 ,A";
        assert_built_in_sub_call!(
            input,
            BuiltInSub::LineInput,
            Expression::IntegerLiteral(1), // has file number
            Expression::IntegerLiteral(1), // file number
            Expression::var_unresolved("A")
        );
    }

    #[test]
    fn test_line_input_string_from_file_eof() {
        std::fs::remove_file("test_line_input_string_from_file_eof.txt").unwrap_or_default();
        std::fs::write(
            "test_line_input_string_from_file_eof.txt",
            "Hello\r\nWorld\r\n",
        )
        .unwrap();

        let input = r#"
        OPEN "test_line_input_string_from_file_eof.txt" FOR INPUT AS #1
        LINE INPUT #1, A$
        LINE INPUT #1, A$
        LINE INPUT #1, A$ ' should EOF here
        CLOSE
        "#;

        assert_interpreter_err!(input, QError::InputPastEndOfFile, 5, 9);
        std::fs::remove_file("test_line_input_string_from_file_eof.txt").unwrap_or_default();
    }

    #[test]
    fn line_input_reading_into_array_user_defined_type_string() {
        let filename = "line_input_reading_into_array_user_defined_type_string.txt";
        std::fs::remove_file(filename).unwrap_or_default();
        std::fs::write(filename, "Hello world!!!\r\n").unwrap();
        let input = format!(
            r#"
        TYPE MyType
            Greeting AS STRING * 11
        END TYPE

        DIM A(1 TO 2) AS MyType

        OPEN "{}" FOR INPUT AS #1
        LINE INPUT #1, A(1).Greeting
        CLOSE

        PRINT A(1).Greeting
        "#,
            filename
        );
        assert_prints!(input, "Hello world");
        std::fs::remove_file(filename).unwrap_or_default();
    }
}
