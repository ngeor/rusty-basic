pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::expression::expression_node_p;
    use crate::parser::expression::file_handle::{
        file_handle_as_expression_node_p, guarded_file_handle_or_expression_p,
    };
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    // <result> ::= <CLOSE> | <CLOSE><file_handles>
    // file_handles ::= <first_file_handle> | <first_file_handle> <opt-ws> "," <opt-ws> <next_file_handles>
    // next_file_handles ::= <file_handle> | <file_handle> <opt-ws> "," <opt-ws> <next_file_handles>
    // first_file_handle ::= "(" <file_handle> ")" | <ws> <file_handle>
    // file_handle ::= "#" <digits> | <expr>
    pub fn parse() -> impl Parser<Output = Statement> {
        seq2(
            keyword(Keyword::Close),
            file_handles(),
            |_, file_handles| Statement::BuiltInSubCall(BuiltInSub::Close, file_handles),
        )
    }

    fn file_handles() -> impl Parser<Output = ExpressionNodes> + NonOptParser {
        AccumulateParser::new(
            guarded_file_handle_or_expression_p(),
            comma().then_demand(file_handle_or_expression_p()),
        )
        .allow_default()
    }

    fn file_handle_or_expression_p() -> impl Parser<Output = ExpressionNode> + NonOptParser {
        file_handle_as_expression_node_p()
            .or(expression_node_p())
            .or_syntax_error("Expected: file handle")
    }
}

pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}

pub mod interpreter {
    use crate::common::{FileHandle, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let file_handles: Vec<FileHandle> = interpreter
            .context()
            .variables()
            .iter()
            .map(VariantCasts::to_file_handle)
            .collect::<Result<Vec<FileHandle>, QError>>()?;
        if file_handles.is_empty() {
            interpreter.file_manager().close_all();
        } else {
            for file_handle in file_handles {
                interpreter.file_manager().close(&file_handle);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::interpreter::test_utils::*;
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

    #[test]
    fn test_close_not_opened_file_is_allowed() {
        interpret("CLOSE 1");
    }

    #[test]
    fn test_close_allows_to_open_again() {
        let input = r#"
        OPEN "a.txt" FOR OUTPUT AS #1
        CLOSE #1
        OPEN "a.txt" FOR OUTPUT AS #1
        CLOSE #1
        "#;
        interpret(input);
        std::fs::remove_file("a.txt").unwrap_or(());
    }

    #[test]
    fn test_close_without_args_closes_all_files() {
        let input = r#"
        OPEN "a.txt" FOR OUTPUT AS #1
        OPEN "b.txt" FOR OUTPUT AS #2
        CLOSE
        OPEN "a.txt" FOR OUTPUT AS #1
        OPEN "b.txt" FOR OUTPUT AS #2
        CLOSE
        "#;
        interpret(input);
        std::fs::remove_file("a.txt").unwrap_or(());
        std::fs::remove_file("b.txt").unwrap_or(());
    }
}
