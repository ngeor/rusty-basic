use rusty_common::*;
use rusty_pc::many::VecManyCombiner;
use rusty_pc::many_ctx::ManyCtxParser;
use rusty_pc::*;

use crate::core::expression::file_handle::file_handle_p;
use crate::core::expression::{expression_pos_p, ws_expr_pos_p};
use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::{
    TokenMatcher, any_symbol_of, any_symbol_of_ws, any_token_of, comma_ws, semicolon_ws, whitespace_ignoring
};
use crate::*;

/// A call to the PRINT sub.
/// As separators are important (even trailing needs to be preserved), PRINT has
/// its own type.
///
/// ```basic
/// PRINT [#file-number%,] [expression list] [{; | ,}]
/// LPRINT [expression list] [{; | ,}]
/// ```
///
/// Formatted output:
///
/// ```basic
/// PRINT [#file-number%,] USING format-string$; [expression list] [{; | ,}]
/// LPRINT USING format-string$; [expression list] [{; | ,}]
/// ```
///
/// `expression list`: A list of one or more numeric or string expressions to
/// print, separated by commas, semicolons, spaces, or tabs (note: spaces or
/// tabs means `PRINT , , 4` is a valid statement, as is `PRINT 4, , ,`).
///
/// `{; | ,}` Determines where the next output begins:
///
/// - `;` means print immediately after the last value.
/// - `,` means print at the start of the next print zone.
///   Print zones are 14 characters wide.
///
/// # Format specifiers
///
/// ## Characters that format a numeric expression
///
/// - `#` Digit position.
/// - `.` Decimal point position.
/// - `,` Placed left of the decimal point, prints a comma every third digit.
/// - `+` Position of number sign.
/// - `^^^^` Prints in exponential format.
/// - `-` Placed after digit, prints trailing sign for negative numbers.
/// - `$$` Prints leading $.
/// - `**` Fills leading spaces with *.
/// - `**$` Combines ** and $$.
///
/// ## Characters used to format a string expression
///
/// - `&` Prints entire string.
/// - `!` Prints only the first character of the string.
/// - `\ \` Prints first n characters, where n is the number of blanks between slashes + 2.
///
/// ## Characters used to print literal characters
///
/// - `_` Prints the following formatting character as a literal.
///
/// Any other character is printed as a literal.
#[derive(Clone, Debug, PartialEq)]
pub struct Print {
    pub file_number: Option<FileHandle>,
    pub lpt1: bool,
    pub format_string: Option<ExpressionPos>,
    pub args: Vec<PrintArg>,
}

impl Print {
    pub fn one(expression_pos: ExpressionPos) -> Self {
        Self {
            file_number: None,
            lpt1: false,
            format_string: None,
            args: vec![PrintArg::Expression(expression_pos)],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrintArg {
    Comma,
    Semicolon,
    Expression(ExpressionPos),
}

impl PrintArg {
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }
}

/// See [Print] for the definition.
pub fn parse_print_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword(Keyword::Print)
        .and_opt_keep_right(print_boundary().and_tuple(seq3(
            opt_file_handle_comma_p(),
            opt_using(),
            print_args_parser(),
            |a, b, c| (a, b, c),
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

pub fn parse_lprint_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword(Keyword::LPrint)
        .and_opt_keep_right(print_boundary().and_tuple(seq2(
            opt_using(),
            print_args_parser(),
            |l, r| (l, r),
        )))
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

fn opt_using() -> impl Parser<StringView, Output = Option<ExpressionPos>, Error = ParserError> {
    seq3(
        keyword(Keyword::Using),
        ws_expr_pos_p().or_expected("expression after USING"),
        semicolon_ws(),
        |_, using_expr, _| using_expr,
    )
    .to_option()
}

fn opt_file_handle_comma_p()
-> impl Parser<StringView, Output = Option<Positioned<FileHandle>>, Error = ParserError> {
    seq2(file_handle_p(), comma_ws(), |file_handle, _| file_handle).to_option()
}

fn print_args_parser() -> impl Parser<StringView, Output = Vec<PrintArg>, Error = ParserError> {
    ManyCtxParser::new(
        print_arg_parser(),
        VecManyCombiner,
        PrintArg::is_expression,
        true,
    )
}

fn print_arg_parser()
-> impl Parser<StringView, bool, Output = PrintArg, Error = ParserError> + SetContext<bool> {
    IifParser::new(delimiter_print_arg(), any_print_arg())
}

fn any_print_arg() -> impl Parser<StringView, Output = PrintArg, Error = ParserError> {
    expression_pos_p()
        .map(PrintArg::Expression)
        .or(delimiter_print_arg())
}

fn delimiter_print_arg() -> impl Parser<StringView, Output = PrintArg, Error = ParserError> {
    // TODO support char based tokens or make the next mapping friendlier
    any_symbol_of_ws!(';', ',').map(|t| {
        if ';'.matches_token(&t) {
            PrintArg::Semicolon
        } else if ','.matches_token(&t) {
            PrintArg::Comma
        } else {
            unreachable!()
        }
    })
}

fn print_boundary() -> impl Parser<StringView, Output = (), Error = ParserError> {
    whitespace_ignoring().or(any_symbol_of!(',', ';', '(').map_to_unit().peek())
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
        assert_parser_err!(input, expected(","), 1, 9);
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
        assert_parser_err!(input, expected(","), 1, 9);
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
        assert_parser_err!(input, expected(";"), 1, 16);
    }

    #[test]
    fn test_lprint_using_no_args_missing_semicolon() {
        let input = "LPRINT USING \"#\"";
        assert_parser_err!(input, expected(";"), 1, 17);
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
        assert_parser_err!(input, expected("end-of-statement"), 1, 11);
    }
}
