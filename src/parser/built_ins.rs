use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc2::binary::BinaryParser;
use crate::parser::pc2::text::{whitespace_p, TextParser};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::{item_p, Parser};
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;

/// Parses built-in subs which have a special syntax.
pub fn parse_built_in<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, crate::parser::Statement, QError>> {
    or_vec(vec![
        close::parse_close_p().convert_to_fn(),
        input::parse_input_p().convert_to_fn(),
        line_input::parse_line_input_p().convert_to_fn(),
        name::parse_name_p().convert_to_fn(),
        open::parse_open_p().convert_to_fn(),
        print::parse_print(),
        print::parse_lprint(),
    ])
}

mod close {
    use super::*;
    use crate::built_ins::BuiltInSub;
    use crate::parser::pc2::many::ManyParser;

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
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(QError::syntax_error("No separator: #"), Location::new(1, 7))
            );
        }

        #[test]
        fn test_one_file_number_with_hash_parenthesis_leading_space() {
            let input = "CLOSE (#1)";
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(
                    QError::syntax_error("Expected: expression inside parenthesis"),
                    Location::new(1, 8)
                )
            );
        }

        #[test]
        fn test_one_file_number_with_hash_parenthesis_no_leading_space() {
            let input = "CLOSE(#1)";
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(QError::syntax_error("No separator: ("), Location::new(1, 7))
            );
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
    use crate::built_ins::BuiltInSub;

    pub fn parse_input_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        // INPUT variable-list
        // LINE INPUT variable$
        // INPUT #file-number%, variable-list
        // LINE INPUT #file-number%, variable$
        keyword_p(Keyword::Input)
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after INPUT"))
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
        use super::*;
        use crate::assert_sub_call;
        use crate::parser::test_utils::*;

        #[test]
        fn test_parse_one_variable() {
            let input = "INPUT A$";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "INPUT",
                Expression::IntegerLiteral(0), // no file number
                Expression::var("A$")
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
                Expression::var("A$"),
                Expression::var("B")
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
                Expression::var("A")
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
                Expression::var("A")
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
                Expression::var("A")
            );
        }
    }
}

mod line_input {
    use super::*;
    use crate::built_ins::BuiltInSub;

    pub fn parse_line_input_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::Line)
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after LINE"))
            .and_demand(keyword_p(Keyword::Input).or_syntax_error("Expected: INPUT"))
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
        use super::*;
        use crate::assert_sub_call;
        use crate::parser::test_utils::*;

        #[test]
        fn test_parse_one_variable() {
            let input = "LINE INPUT A$";
            let result = parse(input).demand_single_statement();
            assert_sub_call!(
                result,
                "LINE INPUT",
                Expression::IntegerLiteral(0), // no file number
                Expression::var("A$")
            );
        }

        #[test]
        fn test_parse_two_variables() {
            let input = "LINE INPUT A$, B";
            assert_eq!(parse_err(input), QError::syntax_error("No separator: ,"));
        }

        #[test]
        fn test_no_whitespace_after_input() {
            let input = "LINE INPUT";
            assert_eq!(
                parse_err(input),
                QError::syntax_error("Expected: whitespace after LINE INPUT")
            );
        }

        #[test]
        fn test_no_variable() {
            let input = "LINE INPUT ";
            assert_eq!(
                parse_err(input),
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
                Expression::var("A")
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
                Expression::var("A")
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
                Expression::var("A")
            );
        }
    }
}

mod name {
    use super::*;
    use crate::built_ins::BuiltInSub;
    use crate::parser::expression::{back_guarded_expression_node_p, guarded_expression_node_p};

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
    use crate::built_ins::BuiltInSub;

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
                            map_opt_locatable_enum(opt_file_mode, FileMode::Input),
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
        i32: From<T>,
    {
        opt_locatable_enum
            .map(|Locatable { element, pos }| Expression::IntegerLiteral(element.into()).at(pos))
            .unwrap_or_else(|| Expression::IntegerLiteral(fallback.into()).at(Location::start()))
    }

