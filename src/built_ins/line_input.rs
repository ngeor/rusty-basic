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
        keyword_pair_p(Keyword::Line, Keyword::Input)
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after LINE INPUT"))
            .and_opt(expression::file_handle_comma_p())
            .and_demand(
                expression::expression_node_p()
                    .or_syntax_error("Expected: #file-number or variable"),
            )
            .map(|((_, opt_loc_file_handle), variable)| {
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
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
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
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
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
}
