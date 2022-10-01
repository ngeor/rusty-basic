pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword(Keyword::Open)
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
    fn parse_open_mode_p() -> impl Parser<Output = Locatable<FileMode>> {
        seq3(
            keyword_followed_by_whitespace_p(Keyword::For),
            keyword_map(&[
                (Keyword::Append, FileMode::Append),
                (Keyword::Input, FileMode::Input),
                (Keyword::Output, FileMode::Output),
                (Keyword::Random, FileMode::Random),
            ])
            .with_pos(),
            whitespace(),
            |_, file_mode, _| file_mode,
        )
    }

    // ACCESS <ws+> READ <ws+>
    fn parse_open_access_p() -> impl Parser<Output = Locatable<FileAccess>> {
        seq4(
            keyword(Keyword::Access),
            whitespace(),
            keyword(Keyword::Read).with_pos(),
            whitespace(),
            |_, _, Locatable { pos, .. }, _| FileAccess::Read.at(pos),
        )
    }

    // AS <ws+> expression
    // AS ( expression )
    fn parse_file_number_p() -> impl Parser<Output = ExpressionNode> {
        keyword(Keyword::As).then_use(
            expression::guarded_file_handle_or_expression_p()
                .or_syntax_error("Expected: #file-number%"),
        )
    }

    fn parse_len_p() -> impl Parser<Output = ExpressionNode> {
        seq3(
            whitespace().and(keyword(Keyword::Len)),
            equal_sign(),
            expression::expression_node_p().or_syntax_error("Expected: expression after LEN ="),
            |_, _, e| e,
        )
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
    use crate::common::{FileAccess, FileHandle, FileMode, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;
    use crate::variant::{QBNumberCast, Variant};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let file_name: String = interpreter.context()[0].to_str_unchecked().to_owned(); // TODO fighting borrow checker
        let file_mode: FileMode = to_file_mode(&interpreter.context()[1]);
        let file_access: FileAccess = to_file_access(&interpreter.context()[2]);
        let file_handle: FileHandle = interpreter.context()[3].to_file_handle()?;
        let rec_len: usize = to_record_length(&interpreter.context()[4])?;
        interpreter
            .file_manager()
            .open(file_handle, &file_name, file_mode, file_access, rec_len)
    }

    fn to_file_mode(v: &Variant) -> FileMode {
        let i: i32 = v
            .try_cast()
            .expect("Internal FileMode argument should be valid");
        FileMode::from(i as u8)
    }

    fn to_file_access(v: &Variant) -> FileAccess {
        let i: i32 = v
            .try_cast()
            .expect("Internal FileAccess argument should be valid");
        FileAccess::from(i as u8)
    }

    fn to_record_length(v: &Variant) -> Result<usize, QError> {
        let i: i32 = v.try_cast()?;
        if i < 0 {
            // TODO make 0 invalid too, now 0 means no value. Afterwards, use VariantCasts trait.
            Err(QError::BadRecordLength)
        } else {
            Ok(i as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::assert_parser_err;
    use crate::assert_prints;
    use crate::built_ins::BuiltInSub;
    use crate::common::QError;
    use crate::common::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::*;
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

    fn read_and_remove(filename: &str) -> String {
        let contents = std::fs::read_to_string(filename).unwrap_or_default();
        std::fs::remove_file(filename).unwrap_or_default();
        contents
    }

    #[test]
    fn test_can_create_file() {
        std::fs::remove_file("TEST1.TXT").unwrap_or(());
        let input = r#"
        OPEN "TEST1.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        CLOSE #1
        "#;
        interpret(input);
        let contents = read_and_remove("TEST1.TXT");
        assert_eq!("Hello, world\r\n", contents);
    }

    #[test]
    fn test_can_read_file() {
        let input = r#"
        OPEN "TEST2A.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        CLOSE #1
        OPEN "TEST2A.TXT" FOR INPUT AS #1
        LINE INPUT #1, T$
        CLOSE #1
        OPEN "TEST2B.TXT" FOR APPEND AS #1
        PRINT #1, T$
        CLOSE #1
        "#;
        interpret(input);
        let contents = read_and_remove("TEST2B.TXT");
        std::fs::remove_file("TEST2A.TXT").unwrap_or(());
        std::fs::remove_file("TEST2B.TXT").unwrap_or(());
        assert_eq!("Hello, world\r\n", contents);
    }

    #[test]
    fn test_can_read_file_until_eof() {
        std::fs::remove_file("TEST3.TXT").unwrap_or(());
        let input = r#"
        OPEN "TEST3.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        PRINT #1, "Hello, again"
        CLOSE #1
        OPEN "TEST3.TXT" FOR INPUT AS #1
        WHILE NOT EOF(1)
        LINE INPUT #1, T$
        PRINT T$
        WEND
        CLOSE #1
        "#;
        assert_prints!(input, "Hello, world", "Hello, again");
        std::fs::remove_file("TEST3.TXT").unwrap_or(());
    }

    #[test]
    fn test_can_write_file_append_mode() {
        std::fs::remove_file("test_can_write_file_append_mode.TXT").unwrap_or(());
        let input = r#"
        OPEN "test_can_write_file_append_mode.TXT" FOR APPEND AS #1
        PRINT #1, "Hello, world"
        PRINT #1, "Hello, again"
        CLOSE #1
        "#;
        interpret(input);
        let read_result = read_and_remove("test_can_write_file_append_mode.TXT");
        assert_eq!(read_result, "Hello, world\r\nHello, again\r\n");
    }

    #[test]
    fn test_open_bad_file_number_runtime_error() {
        let input = r#"
        A = 256
        OPEN "TEST.TXT" FOR INPUT AS A
        CLOSE A
        "#;
        assert_interpreter_err!(input, QError::BadFileNameOrNumber, 3, 9);
    }

    #[test]
    fn test_open_twice_without_closing_error() {
        let input = r#"
        OPEN "a.txt" FOR OUTPUT AS #1
        OPEN "a.txt" FOR OUTPUT AS #1
        "#;
        assert_interpreter_err!(input, QError::FileAlreadyOpen, 3, 9);
        std::fs::remove_file("a.txt").unwrap_or(());
    }

    #[test]
    fn open_random_file_field_lset_put() {
        let input = r#"
        OPEN "rnd1.txt" FOR RANDOM AS #1 LEN = 64
        FIELD #1, 10 AS FirstName$, 20 AS LastName$
        LSET FirstName$ = "Nikos"
        LSET LastName$ = "Georgiou"
        PUT #1, 1
        CLOSE
        "#;
        interpret(input);
        let contents = read_and_remove("rnd1.txt");
        assert_eq!(contents, "Nikos\0\0\0\0\0Georgiou\0\0\0\0\0\0\0\0\0\0\0\0");
    }

    #[test]
    fn open_random_file_field_lset_put_get() {
        let input = r#"
        OPEN "rnd2.txt" FOR RANDOM AS #1 LEN = 15
        FIELD #1, 10 AS FirstName$, 5 AS LastName$
        LSET FirstName$ = "Nikos"
        LSET LastName$ = "Georgiou"
        PUT #1, 1
        LSET FirstName$ = "Someone"
        LSET LastName$ = "Else"
        PUT #1, 2
        GET #1, 1
        PRINT FirstName$; LastName$
        CLOSE
        "#;
        assert_prints!(input, "Nikos\0\0\0\0\0Georg");
        let contents = read_and_remove("rnd2.txt");
        assert_eq!(contents, "Nikos\0\0\0\0\0GeorgSomeone\0\0\0Else\0");
    }
}