    // FOR <ws+> INPUT <ws+>
    fn parse_open_mode_p<R>() -> impl Parser<R, Output = Locatable<FileMode>>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::For)
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after FOR"))
            .and_demand(
                keyword_p(Keyword::Append)
                    .or(keyword_p(Keyword::Input))
                    .or(keyword_p(Keyword::Output))
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
        keyword_p(Keyword::Access)
            .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after ACCESS"))
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
                        1.as_lit_expr(1, 21),
                        1.as_lit_expr(1, 34),
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
                        1.as_lit_expr(1, 21),
                        0.as_lit_expr(1, 1),
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
                        1.as_lit_expr(1, 1),
                        1.as_lit_expr(1, 24),
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
                        1.as_lit_expr(1, 1),
                        0.as_lit_expr(1, 1),
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
                        1.as_lit_expr(1, 1),
                        0.as_lit_expr(1, 1),
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
                        1.as_lit_expr(1, 1),
                        0.as_lit_expr(1, 1),
                        Expression::Parenthesis(Box::new(1.as_lit_expr(1, 20))).at_rc(1, 19)
                    ]
                )
            );
        }

        #[test]
        fn test_open_access_read_for_input_as_file_handle_with_spaces() {
            let input = r#"OPEN "FILE.TXT" ACCESS READ FOR INPUT AS #1"#;
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(
                    QError::syntax_error("Expected: AS file-number"),
                    Location::new(1, 29)
                )
            );
        }
    }
}

mod print {
    use super::*;
    use crate::parser::pc::combine::{combine_if_first_ok, switch};
    use crate::parser::pc::map::opt_map;

