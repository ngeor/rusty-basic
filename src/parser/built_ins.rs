use crate::built_ins::BuiltInSub;
use crate::common::*;
use crate::parser::expression;
use crate::parser::pc::*;
use crate::parser::pc_specific::{keyword_p, PcSpecific};
use crate::parser::types::*;

/// Parses built-in subs which have a special syntax.
pub fn parse_built_in_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    close::parse_close_p()
        .or(input::parse_input_p())
        .or(line_input::parse_line_input_p())
        .or(name::parse_name_p())
        .or(open::parse_open_p())
        .or(field::parse_field_p())
        .or(lset::parse_lset_p())
        .or(get::parse_get_p())
        .or(put::parse_put_p())
}

mod close {
    use super::*;

    pub fn parse_close_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::Close)
            .and_opt(
                guarded_file_handle_or_expression_p().and_opt(
                    item_p(',')
                        .surrounded_by_opt_ws()
                        .and(file_handle_or_expression_p())
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_parser_err;
        use crate::parser::test_utils::*;

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
            assert_parser_err!(input, QError::syntax_error("No separator: ("), 1, 7);
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
}

mod input {

    use super::*;
    use crate::parser::pc_specific::keyword_followed_by_whitespace_p;

    pub fn parse_input_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        // INPUT variable-list
        // LINE INPUT variable$
        // INPUT #file-number%, variable-list
        // LINE INPUT #file-number%, variable$
        keyword_followed_by_whitespace_p(Keyword::Input)
            .and_opt(file_handle_comma_p())
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

    #[cfg(test)]
    mod tests {
        use crate::assert_built_in_sub_call;
        use crate::assert_parser_err;
        use crate::parser::test_utils::*;

        use super::*;

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
    }
}

mod line_input {
    use super::*;
    use crate::parser::pc_specific::keyword_pair_p;

    pub fn parse_line_input_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_pair_p(Keyword::Line, Keyword::Input)
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after LINE INPUT"))
            .and_opt(file_handle_comma_p())
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

    #[cfg(test)]
    mod tests {
        use crate::assert_built_in_sub_call;
        use crate::assert_parser_err;
        use crate::parser::test_utils::*;

        use super::*;

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
}

mod name {
    use crate::parser::expression::{back_guarded_expression_node_p, guarded_expression_node_p};

    use super::*;

    pub fn parse_name_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::Name)
            .and_demand(back_guarded_expression_node_p().or_syntax_error("Expected: old file name"))
            .and_demand(keyword_p(Keyword::As).or_syntax_error("Expected: AS"))
            .keep_middle()
            .and_demand(guarded_expression_node_p().or_syntax_error("Expected: new file name"))
            .map(|(l, r)| Statement::BuiltInSubCall(BuiltInSub::Name, vec![l, r]))
    }
}

mod open {
    use super::*;
    use crate::parser::expression::guarded_expression_node_p;
    use crate::parser::pc_specific::{keyword_choice_p, keyword_followed_by_whitespace_p};

    pub fn parse_open_p<R>() -> impl Parser<R, Output = Statement>
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
                guarded_file_handle_or_expression_p().or_syntax_error("Expected: #file-number%"),
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
                guarded_expression_node_p().or_syntax_error("Expected: expression after LEN ="),
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_parser_err;
        use crate::parser::test_utils::*;

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
    }
}

/// Parses a file handle ( e.g. `#1` ) as an integer literal expression.
fn file_handle_as_expression_node_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::file_handle_p()
        .map(|Locatable { element, pos }| Expression::IntegerLiteral(element.into()).at(pos))
}

fn file_handle_comma_p<R>() -> impl Parser<R, Output = Locatable<FileHandle>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::file_handle_p()
        .and_demand(
            item_p(',')
                .surrounded_by_opt_ws()
                .or_syntax_error("Expected: ,"),
        )
        .keep_left()
}

fn file_handle_or_expression_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    file_handle_as_expression_node_p().or(expression::expression_node_p())
}

fn guarded_file_handle_or_expression_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(file_handle_as_expression_node_p())
        .keep_right()
        .or(expression::guarded_expression_node_p())
}

mod field {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::expression::{expression_node_p, file_handle_p};
    use crate::parser::name::name_with_dot_p;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::types::*;

