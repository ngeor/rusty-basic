use crate::common::*;
use crate::parser::expression;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

pub fn parse_print_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Print)
            .and_opt(ws_file_handle_comma_p())
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

fn ws_file_handle_comma_p<R>() -> impl Parser<R, Output = Locatable<FileHandle>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(expression::file_handle_p())
        .and_demand(
            item_p(',')
                .surrounded_by_opt_ws()
                .or_syntax_error("Expected: ,"),
        )
        .keep_middle()
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
