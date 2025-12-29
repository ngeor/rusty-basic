use crate::input::RcStringView;
use crate::pc::*;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::BuiltInSub;
use crate::ParseError;
use rusty_common::*;
pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq6(
        keyword(Keyword::Open),
        ws_expr_pos_ws_p().or_syntax_error("Expected: file name after OPEN"),
        parse_open_mode_p().to_option(),
        parse_open_access_p().to_option(),
        parse_file_number_p().or_syntax_error("Expected: AS file-number"),
        parse_len_p().to_option(),
        |_, file_name, opt_file_mode, opt_file_access, file_number, opt_len| {
            Statement::built_in_sub_call(
                BuiltInSub::Open,
                vec![
                    file_name,
                    map_opt_positioned_enum(opt_file_mode, FileMode::Random),
                    map_opt_positioned_enum(opt_file_access, FileAccess::Unspecified),
                    file_number,
                    map_opt_len(opt_len),
                ],
            )
        },
    )
}

// FOR <ws+> INPUT <ws+>
fn parse_open_mode_p(
) -> impl Parser<RcStringView, Output = Positioned<FileMode>, Error = ParseError> {
    seq4(
        keyword(Keyword::For),
        whitespace(),
        keyword_map(&[
            (Keyword::Append, FileMode::Append),
            (Keyword::Input, FileMode::Input),
            (Keyword::Output, FileMode::Output),
            (Keyword::Random, FileMode::Random),
        ])
        .with_pos(),
        whitespace(),
        |_, _, file_mode, _| file_mode,
    )
}

// ACCESS <ws+> READ <ws+>
fn parse_open_access_p(
) -> impl Parser<RcStringView, Output = Positioned<FileAccess>, Error = ParseError> {
    seq4(
        keyword(Keyword::Access),
        whitespace(),
        keyword(Keyword::Read).with_pos(),
        whitespace(),
        |_, _, positioned, _| FileAccess::Read.at(&positioned),
    )
}

// AS <ws+> expression
// AS ( expression )
fn parse_file_number_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    keyword(Keyword::As).and_keep_right(
        guarded_file_handle_or_expression_p().or_syntax_error("Expected: #file-number%"),
    )
}

fn parse_len_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    seq3(
        whitespace().and_tuple(keyword(Keyword::Len)),
        equal_sign(),
        expression_pos_p().or_syntax_error("Expected: expression after LEN ="),
        |_, _, e| e,
    )
}

fn map_opt_positioned_enum<T>(
    opt_positioned_enum: Option<Positioned<T>>,
    fallback: T,
) -> ExpressionPos
where
    u8: From<T>,
{
    opt_positioned_enum
        .map(|positioned| positioned.map(u8_to_expr))
        .unwrap_or_else(|| u8_to_expr(fallback).at_pos(Position::start()))
}

fn u8_to_expr<T>(x: T) -> Expression
where
    u8: From<T>,
{
    Expression::IntegerLiteral(u8::from(x) as i32)
}

fn map_opt_len(opt_len: Option<ExpressionPos>) -> ExpressionPos {
    opt_len.unwrap_or_else(|| Expression::IntegerLiteral(0).at_pos(Position::start()))
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
    fn test_open_for_input_access_read_as_file_handle_with_spaces() {
        let input = r#"OPEN "FILE.TXT" FOR INPUT ACCESS READ AS #1"#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::built_in_sub_call(
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
            Statement::built_in_sub_call(
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
            Statement::built_in_sub_call(
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
            Statement::built_in_sub_call(
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
            Statement::built_in_sub_call(
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
            Statement::built_in_sub_call(
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
            Statement::built_in_sub_call(
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
            ParseError::syntax_error("Expected: AS file-number"),
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
            Statement::built_in_sub_call(
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
}