    pub fn parse_print<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        parse_print_or_lprint(false)
    }

    pub fn parse_lprint<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        parse_print_or_lprint(true)
    }

    fn parse_print_or_lprint<T: BufRead + 'static>(
        lpt1: bool,
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
        map(
            opt_seq2(
                keyword(if lpt1 {
                    Keyword::LPrint
                } else {
                    Keyword::Print
                }),
                switch(
                    print_file_number_and_format_string_and_first_arg(lpt1),
                    parse_remaining_print_args,
                ),
            ),
            move |(_, o)| map_print_result(lpt1, o),
        )
    }

    fn map_print_result(
        lpt1: bool,
        o: Option<(
            Option<Locatable<FileHandle>>,
            Option<ExpressionNode>,
            Vec<PrintArg>,
        )>,
    ) -> Statement {
        match o {
            Some((file_number, format_string, args)) => Statement::Print(PrintNode {
                file_number: file_number.map(|x| x.strip_location()),
                lpt1,
                format_string,
                args,
            }),
            None => Statement::Print(PrintNode {
                file_number: None,
                lpt1,
                format_string: None,
                args: vec![],
            }),
        }
    }

    fn parse_remaining_print_args<T: BufRead + 'static>(
        args: (
            Option<Locatable<FileHandle>>,
            Option<ExpressionNode>,
            Option<PrintArg>,
        ),
    ) -> Box<
        dyn Fn(
            EolReader<T>,
        ) -> ReaderResult<
            EolReader<T>,
            (
                Option<Locatable<FileHandle>>,
                Option<ExpressionNode>,
                Vec<PrintArg>,
            ),
            QError,
        >,
    > {
        let (opt_file_handle, opt_format_string, opt_first_arg) = args;
        match opt_first_arg {
            Some(first_arg) => map(
                many_looking_back(Some(first_arg), parse_print_arg_looking_back),
                move |args| (opt_file_handle.clone(), opt_format_string.clone(), args),
            ),
            None => Box::new(move |r| {
                Ok((
                    r,
                    Some((opt_file_handle.clone(), opt_format_string.clone(), vec![])),
                ))
            }),
        }
    }

    fn print_file_number_and_format_string_and_first_arg<T: BufRead + 'static>(
        lpt1: bool,
    ) -> Box<
        dyn Fn(
            EolReader<T>,
        ) -> ReaderResult<
            EolReader<T>,
            (
                Option<Locatable<FileHandle>>,
                Option<ExpressionNode>,
                Option<PrintArg>,
            ),
            QError,
        >,
    > {
        opt_map(
            combine_if_first_ok(
                print_file_number_and_format_string(lpt1),
                parse_first_print_arg,
            ),
            |(opt_file_number_and_format_string, opt_first_print_arg)| {
                match opt_file_number_and_format_string {
                    Some((opt_file_number, opt_format_string)) => {
                        Some((opt_file_number, opt_format_string, opt_first_print_arg))
                    }
                    None => {
                        if opt_first_print_arg.is_some() {
                            Some((None, None, opt_first_print_arg))
                        } else {
                            None
                        }
                    }
                }
            },
        )
    }

    fn print_file_number_and_format_string<T: BufRead + 'static>(
        lpt1: bool,
    ) -> Box<
        dyn Fn(
            EolReader<T>,
        ) -> ReaderResult<
            EolReader<T>,
            (Option<Locatable<FileHandle>>, Option<ExpressionNode>),
            QError,
        >,
    > {
        if lpt1 {
            combine_if_first_ok(|r| Ok((r, None)), parse_using)
        } else {
            combine_if_first_ok(parse_file_number(), parse_using)
        }
    }

    fn parse_using<T: BufRead + 'static>(
        file_number: Option<&Locatable<FileHandle>>,
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        if file_number.is_some() {
            // we are past PRINT #1,  we don't need to demand space before USING
            map(
                seq3(
                    ws::zero_or_more_leading(keyword(Keyword::Using)),
                    expression::demand_guarded_expression_node(),
                    demand(read(';'), QError::syntax_error_fn("Expected: ;")),
                ),
                |(_, expr, _)| expr,
            )
        } else {
            // we are past PRINT, we need a whitespace
            map(
                seq3(
                    ws::one_or_more_leading(keyword(Keyword::Using)),
                    expression::demand_guarded_expression_node(),
                    demand(read(';'), QError::syntax_error_fn("Expected: ;")),
                ),
                |(_, expr, _)| expr,
            )
        }
    }

    fn parse_first_print_arg<T: BufRead + 'static>(
        opt_file_handle_and_format_string: Option<&(
            Option<Locatable<FileHandle>>,
            Option<ExpressionNode>,
        )>,
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, PrintArg, QError>> {
        if opt_file_handle_and_format_string.is_some() {
            // we're either past PRINT #1, or PRINT [#1,]USING "x";
            // in any case, no need to demand whitespace
            ws::zero_or_more_leading(or_vec(vec![
                map(read(';'), |_| PrintArg::Semicolon),
                map(read(','), |_| PrintArg::Comma),
                map(expression::expression_node(), |e| PrintArg::Expression(e)),
            ]))
        } else {
            // we're just past PRINT. No need for space for ; or , but we need it for expressions
            or_vec(vec![
                map(ws::zero_or_more_leading(read(';')), |_| PrintArg::Semicolon),
                map(ws::zero_or_more_leading(read(',')), |_| PrintArg::Comma),
                map(expression::guarded_expression_node(), |e| {
                    PrintArg::Expression(e)
                }),
            ])
        }
    }

    fn parse_print_arg_looking_back<T: BufRead + 'static>(
        opt_prev_arg: Option<&PrintArg>,
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, PrintArg, QError>> {
        match opt_prev_arg {
            Some(prev_arg) => {
                match prev_arg {
                    PrintArg::Expression(_) => {
                        // only comma or semicolon is allowed
                        ws::zero_or_more_leading(or_vec(vec![
                            map(read(';'), |_| PrintArg::Semicolon),
                            map(read(','), |_| PrintArg::Comma),
                        ]))
                    }
                    _ => {
                        // everything is allowed
                        ws::zero_or_more_leading(or_vec(vec![
                            map(read(';'), |_| PrintArg::Semicolon),
                            map(read(','), |_| PrintArg::Comma),
                            map(expression::expression_node(), |e| PrintArg::Expression(e)),
                        ]))
                    }
                }
            }
            None => Box::new(|r| Ok((r, None))),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
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
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(QError::syntax_error("Expected: ,"), Location::new(1, 9))
            );
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
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(QError::syntax_error("Expected: ,"), Location::new(1, 9))
            );
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
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(QError::syntax_error("Expected: ;"), Location::new(1, 16))
            );
        }

        #[test]
        fn test_lprint_using_no_args_missing_semicolon() {
            let input = "LPRINT USING \"#\"";
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(QError::syntax_error("Expected: ;"), Location::new(1, 17))
            );
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
            assert_eq!(
                parse_err_node(input),
                QErrorNode::Pos(
                    QError::syntax_error("No separator: 2"),
                    Location::new(1, 11)
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

#[deprecated]
fn parse_file_number<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<FileHandle>, QError>> {
    parse_file_number_p().convert_to_fn()
}

fn parse_file_number_p<R>() -> impl Parser<R, Output = Locatable<FileHandle>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::file_handle_p()
        .preceded_by_opt_ws()
        .keep_right()
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
