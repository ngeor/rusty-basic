use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Parses built-in subs which have a special syntax.
pub fn parse_built_in<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, crate::parser::Statement, QError>> {
    or_vec(vec![
        input::parse_input(),
        line_input::parse_line_input(),
        name::parse_name(),
        open::parse_open(),
    ])
}

mod input {
    use super::*;
    pub fn parse_input<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(parse_input_args(), |r| {
            Statement::SubCall("INPUT".into(), r)
        })
    }

    pub fn parse_input_args<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Vec<ExpressionNode>, QError>> {
        drop_left(crate::parser::pc::ws::seq2(
            keyword(Keyword::Input),
            demand(
                map_default_to_not_found(csv_zero_or_more(expression::expression_node())),
                QError::syntax_error_fn("Expected: at least one variable"),
            ),
            QError::syntax_error_fn("Expected: whitespace after INPUT"),
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_sub_call;
        use crate::parser::test_utils::*;

        #[test]
        fn test_parse_one_variable() {
            let input = "INPUT A$";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(result, "INPUT", Expression::VariableName("A$".into()));
        }

        #[test]
        fn test_parse_two_variables() {
            let input = "INPUT A$, B";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
                Expression::VariableName("A$".into()),
                Expression::VariableName("B".into())
            );
        }

        #[test]
        fn test_no_whitespace_after_input() {
            let input = "INPUT";
            assert_eq!(
                parse_err(input),
                QError::syntax_error("Expected: whitespace after INPUT")
            );
        }

        #[test]
        fn test_no_variable() {
            let input = "INPUT ";
            assert_eq!(
                parse_err(input),
                QError::syntax_error("Expected: at least one variable")
            );
        }
    }
}

mod line_input {
    use super::*;
    pub fn parse_line_input<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(
            crate::parser::pc::ws::seq2(
                keyword(Keyword::Line),
                demand(
                    super::input::parse_input_args(),
                    QError::syntax_error_fn("Expected: INPUT after LINE"),
                ),
                QError::syntax_error_fn("Expected: whitespace after LINE"),
            ),
            |(_, r)| Statement::SubCall("LINE INPUT".into(), r),
        )
    }
}

mod name {
    use super::*;
    pub fn parse_name<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(
            seq4(
                keyword(Keyword::Name),
                expression::demand_back_guarded_expression_node(),
                demand_keyword(Keyword::As),
                expression::demand_guarded_expression_node(),
            ),
            |(_, l, _, r)| Statement::SubCall("NAME".into(), vec![l, r]),
        )
    }
}

mod open {
    use super::*;