    pub fn parse_field_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::Field)
            .and_demand(field_node_p().or_syntax_error("Expected: file number after FIELD"))
            .keep_right()
    }

    fn field_node_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        whitespace_p()
            .and_demand(file_handle_p().or_syntax_error("Expected: file-number"))
            .and_demand(
                item_p(',')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ,"),
            )
            .and_demand(
                field_item_p()
                    .csv()
                    .or_syntax_error("Expected: field width"),
            )
            .map(|(((_, file_number), _), fields)| {
                Statement::BuiltInSubCall(BuiltInSub::Field, build_args(file_number, fields))
            })
    }

    fn field_item_p<R>() -> impl Parser<R, Output = (ExpressionNode, NameNode)>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        expression_node_p()
            // TODO 'AS' does not need leading whitespace if expression has parenthesis
            // TODO solve this not by peeking the previous but with a new expression:: function
            .and_demand(
                keyword_p(Keyword::As)
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: AS"),
            )
            .and_demand(
                name_with_dot_p()
                    .with_pos()
                    .or_syntax_error("Expected: variable name"),
            )
            .map(|((width, _), name)| (width, name))
    }

    fn build_args(
        file_number: Locatable<FileHandle>,
        fields: Vec<(ExpressionNode, NameNode)>,
    ) -> ExpressionNodes {
        let mut args: ExpressionNodes = vec![];
        args.push(file_number.map(|x| Expression::IntegerLiteral(x.into())));
        for (width, Locatable { element: name, pos }) in fields {
            args.push(width);
            let variable_name: String = name.bare_name().as_ref().to_string();
            args.push(Expression::StringLiteral(variable_name).at(pos));
            // to lint the variable, not used at runtime
            args.push(Expression::Variable(name, VariableInfo::unresolved()).at(pos));
        }
        args
    }
}

mod lset {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::expression::expression_node_p;
    use crate::parser::name::name_with_dot_p;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::types::*;

    pub fn parse_lset_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::LSet)
            .and_demand(
                name_with_dot_p()
                    .with_pos()
                    .or_syntax_error("Expected: variable after LSET"),
            )
            .and_demand(
                item_p('=')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ="),
            )
            .and_demand(expression_node_p().or_syntax_error("Expected: expression"))
            .map(|(((_, name_node), _), value_expr_node)| {
                Statement::BuiltInSubCall(BuiltInSub::LSet, build_args(name_node, value_expr_node))
            })
    }

    fn build_args(name_node: NameNode, value_expr_node: ExpressionNode) -> ExpressionNodes {
        let Locatable { element: name, pos } = name_node;
        let variable_name: String = name.bare_name().as_ref().to_owned();
        let name_expr_node = Expression::Variable(name, VariableInfo::unresolved()).at(pos);
        vec![
            // pass the name of the variable as a special argument
            Expression::StringLiteral(variable_name).at(pos),
            // pass the variable itself (ByRef) to be able to write to it
            name_expr_node,
            // pass the value to assign to the variable
            value_expr_node,
        ]
    }
}

mod get {
    use super::*;
    use crate::built_ins::BuiltInSub;
    use crate::parser::expression;
    use crate::parser::pc_specific::keyword_followed_by_whitespace_p;

    pub fn parse_get_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::Get)
            .and_demand(expression::file_handle_p().or_syntax_error("Expected: file-number"))
            .and_demand(
                item_p(',')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ,"),
            )
            .and_demand(expression::expression_node_p().or_syntax_error("Expected: record-number"))
            .map(|(((_, file_number), _), r)| {
                Statement::BuiltInSubCall(
                    BuiltInSub::Get,
                    vec![file_number.map(|x| Expression::IntegerLiteral(x.into())), r],
                )
            })
    }
}

mod put {
    use super::*;
    use crate::built_ins::BuiltInSub;
    use crate::parser::expression;
    use crate::parser::pc_specific::keyword_followed_by_whitespace_p;

    pub fn parse_put_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::Put)
            .and_demand(expression::file_handle_p().or_syntax_error("Expected: file-number"))
            .and_demand(
                item_p(',')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ,"),
            )
            .and_demand(expression::expression_node_p().or_syntax_error("Expected: record-number"))
            .map(|(((_, file_number), _), r)| {
                Statement::BuiltInSubCall(
                    BuiltInSub::Put,
                    vec![file_number.map(|x| Expression::IntegerLiteral(x.into())), r],
                )
            })
    }
}
