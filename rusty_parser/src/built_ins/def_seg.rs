use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::equal_sign_ws;
use crate::{BuiltInSub, ParseError, *};

// DEF SEG(=expr)?
pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq2(
        keyword_pair(Keyword::Def, Keyword::Seg),
        equal_sign_and_expression().to_option(),
        |_, opt_expr_pos| {
            Statement::built_in_sub_call(BuiltInSub::DefSeg, opt_expr_pos.into_iter().collect())
        },
    )
}

fn equal_sign_and_expression()
-> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    equal_sign_ws()
        .and_keep_right(expression_pos_p().or_syntax_error("Expected expression after equal sign"))
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{DemandSingleStatement, ExpressionLiteralFactory};
    use crate::{BuiltInSub, Statement, *};

    #[test]
    fn parse_no_items_is_allowed() {
        let input = "DEF SEG";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::built_in_sub_call(BuiltInSub::DefSeg, vec![])
        );
    }

    #[test]
    fn parse_one_item() {
        let input = "DEF SEG = 42";
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::built_in_sub_call(BuiltInSub::DefSeg, vec![42.as_lit_expr(1, 11)])
        );
    }
}
