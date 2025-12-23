use crate::built_ins::built_in_sub::BuiltInSub;
use crate::built_ins::parse_built_in_sub_with_opt_args;
use crate::pc::*;
use crate::specific::*;

pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    parse_built_in_sub_with_opt_args(Keyword::Color, BuiltInSub::Color)
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::built_ins::built_in_sub::BuiltInSub;
    use crate::error::ParseError;
    use crate::parse;
    use crate::specific::Statement;
    use crate::test_utils::*;

    #[test]
    fn parse_foreground_only() {
        let input = "COLOR 7";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![1.as_lit_expr(1, 1), 7.as_lit_expr(1, 7)]
            )
        );
    }

    #[test]
    fn parse_background_only() {
        let input = "COLOR , 7";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![2.as_lit_expr(1, 1), 7.as_lit_expr(1, 9)]
            )
        );
    }

    #[test]
    fn parse_both_colors() {
        let input = "COLOR 7, 4";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::BuiltInSubCall(
                BuiltInSub::Color,
                vec![
                    3.as_lit_expr(1, 1),
                    7.as_lit_expr(1, 7),
                    4.as_lit_expr(1, 10)
                ]
            )
        );
    }

    #[test]
    fn parse_no_args() {
        let input = "COLOR";
        assert_parser_err!(
            input,
            ParseError::syntax_error("Expected: whitespace"),
            1,
            6
        );
    }
}
