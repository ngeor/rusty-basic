pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
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
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_built_in_sub_call;
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::*;

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
}
