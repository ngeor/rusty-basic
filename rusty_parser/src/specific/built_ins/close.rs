use crate::pc::*;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::specific::{file_handle_as_expression_pos_p, guarded_file_handle_or_expression_p};
use crate::BuiltInSub;

// <result> ::= <CLOSE> | <CLOSE><file_handles>
// file_handles ::= <first_file_handle> | <first_file_handle> <opt-ws> "," <opt-ws> <next_file_handles>
// next_file_handles ::= <file_handle> | <file_handle> <opt-ws> "," <opt-ws> <next_file_handles>
// first_file_handle ::= "(" <file_handle> ")" | <ws> <file_handle>
// file_handle ::= "#" <digits> | <expr>
pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    seq2(
        keyword(Keyword::Close),
        file_handles(),
        |_, file_handles| Statement::BuiltInSubCall(BuiltInSub::Close, file_handles),
    )
}

fn file_handles() -> impl Parser<RcStringView, Output = Expressions> {
    AccumulateParser::new(
        guarded_file_handle_or_expression_p(),
        comma().and_without_undo_keep_right(file_handle_or_expression_p()),
    )
    .or_default()
}

fn file_handle_or_expression_p() -> impl Parser<RcStringView, Output = ExpressionPos> {
    OrParser::new(vec![
        Box::new(file_handle_as_expression_pos_p()),
        Box::new(expression_pos_p()),
    ])
    .or_syntax_error("Expected: file handle")
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::error::ParseError;
    use crate::specific::*;
    use crate::test_utils::*;
    use crate::BuiltInSub;
    use crate::*;
    use rusty_common::*;

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
        assert_parser_err!(input, ParseError::syntax_error("No separator: #"), 1, 7);
    }

    #[test]
    fn test_one_file_number_with_hash_parenthesis_leading_space() {
        let input = "CLOSE (#1)";
        assert_parser_err!(
            input,
            ParseError::syntax_error("Expected: expression inside parenthesis"),
            1,
            8
        );
    }

    #[test]
    fn test_one_file_number_with_hash_parenthesis_no_leading_space() {
        let input = "CLOSE(#1)";
        assert_parser_err!(
            input,
            ParseError::syntax_error("Expected: expression inside parenthesis"),
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
                GlobalStatement::Statement(Statement::BuiltInSubCall(
                    BuiltInSub::Close,
                    vec![1.as_lit_expr(1, 7)]
                ))
                .at_rc(1, 1),
                GlobalStatement::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 10)
            ]
        );
    }
}
