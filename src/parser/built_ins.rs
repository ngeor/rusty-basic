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
        .or(print::parse_print_p())
        .or(print::parse_lprint_p())
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
                Statement::SubCall(BuiltInSub::Close.into(), args)
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
            assert_eq!(statement, Statement::SubCall("CLOSE".into(), vec![]));
        }

        #[test]
        fn test_one_file_number_no_hash() {
            let input = "CLOSE 1";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::SubCall("CLOSE".into(), vec![1.as_lit_expr(1, 7)])
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall("CLOSE".into(), vec![1.as_lit_expr(1, 7)])
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
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
                Statement::SubCall(
                    "CLOSE".into(),
                    vec![1.as_lit_expr(1, 7), 2.as_lit_expr(1, 10)]
                )
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
            .and_opt(parse_file_number_p())
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
                Statement::SubCall(BuiltInSub::Input.into(), args)
            })
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::assert_sub_call;
        use crate::parser::test_utils::*;

        use super::*;

        #[test]
        fn test_parse_one_variable() {
            let input = "INPUT A$";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
                Expression::IntegerLiteral(0), // no file number
                Expression::var_unresolved("A$")
            );
        }

        #[test]
        fn test_parse_two_variables() {
            let input = "INPUT A$, B";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
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
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
                Expression::IntegerLiteral(1), // has file number
                Expression::IntegerLiteral(1), // file number
                Expression::var_unresolved("A")
            );
        }

        #[test]
        fn test_file_hash_one_variable_no_comma() {
            let input = "INPUT #2,A";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
                Expression::IntegerLiteral(1), // has file number
                Expression::IntegerLiteral(2), // file number
                Expression::var_unresolved("A")
            );
        }

        #[test]
        fn test_file_hash_one_variable_space_before_comma() {
            let input = "INPUT #3 ,A";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
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
            .and_opt(parse_file_number_p())
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
                Statement::SubCall(BuiltInSub::LineInput.into(), args)
            })
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::assert_sub_call;
        use crate::parser::test_utils::*;

        use super::*;

        #[test]
        fn test_parse_one_variable() {
            let input = "LINE INPUT A$";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "LINE INPUT",
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
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "LINE INPUT",
                Expression::IntegerLiteral(1), // has file number
                Expression::IntegerLiteral(1), // file number
                Expression::var_unresolved("A")
            );
        }

        #[test]
        fn test_file_hash_one_variable_no_comma() {
            let input = "LINE INPUT #2,A";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "LINE INPUT",
                Expression::IntegerLiteral(1), // has file number
                Expression::IntegerLiteral(2), // file number
                Expression::var_unresolved("A")
            );
        }

        #[test]
        fn test_file_hash_one_variable_space_before_comma() {
            let input = "LINE INPUT #1 ,A";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "LINE INPUT",
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
            .map(|(l, r)| Statement::SubCall(BuiltInSub::Name.into(), vec![l, r]))
    }
}