    pub fn parse_open<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(
            crate::parser::pc::common::seq4(
                // OPEN
                keyword(Keyword::Open),
                // <ws+>file-name<ws+> OR ( file-name )
                expression::demand_back_guarded_expression_node(),
                // (FOR INPUT ACCESS READ OR FOR INPUT OR ACCESS READ)<ws+> OR nothing
                parse_mode_access(),
                // AS <ws+> #1 OR AS <ws+> 1 OR AS(#1) OR AS(1)
                demand(
                    parse_file_number(),
                    QError::syntax_error_fn("Expected: AS file-number"),
                ),
            ),
            |(_, file_name, (opt_file_mode, opt_file_access), file_number)| {
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        file_name,
                        map_opt_locatable_enum(opt_file_mode, FileMode::Input),
                        map_opt_locatable_enum(opt_file_access, FileAccess::Unspecified),
                        file_number,
                    ],
                )
            },
        )
    }

    // FOR <ws+> INPUT <ws+> ACCESS <ws+> READ <ws+>
    // FOR <ws+> INPUT <ws+>
    // ACCESS <ws+> READ <ws+>
    // or nothing
    fn parse_mode_access<T: BufRead + 'static>() -> Box<
        dyn Fn(
            EolReader<T>,
        ) -> ReaderResult<
            EolReader<T>,
            (Option<Locatable<FileMode>>, Option<Locatable<FileAccess>>),
            QError,
        >,
    > {
        or_vec(vec![
            map(
                opt_seq2(parse_open_mode(), parse_open_access()),
                |(m, opt_a)| (Some(m), opt_a),
            ),
            map(parse_open_access(), |a| (None, Some(a))),
            Box::new(|reader: EolReader<T>| Ok((reader, Some((None, None))))),
        ])
    }

    fn map_opt_locatable_enum<T>(
        opt_locatable_enum: Option<Locatable<T>>,
        fallback: T,
    ) -> ExpressionNode
    where
        i32: From<T>,
    {
        opt_locatable_enum
            .map(|Locatable { element, pos }| Expression::IntegerLiteral(element.into()).at(pos))
            .unwrap_or_else(|| Expression::IntegerLiteral(fallback.into()).at(Location::start()))
    }

    // FOR <ws+> INPUT <ws+>
    fn parse_open_mode<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<FileMode>, QError>> {
        crate::parser::pc::ws::one_or_more_trailing(
            drop_left(crate::parser::pc::ws::seq2(
                keyword(Keyword::For),
                with_pos(demand(
                    or_vec(vec![
                        map(keyword(Keyword::Append), |_| FileMode::Append),
                        map(keyword(Keyword::Input), |_| FileMode::Input),
                        map(keyword(Keyword::Output), |_| FileMode::Output),
                    ]),
                    QError::syntax_error_fn("Invalid file mode"),
                )),
                QError::syntax_error_fn("Expected: whitespace after FOR"),
            )),
            QError::syntax_error_fn("Expected: space after file mode"),
        )
    }

    // ACCESS <ws+> READ <ws+>
    fn parse_open_access<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<FileAccess>, QError>>
    {
        map(
            crate::parser::pc::common::seq4(
                keyword(Keyword::Access),
                demand(
                    crate::parser::pc::ws::one_or_more(),
                    QError::syntax_error_fn("Expected: whitespace after ACCESS"),
                ),
                with_pos(demand(
                    map(keyword(Keyword::Read), |_| FileAccess::Read),
                    QError::syntax_error_fn("Invalid file access"),
                )),
                demand(
                    crate::parser::pc::ws::one_or_more(),
                    QError::syntax_error_fn("Expected: whitespace after file-access"),
                ),
            ),
            |(_, _, a, _)| a,
        )
    }

    // AS <ws+> expression
    // AS ( expression )
    fn parse_file_number<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        drop_left(crate::parser::pc::common::seq2(
            keyword(Keyword::As),
            expression::demand_guarded_expression_node(),
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::parser::parse_main_str;
        use crate::parser::test_utils::*;

        #[test]
        fn test_open_for_input_access_read_as_file_handle_with_spaces() {
            let input = r#"OPEN "FILE.TXT" FOR INPUT ACCESS READ AS #1"#;
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        "FILE.TXT".as_lit_expr(1, 6),
                        1.as_lit_expr(1, 21),
                        1.as_lit_expr(1, 34),
                        Expression::FileHandle(1.into()).at_rc(1, 42)
                    ]
                )
            );
        }

        #[test]
        fn test_open_for_input_access_read_as_file_handle_no_spaces() {
            let input = r#"OPEN("FILE.TXT")FOR INPUT ACCESS READ AS(#1)"#;
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        Expression::Parenthesis(Box::new("FILE.TXT".as_lit_expr(1, 6))).at_rc(1, 5),
                        1.as_lit_expr(1, 21),
                        1.as_lit_expr(1, 34),
                        Expression::Parenthesis(Box::new(
                            Expression::FileHandle(1.into()).at_rc(1, 42)
                        ))
                        .at_rc(1, 41)
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        "FILE.TXT".as_lit_expr(1, 6),
                        1.as_lit_expr(1, 21),
                        0.as_lit_expr(1, 1),
                        Expression::FileHandle(1.into()).at_rc(1, 30)
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        "FILE.TXT".as_lit_expr(1, 6),
                        1.as_lit_expr(1, 1),
                        1.as_lit_expr(1, 24),
                        Expression::FileHandle(1.into()).at_rc(1, 32)
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        "FILE.TXT".as_lit_expr(1, 6),
                        1.as_lit_expr(1, 1),
                        0.as_lit_expr(1, 1),
                        Expression::FileHandle(1.into()).at_rc(1, 20)
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        "FILE.TXT".as_lit_expr(1, 6),
                        1.as_lit_expr(1, 1),
                        0.as_lit_expr(1, 1),
                        Expression::IntegerLiteral(1).at_rc(1, 20)
                    ]
                )
            );
        }

        #[test]
        fn test_open_as_file_handle_no_spaces() {
            let input = r#"OPEN("FILE.TXT")AS(#1)"#;
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        Expression::Parenthesis(Box::new("FILE.TXT".as_lit_expr(1, 6))).at_rc(1, 5),
                        1.as_lit_expr(1, 1),
                        0.as_lit_expr(1, 1),
                        Expression::Parenthesis(Box::new(
                            Expression::FileHandle(1.into()).at_rc(1, 20)
                        ))
                        .at_rc(1, 19)
                    ]
                )
            );
        }

        #[test]
        fn test_open_access_read_for_input_as_file_handle_with_spaces() {
            let input = r#"OPEN "FILE.TXT" ACCESS READ FOR INPUT AS #1"#;
            assert_eq!(
                parse_main_str(input).unwrap_err(),
                QErrorNode::Pos(
                    QError::syntax_error("Expected: AS file-number"),
                    Location::new(1, 29)
                )
            );
        }
    }
}
