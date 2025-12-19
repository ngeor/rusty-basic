use crate::expression::file_handle::file_handle_p;
use crate::expression::guard::Guard;
use crate::expression::{expression_pos_p, ws_expr_pos_p};
use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;
use crate::ParseError;
use rusty_common::*;

/// See [Print] for the definition.
pub fn parse_print_p() -> impl Parser<RcStringView, Output = Statement> {
    keyword(Keyword::Print)
        .and_opt_keep_right(print_boundary().and_tuple(Seq3::new(
            opt_file_handle_comma_p(),
            opt_using(),
            PrintArgsParser,
        )))
        .map(|opt_args| opt_args.unwrap_or_default())
        .map(|(_, (opt_file_number, format_string, args))| {
            Statement::Print(Print {
                file_number: opt_file_number.map(|x| x.element),
                lpt1: false,
                format_string,
                args,
            })
        })
}

pub fn parse_lprint_p() -> impl Parser<RcStringView, Output = Statement> {
    keyword(Keyword::LPrint)
        .and_opt_keep_right(print_boundary().and_tuple(Seq2::new(opt_using(), PrintArgsParser)))
        .map(|opt_args| opt_args.unwrap_or_default())
        .map(|(_, (format_string, args))| {
            Statement::Print(Print {
                file_number: None,
                lpt1: true,
                format_string,
                args,
            })
        })
}

fn opt_using() -> impl Parser<RcStringView, Output = Option<ExpressionPos>> {
    seq3(
        keyword(Keyword::Using),
        ws_expr_pos_p().or_syntax_error("Expected: expression after USING"),
        semicolon().no_incomplete(),
        |_, using_expr, _| using_expr,
    )
    .to_option()
}

fn opt_file_handle_comma_p() -> impl Parser<RcStringView, Output = Option<Positioned<FileHandle>>> {
    seq2(
        file_handle_p(),
        comma().no_incomplete(),
        |file_handle, _| file_handle,
    )
    .to_option()
}

pub struct PrintArgsParser;

impl PrintArgsParser {
    fn next(
        tokenizer: RcStringView,
        allow_expr: bool,
    ) -> ParseResult<RcStringView, PrintArg, ParseError> {
        if allow_expr {
            Self::any_print_arg().parse(tokenizer)
        } else {
            Self::delimiter_print_arg().parse(tokenizer)
        }
    }

    fn any_print_arg() -> impl Parser<RcStringView, Output = PrintArg> {
        expression_pos_p()
            .map(PrintArg::Expression)
            .or(Self::delimiter_print_arg())
    }

    fn delimiter_print_arg() -> impl Parser<RcStringView, Output = PrintArg> {
        semicolon()
            .map(|_| PrintArg::Semicolon)
            .or(comma().map(|_| PrintArg::Comma))
    }
}

impl Parser<RcStringView> for PrintArgsParser {
    type Output = Vec<PrintArg>;

    fn parse(
        &self,
        mut tokenizer: RcStringView,
    ) -> ParseResult<RcStringView, Self::Output, ParseError> {
        let mut result: Vec<PrintArg> = vec![];
        let mut last_one_was_expression = false;
        loop {
            match Self::next(tokenizer, !last_one_was_expression) {
                Ok((remaining, next)) => {
                    last_one_was_expression = next.is_expression();
                    result.push(next);
                    tokenizer = remaining;
                }
                Err((false, remaining, _)) => {
                    tokenizer = remaining;
                    break;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok((tokenizer, result))
    }
}

fn print_boundary() -> impl Parser<RcStringView, Output = Guard> {
    whitespace()
        .map(|_| Guard::Whitespace)
        .or(peek_token().flat_map(|input, token| {
            if TokenType::Comma.matches(&token)
                || TokenType::Semicolon.matches(&token)
                || TokenType::LParen.matches(&token)
            {
                Ok((input, Guard::Peeked))
            } else {
                default_parse_error(input)
            }
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::{assert_parser_err, parse};

    #[test]
    fn test_print_no_args() {
        let input = "PRINT";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::Print(Print {
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
                Statement::Print(Print {
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
                Statement::Print(Print {
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
                Statement::Print(print) => {
                    assert_eq!(print.file_number, None);
                    assert_eq!(print.lpt1, false);
                    assert_eq!(print.format_string, None);
                    assert_eq!(print.args[0], PrintArg::Comma);
                    match print.args[1] {
                        PrintArg::Expression(Positioned {
                            element: Expression::IntegerLiteral(1),
                            ..
                        }) => {}
                        _ => panic!("Argument mismatch"),
                    }
                    assert_eq!(print.args.len(), 2);
                }
                _ => panic!("{} did not yield a Print", input),
            }
        }
    }

    #[test]
    fn test_print_leading_semicolon_numeric_arg_space_variations() {
        let variations = ["PRINT;1", "PRINT ;1", "PRINT; 1", "PRINT ; 1"];
        for input in &variations {
            let statement = parse(*input).demand_single_statement();
            match statement {
                Statement::Print(print) => {
                    assert_eq!(print.file_number, None);
                    assert_eq!(print.lpt1, false);
                    assert_eq!(print.format_string, None);
                    assert_eq!(print.args[0], PrintArg::Semicolon);
                    match print.args[1] {
                        PrintArg::Expression(Positioned {
                            element: Expression::IntegerLiteral(1),
                            ..
                        }) => {}
                        _ => panic!("Argument mismatch"),
                    }
                    assert_eq!(print.args.len(), 2);
                }
                _ => panic!("{} did not yield a Print", input),
            }
        }
    }

    #[test]
    fn test_lprint_no_args() {
        let input = "LPRINT";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
        assert_parser_err!(input, ParseError::syntax_error("Expected: ,"), 1, 9);
    }

    #[test]
    fn test_print_file_no_args() {
        let input = "PRINT #1,";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::Print(Print {
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
            Statement::Print(Print {
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
        assert_parser_err!(input, ParseError::syntax_error("Expected: ,"), 1, 9);
    }

    #[test]
    fn test_print_file_two_args() {
        let input = "PRINT #1, A, B";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
        assert_parser_err!(input, ParseError::syntax_error("Expected: ;"), 1, 16);
    }

    #[test]
    fn test_lprint_using_no_args_missing_semicolon() {
        let input = "LPRINT USING \"#\"";
        assert_parser_err!(input, ParseError::syntax_error("Expected: ;"), 1, 17);
    }

    #[test]
    fn test_print_using_one_arg() {
        let input = "PRINT USING \"#\"; 42";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
            Statement::Print(Print {
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
        assert_parser_err!(input, ParseError::syntax_error("No separator: 2"), 1, 11);
    }
}