mod open {
    use super::*;
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
            .map(
                |((((_, file_name), opt_file_mode), opt_file_access), file_number)| {
                    Statement::SubCall(
                        BuiltInSub::Open.into(),
                        vec![
                            file_name,
                            map_opt_locatable_enum(opt_file_mode, FileMode::Random),
                            map_opt_locatable_enum(opt_file_access, FileAccess::Unspecified),
                            file_number,
                        ],
                    )
                },
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        "FILE.TXT".as_lit_expr(1, 6),
                        FILE_MODE_INPUT.as_lit_expr(1, 21),
                        FILE_ACCESS_READ.as_lit_expr(1, 34),
                        1.as_lit_expr(1, 42)
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        Expression::Parenthesis(Box::new("FILE.TXT".as_lit_expr(1, 6))).at_rc(1, 5),
                        FILE_MODE_INPUT.as_lit_expr(1, 21),
                        FILE_ACCESS_READ.as_lit_expr(1, 34),
                        Expression::Parenthesis(Box::new(1.as_lit_expr(1, 42))).at_rc(1, 41)
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
                        FILE_MODE_INPUT.as_lit_expr(1, 21),
                        FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                        1.as_lit_expr(1, 30)
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
                        FILE_MODE_RANDOM.as_lit_expr(1, 1),
                        FILE_ACCESS_READ.as_lit_expr(1, 24),
                        1.as_lit_expr(1, 32),
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
                        FILE_MODE_RANDOM.as_lit_expr(1, 1),
                        FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                        1.as_lit_expr(1, 20)
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
                        FILE_MODE_RANDOM.as_lit_expr(1, 1),
                        FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                        Expression::IntegerLiteral(1).at_rc(1, 20)
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
                Statement::SubCall(
                    "OPEN".into(),
                    vec![
                        Expression::Parenthesis(Box::new("FILE.TXT".as_lit_expr(1, 6))).at_rc(1, 5),
                        FILE_MODE_RANDOM.as_lit_expr(1, 1),
                        FILE_ACCESS_UNSPECIFIED.as_lit_expr(1, 1),
                        Expression::Parenthesis(Box::new(1.as_lit_expr(1, 20))).at_rc(1, 19)
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
    }
}

mod print {
    use super::*;

    pub fn parse_print_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::Print)
            .and_opt(parse_file_number_p())
            .and_opt_factory(|(_, opt_file_number)| using_p(opt_file_number.is_none()))
            .and_opt_factory(|((_, opt_file_number), opt_using)|
                 // we're just past PRINT. No need for space for ; or , but we need it for expressions
                first_print_arg_p( opt_file_number.is_none() && opt_using.is_none()).one_or_more_looking_back(|prev_arg| {
                    PrintArgLookingBack {
                        prev_print_arg_was_expression: prev_arg.is_expression(),
                    }
                })
            )
            .map(|(((_,opt_file_number), format_string), opt_args)| {
                Statement::Print(PrintNode {
                    file_number: opt_file_number.map(|x| x.element),
                    lpt1: false,
                    format_string,
                    args: opt_args.unwrap_or_default(),
                })
            })
    }

