use rusty_pc::*;

use crate::built_ins::common::{encode_opt_file_handle_arg, opt_file_handle_comma_p};
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::whitespace_ignoring;
use crate::{BuiltInSub, ParserError, *};
// LINE INPUT variable$
// LINE INPUT #file-number%, variable$
pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParserError> {
    seq4(
        keyword_pair(Keyword::Line, Keyword::Input),
        whitespace_ignoring(),
        opt_file_handle_comma_p(),
        expression_pos_p().or_expected("#file-number or variable"),
        |_, _, opt_file_number_pos, variable| {
            let mut args: Expressions = encode_opt_file_handle_arg(opt_file_number_pos);
            // add the LINE INPUT variable
            args.push(variable);
            Statement::built_in_sub_call(BuiltInSub::LineInput, args)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::{BuiltInSub, assert_built_in_sub_call, assert_parser_err, *};
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
        assert_parser_err!(input, ParserErrorKind::expected("end-of-statement"));
    }

    #[test]
    fn test_no_whitespace_ignoring_after_input() {
        let input = "LINE INPUT";
        assert_parser_err!(input, ParserErrorKind::expected("whitespace"));
    }

    #[test]
    fn test_no_variable() {
        let input = "LINE INPUT ";
        assert_parser_err!(input, ParserErrorKind::expected("#file-number or variable"));
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
