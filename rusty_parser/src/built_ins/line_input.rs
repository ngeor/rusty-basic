use crate::built_ins::{encode_opt_file_handle_arg, opt_file_handle_comma_p};
use crate::expression::expression_pos_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

// LINE INPUT variable$
// LINE INPUT #file-number%, variable$
pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    seq4(
        keyword_pair(Keyword::Line, Keyword::Input),
        whitespace(),
        opt_file_handle_comma_p(),
        expression_pos_p().or_syntax_error("Expected: #file-number or variable"),
        |_, _, opt_file_number_pos, variable| {
            let mut args: Expressions = encode_opt_file_handle_arg(opt_file_number_pos);
            // add the LINE INPUT variable
            args.push(variable);
            Statement::BuiltInSubCall(BuiltInSub::LineInput, args)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_built_in_sub_call;
    use crate::assert_parser_err;
    use crate::test_utils::*;
    use crate::*;

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
        assert_parser_err!(input, ParseError::syntax_error("No separator: ,"));
    }

    #[test]
    fn test_no_whitespace_after_input() {
        let input = "LINE INPUT";
        assert_parser_err!(input, ParseError::syntax_error("Expected: whitespace"));
    }

    #[test]
    fn test_no_variable() {
        let input = "LINE INPUT ";
        assert_parser_err!(
            input,
            ParseError::syntax_error("Expected: #file-number or variable")
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
