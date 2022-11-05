use crate::built_ins::{encode_opt_file_handle_arg, opt_file_handle_comma_p};
use crate::expression::csv_expressions_non_opt;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;

// INPUT variable-list
// INPUT #file-number%, variable-list
pub fn parse() -> impl Parser<Output = Statement> {
    seq4(
        keyword(Keyword::Input),
        whitespace().no_incomplete(),
        opt_file_handle_comma_p(),
        csv_expressions_non_opt("Expected: #file-number or variable"),
        |_, _, opt_file_number_pos, variables| {
            let mut args: Expressions = encode_opt_file_handle_arg(opt_file_number_pos);
            args.extend(variables);
            Statement::BuiltInSubCall(BuiltInSub::Input, args)
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_built_in_sub_call;
    use crate::assert_parser_err;
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;

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
        assert_parser_err!(input, QError::syntax_error("Expected: whitespace"));
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
