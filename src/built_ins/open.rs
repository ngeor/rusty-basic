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
        keyword_p(Keyword::Open)
            .and_demand(
                expression::back_guarded_expression_node_p()
                    .or_syntax_error("Expected: file name after OPEN"),
            )
            .and_opt(parse_open_mode_p())
            .and_opt(parse_open_access_p())
            .and_demand(parse_file_number_p().or_syntax_error("Expected: AS file-number"))
            .and_opt(parse_len_p())
            .map(
                |(((((_, file_name), opt_file_mode), opt_file_access), file_number), opt_len)| {
                    Statement::BuiltInSubCall(
                        BuiltInSub::Open,
                        vec![
                            file_name,
                            map_opt_locatable_enum(opt_file_mode, FileMode::Random),
                            map_opt_locatable_enum(opt_file_access, FileAccess::Unspecified),
                            file_number,
                            map_opt_len(opt_len),
                        ],
                    )
                },
            )
    }

    // FOR <ws+> INPUT <ws+>
    fn parse_open_mode_p<R>() -> impl Parser<R, Output = Locatable<FileMode>>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::For)
            .and_demand(
                keyword_choice_p(&[
                    Keyword::Append,
                    Keyword::Input,
                    Keyword::Output,
                    Keyword::Random,
                ])
                .or_syntax_error("Expected: APPEND, INPUT or OUTPUT")
                .with_pos(),
            )
            .keep_right()
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after file mode"))
            .keep_left()
            .map(
                |Locatable {
                     element: (file_mode, _),
                     pos,
                 }| {
                    (match file_mode {
                        Keyword::Append => FileMode::Append,
                        Keyword::Input => FileMode::Input,
                        Keyword::Output => FileMode::Output,
                        Keyword::Random => FileMode::Random,
                        _ => panic!("Parser should not have parsed {}", file_mode),
                    })
                    .at(pos)
                },
            )
    }

    // ACCESS <ws+> READ <ws+>
    fn parse_open_access_p<R>() -> impl Parser<R, Output = Locatable<FileAccess>>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::Access)
            .and_demand(
                keyword_p(Keyword::Read)
                    .with_pos()
                    .or_syntax_error("Invalid file access"),
            )
            .keep_right()
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after file-access"))
            .keep_left()
            .map(|x| FileAccess::Read.at(x.pos()))
    }

    // AS <ws+> expression
    // AS ( expression )
    fn parse_file_number_p<R>() -> impl Parser<R, Output = ExpressionNode>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::As)
            .and_demand(
                expression::guarded_file_handle_or_expression_p()
                    .or_syntax_error("Expected: #file-number%"),
            )
            .keep_right()
    }

    fn parse_len_p<R>() -> impl Parser<R, Output = ExpressionNode>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        whitespace_p()
            .and(keyword_p(Keyword::Len))
            .and_demand(
                item_p('=')
                    .preceded_by_opt_ws()
                    .or_syntax_error("Expected: = after LEN"),
            )
            .and_demand(
                expression::guarded_expression_node_p()
                    .or_syntax_error("Expected: expression after LEN ="),
            )
            .keep_right()
    }

    fn map_opt_locatable_enum<T>(
        opt_locatable_enum: Option<Locatable<T>>,
        fallback: T,
    ) -> ExpressionNode
    where
        u8: From<T>,
    {
        opt_locatable_enum
            .map(|Locatable { element, pos }| u8_to_expr(element).at(pos))
            .unwrap_or_else(|| u8_to_expr(fallback).at(Location::start()))
    }

    fn u8_to_expr<T>(x: T) -> Expression
    where
        u8: From<T>,
    {
        Expression::IntegerLiteral(u8::from(x) as i32)
    }

    fn map_opt_len(opt_len: Option<ExpressionNode>) -> ExpressionNode {
        match opt_len {
            Some(expr) => expr,
            _ => Expression::IntegerLiteral(0).at(Location::start()),
        }
    }
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // must have 5 arguments:
        // filename
        // file mode
        // file access
        // file number
        // rec len
        if args.len() != 5 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        args.require_string_argument(0)?;
        for i in 1..args.len() {
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
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::*;

    #[test]
    fn test_open_for_input_access_read_as_file_handle_with_spaces() {
        let input = r#"OPEN "FILE.TXT" FOR INPUT ACCESS READ AS #1"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    "FILE.TXT".as_lit_expr(1, 6),
                    FILE_MODE_INPUT.as_lit_expr(1, 21),
                    FILE_ACCESS_READ.as_lit_expr(1, 34),
                    1.as_lit_expr(1, 42),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_for_input_access_read_as_file_handle_no_spaces() {
        let input = r#"OPEN("FILE.TXT")FOR INPUT ACCESS READ AS(1)"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    Expression::Parenthesis(Box::new("FILE.TXT".as_lit_expr(1, 6))).at_rc(1, 5),
                    FILE_MODE_INPUT.as_lit_expr(1, 21),
                    FILE_ACCESS_READ.as_lit_expr(1, 34),
                    Expression::Parenthesis(Box::new(1.as_lit_expr(1, 42))).at_rc(1, 41),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_for_input_as_file_handle_with_spaces() {
        let input = r#"OPEN "FILE.TXT" FOR INPUT AS #1"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    "FILE.TXT".as_lit_expr(1, 6),
                    FILE_MODE_INPUT.as_lit_expr(1, 21),
                    FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                    1.as_lit_expr(1, 30),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_access_read_as_file_handle_with_spaces() {
        let input = r#"OPEN "FILE.TXT" ACCESS READ AS #1"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    "FILE.TXT".as_lit_expr(1, 6),
                    FILE_MODE_RANDOM.as_lit_expr(1, 1),
                    FILE_ACCESS_READ.as_lit_expr(1, 24),
                    1.as_lit_expr(1, 32),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_as_file_handle_with_spaces() {
        let input = r#"OPEN "FILE.TXT" AS #1"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    "FILE.TXT".as_lit_expr(1, 6),
                    FILE_MODE_RANDOM.as_lit_expr(1, 1),
                    FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                    1.as_lit_expr(1, 20),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_as_number_with_spaces() {
        let input = r#"OPEN "FILE.TXT" AS 1"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    "FILE.TXT".as_lit_expr(1, 6),
                    FILE_MODE_RANDOM.as_lit_expr(1, 1),
                    FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                    Expression::IntegerLiteral(1).at_rc(1, 20),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_as_file_handle_no_spaces() {
        let input = r#"OPEN("FILE.TXT")AS(1)"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    Expression::Parenthesis(Box::new("FILE.TXT".as_lit_expr(1, 6))).at_rc(1, 5),
                    FILE_MODE_RANDOM.as_lit_expr(1, 1),
                    FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                    Expression::Parenthesis(Box::new(1.as_lit_expr(1, 20))).at_rc(1, 19),
                    0.as_lit_expr(1, 1) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_access_read_for_input_as_file_handle_with_spaces() {
        let input = r#"OPEN "FILE.TXT" ACCESS READ FOR INPUT AS #1"#;
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: AS file-number"),
            1,
            29
        );
    }

    #[test]
    fn test_open_random_explicit_len() {
        let input = r#"OPEN "A.TXT" FOR RANDOM AS #1 LEN = 64"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Open,
                vec![
                    "A.TXT".as_lit_expr(1, 6),
                    FILE_MODE_RANDOM.as_lit_expr(1, 18),
                    FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                    1.as_lit_expr(1, 28),
                    64.as_lit_expr(1, 37) // rec-len%
                ]
            )
        );
    }

    #[test]
    fn test_open_filename_must_be_string() {
        let program = "OPEN 42 AS #1";
        assert_linter_err!(program, QError::ArgumentTypeMismatch, 1, 6);
    }

    #[test]
    fn test_rec_len_must_be_numeric() {
        let program = r#"OPEN "a.txt" AS #1 LEN = "hi""#;
        assert_linter_err!(program, QError::ArgumentTypeMismatch, 1, 26);
    }
}
