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
        keyword_p(Keyword::Close)
            .and_opt(
                expression::guarded_file_handle_or_expression_p().and_opt(
                    item_p(',')
                        .surrounded_by_opt_ws()
                        .and(expression::file_handle_or_expression_p())
                        .keep_right()
                        .one_or_more(),
                ),
            )
            .keep_right()
            .map(|opt_first_and_remaining| {
                let mut args: ExpressionNodes = vec![];
                if let Some((first, opt_remaining)) = opt_first_and_remaining {
                    args.push(first);
                    args.extend(opt_remaining.unwrap_or_default());
                }
                Statement::BuiltInSubCall(BuiltInSub::Close, args)
            })
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
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
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::*;

    #[test]
    fn test_no_args() {
        let input = "CLOSE";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Close, vec![])
        );
    }

    #[test]
    fn test_one_file_number_no_hash() {
        let input = "CLOSE 1";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Close, vec![1.as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_one_file_number_no_hash_no_leading_space() {
        let input = "CLOSE1";
        let statement = parse(input).demand_single_statement();
        assert_eq!(statement, Statement::SubCall("CLOSE1".into(), vec![]));
    }

    #[test]
    fn test_one_file_number_no_hash_parenthesis_leading_space() {
        let input = "CLOSE (1)";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![Expression::Parenthesis(Box::new(1.as_lit_expr(1, 8))).at_rc(1, 7)]
            )
        );
    }

    #[test]
    fn test_one_file_number_no_hash_parenthesis_no_leading_space() {
        let input = "CLOSE(1)";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![Expression::Parenthesis(Box::new(1.as_lit_expr(1, 7))).at_rc(1, 6)]
            )
        );
    }

    #[test]
    fn test_one_file_number_with_hash() {
        let input = "CLOSE #1";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(BuiltInSub::Close, vec![1.as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_one_file_number_with_hash_no_leading_space() {
        let input = "CLOSE#1";
        assert_parser_err!(input, QError::syntax_error("No separator: #"), 1, 7);
    }

    #[test]
    fn test_one_file_number_with_hash_parenthesis_leading_space() {
        let input = "CLOSE (#1)";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: expression inside parenthesis"),
            1,
            8
        );
    }

    #[test]
    fn test_one_file_number_with_hash_parenthesis_no_leading_space() {
        let input = "CLOSE(#1)";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: expression inside parenthesis"),
            1,
            7
        );
    }

    #[test]
    fn test_two_file_number_no_hash_space_after_comma() {
        let input = "CLOSE 1, 2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 10)]
            )
        );
    }

    #[test]
    fn test_two_file_number_no_hash_space_before_comma() {
        let input = "CLOSE 1 ,2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 10)]
            )
        );
    }

    #[test]
    fn test_two_file_number_no_hash_space_around_comma() {
        let input = "CLOSE 1 , 2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 11)]
            )
        );
    }

    #[test]
    fn test_two_file_number_no_hash_no_space_around_comma() {
        let input = "CLOSE 1,2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 9)]
            )
        );
    }

    #[test]
    fn test_two_file_number_hash_space_after_comma() {
        let input = "CLOSE #1, #2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 11)]
            )
        );
    }

    #[test]
    fn test_two_file_number_hash_space_before_comma() {
        let input = "CLOSE #1 ,#2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 11)]
            )
        );
    }

    #[test]
    fn test_two_file_number_hash_space_around_comma() {
        let input = "CLOSE #1 , #2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 12)]
            )
        );
    }

    #[test]
    fn test_two_file_number_hash_no_space_around_comma() {
        let input = "CLOSE #1,#2";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Close,
                vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 10)]
            )
        );
    }

    #[test]
    fn test_close_inline_comment() {
        let input = "CLOSE #1 ' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::BuiltInSubCall(
                    BuiltInSub::Close,
                    vec![1.as_lit_expr(1, 7)]
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 10)
            ]
        );
    }
}