    pub fn parse_lprint_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::LPrint)
            .and_opt(using_p(true))
            .and_opt_factory(|(_keyword, opt_using)| {
                // we're just past LPRINT. No need for space for ; or , but we need it for expressions
                first_print_arg_p(opt_using.is_none()).one_or_more_looking_back(|prev_arg| {
                    PrintArgLookingBack {
                        prev_print_arg_was_expression: prev_arg.is_expression(),
                    }
                })
            })
            .map(|((_, format_string), opt_args)| {
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: true,
                    format_string,
                    args: opt_args.unwrap_or_default(),
                })
            })
    }

    fn using_p<R>(needs_leading_whitespace: bool) -> impl Parser<R, Output = ExpressionNode>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        opt_whitespace_p(needs_leading_whitespace)
            .and(keyword_p(Keyword::Using))
            .and_demand(
                expression::guarded_expression_node_p()
                    .or_syntax_error("Expected: expression after USING"),
            )
            .and_demand(item_p(';').or_syntax_error("Expected: ;"))
            .keep_left()
            .keep_right()
    }

    fn first_print_arg_p<R>(
        needs_leading_whitespace_for_expression: bool,
    ) -> impl Parser<R, Output = PrintArg>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        FirstPrintArg {
            needs_leading_whitespace_for_expression,
        }
    }

    struct FirstPrintArg {
        needs_leading_whitespace_for_expression: bool,
    }

    impl<R> Parser<R> for FirstPrintArg
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        type Output = PrintArg;

        fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
            if self.needs_leading_whitespace_for_expression {
                semicolon_or_comma_as_print_arg_p()
                    .preceded_by_opt_ws()
                    .or(expression::guarded_expression_node_p().map(|e| PrintArg::Expression(e)))
                    .parse(reader)
            } else {
                any_print_arg_p().preceded_by_opt_ws().parse(reader)
            }
        }
    }

    fn any_print_arg_p<R>() -> impl Parser<R, Output = PrintArg>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        semicolon_or_comma_as_print_arg_p()
            .or(expression::expression_node_p().map(|e| PrintArg::Expression(e)))
    }

    fn semicolon_or_comma_as_print_arg_p<R>() -> impl Parser<R, Output = PrintArg>
    where
        R: Reader<Item = char, Err = QError> + HasLocation,
    {
        any_p()
            .filter_reader_item(|ch| ch == ';' || ch == ',')
            .map(|ch| match ch {
                ';' => PrintArg::Semicolon,
                ',' => PrintArg::Comma,
                _ => panic!("Parser should not have parsed {}", ch),
            })
    }

    struct PrintArgLookingBack {
        prev_print_arg_was_expression: bool,
    }

    impl<R> Parser<R> for PrintArgLookingBack
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        type Output = PrintArg;

        fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
            if self.prev_print_arg_was_expression {
                // only comma or semicolon is allowed
                semicolon_or_comma_as_print_arg_p()
                    .preceded_by_opt_ws()
                    .parse(reader)
            } else {
                // everything is allowed
                any_print_arg_p().preceded_by_opt_ws().parse(reader)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_parser_err;
        use crate::parser::test_utils::*;

        #[test]
        fn test_print_no_args() {
            let input = "PRINT";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: false,
                    format_string: None,
                    args: vec![]
                })
            );
        }

        #[test]
        fn test_print_no_other_args_only_trailing_comma_space_variations() {
            let variations = ["PRINT,", "PRINT ,"];
            for input in &variations {
                let statement = parse(*input).demand_single_statement();
                assert_eq!(
                    statement,
                    Statement::Print(PrintNode {
                        file_number: None,
                        lpt1: false,
                        format_string: None,
                        args: vec![PrintArg::Comma]
                    })
                );
            }
        }

        #[test]
        fn test_print_no_other_args_only_trailing_semicolon_space_variations() {
            let variations = ["PRINT;", "PRINT ;"];
            for input in &variations {
                let statement = parse(*input).demand_single_statement();
                assert_eq!(
                    statement,
                    Statement::Print(PrintNode {
                        file_number: None,
                        lpt1: false,
                        format_string: None,
                        args: vec![PrintArg::Semicolon]
                    })
                );
            }
        }

        #[test]
        fn test_print_leading_comma_numeric_arg_space_variations() {
            let variations = ["PRINT,1", "PRINT ,1", "PRINT, 1", "PRINT , 1"];
            for input in &variations {
                let statement = parse(*input).demand_single_statement();
                match statement {
                    Statement::Print(print_node) => {
                        assert_eq!(print_node.file_number, None);
                        assert_eq!(print_node.lpt1, false);
                        assert_eq!(print_node.format_string, None);
                        assert_eq!(print_node.args[0], PrintArg::Comma);
                        match print_node.args[1] {
                            PrintArg::Expression(Locatable {
                                element: Expression::IntegerLiteral(1),
                                ..
                            }) => {}
                            _ => panic!("Argument mismatch"),
                        }
                        assert_eq!(print_node.args.len(), 2);
                    }
                    _ => panic!("{} did not yield a PrintNode", input),
                }
            }
        }

        #[test]
        fn test_print_leading_semicolon_numeric_arg_space_variations() {
            let variations = ["PRINT;1", "PRINT ;1", "PRINT; 1", "PRINT ; 1"];
            for input in &variations {
                let statement = parse(*input).demand_single_statement();
                match statement {
                    Statement::Print(print_node) => {
                        assert_eq!(print_node.file_number, None);
                        assert_eq!(print_node.lpt1, false);
                        assert_eq!(print_node.format_string, None);
                        assert_eq!(print_node.args[0], PrintArg::Semicolon);
                        match print_node.args[1] {
                            PrintArg::Expression(Locatable {
                                element: Expression::IntegerLiteral(1),
                                ..
                            }) => {}
                            _ => panic!("Argument mismatch"),
                        }
                        assert_eq!(print_node.args.len(), 2);
                    }
                    _ => panic!("{} did not yield a PrintNode", input),
                }
            }
        }

        #[test]
        fn test_lprint_no_args() {
            let input = "LPRINT";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: true,
                    format_string: None,
                    args: vec![]
                })
            );
        }

        #[test]
        fn test_print_one_arg() {
            let input = "PRINT 42";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: false,
                    format_string: None,
                    args: vec![PrintArg::Expression(42.as_lit_expr(1, 7))]
                })
            );
        }

        #[test]
        fn test_lprint_one_arg() {
            let input = "LPRINT 42";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: true,
                    format_string: None,
                    args: vec![PrintArg::Expression(42.as_lit_expr(1, 8))]
                })
            );
        }

        #[test]
        fn test_print_two_args() {
            let input = "PRINT 42, A";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: false,
                    format_string: None,
                    args: vec![
                        PrintArg::Expression(42.as_lit_expr(1, 7)),
                        PrintArg::Comma,
                        PrintArg::Expression("A".as_var_expr(1, 11))
                    ]
                })
            );
        }

        #[test]
        fn test_lprint_two_args() {
            let input = "LPRINT 42, A";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: true,
                    format_string: None,
                    args: vec![
                        PrintArg::Expression(42.as_lit_expr(1, 8)),
                        PrintArg::Comma,
                        PrintArg::Expression("A".as_var_expr(1, 12))
                    ]
                })
            );
        }

        #[test]
        fn test_print_file_no_args_no_comma() {
            let input = "PRINT #1";
            assert_parser_err!(input, QError::syntax_error("Expected: ,"), 1, 9);
        }

        #[test]
        fn test_print_file_no_args() {
            let input = "PRINT #1,";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: None,
                    args: vec![]
                })
            );
        }

        #[test]
        fn test_print_file_one_arg() {
            let input = "PRINT #1, 42";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: None,
                    args: vec![PrintArg::Expression(42.as_lit_expr(1, 11))]
                })
            );
        }

        #[test]
        fn test_print_file_semicolon_after_file_number_err() {
            let input = "PRINT #1; 42";
            assert_parser_err!(input, QError::syntax_error("Expected: ,"), 1, 9);
        }

        #[test]
        fn test_print_file_two_args() {
            let input = "PRINT #1, A, B";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: None,
                    args: vec![
                        PrintArg::Expression("A".as_var_expr(1, 11)),
                        PrintArg::Comma,
                        PrintArg::Expression("B".as_var_expr(1, 14))
                    ]
                })
            );
        }

        #[test]
        fn test_print_file_leading_comma() {
            let input = "PRINT #1,, A, B";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: None,
                    args: vec![
                        PrintArg::Comma,
                        PrintArg::Expression("A".as_var_expr(1, 12)),
                        PrintArg::Comma,
                        PrintArg::Expression("B".as_var_expr(1, 15))
                    ]
                })
            );
        }

        #[test]
        fn test_print_file_leading_semicolon() {
            let input = "PRINT #1,; A, B";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: None,
                    args: vec![
                        PrintArg::Semicolon,
                        PrintArg::Expression("A".as_var_expr(1, 12)),
                        PrintArg::Comma,
                        PrintArg::Expression("B".as_var_expr(1, 15))
                    ]
                })
            );
        }

        #[test]
        fn test_print_using_no_args() {
            let input = "PRINT USING \"#\";";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: false,
                    format_string: Some("#".as_lit_expr(1, 13)),
                    args: vec![]
                })
            );
        }

        #[test]
        fn test_lprint_using_no_args() {
            let input = "LPRINT USING \"#\";";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: true,
                    format_string: Some("#".as_lit_expr(1, 14)),
                    args: vec![]
                })
            );
        }

        #[test]
        fn test_print_using_no_args_missing_semicolon() {
            let input = "PRINT USING \"#\"";
            assert_parser_err!(input, QError::syntax_error("Expected: ;"), 1, 16);
        }

        #[test]
        fn test_lprint_using_no_args_missing_semicolon() {
            let input = "LPRINT USING \"#\"";
            assert_parser_err!(input, QError::syntax_error("Expected: ;"), 1, 17);
        }

        #[test]
        fn test_print_using_one_arg() {
            let input = "PRINT USING \"#\"; 42";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: false,
                    format_string: Some("#".as_lit_expr(1, 13)),
                    args: vec![PrintArg::Expression(42.as_lit_expr(1, 18))]
                })
            );
        }

        #[test]
        fn test_lprint_using_one_arg() {
            let input = "LPRINT USING \"#\"; 42";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: true,
                    format_string: Some("#".as_lit_expr(1, 14)),
                    args: vec![PrintArg::Expression(42.as_lit_expr(1, 19))]
                })
            );
        }

        #[test]
        fn test_print_file_using_no_args() {
            let input = "PRINT #1, USING \"#\";";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: Some("#".as_lit_expr(1, 17)),
                    args: vec![]
                })
            );
        }

        #[test]
        fn test_print_file_using_one_arg() {
            let input = "PRINT #1, USING \"#\"; 3.14";
            let statement = parse(input).demand_single_statement();
            assert_eq!(
                statement,
                Statement::Print(PrintNode {
                    file_number: Some(FileHandle::from(1)),
                    lpt1: false,
                    format_string: Some("#".as_lit_expr(1, 17)),
                    args: vec![PrintArg::Expression(3.14_f32.as_lit_expr(1, 22))]
                })
            );
        }

        #[test]
        fn test_lprint_no_comma_between_expressions_is_error() {
            let input = "LPRINT 1 2";
            assert_parser_err!(input, QError::syntax_error("No separator: 2"), 1, 11);
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

fn parse_file_number_p<R>() -> impl Parser<R, Output = Locatable<FileHandle>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::file_handle_p()
        .preceded_by_opt_ws()
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
