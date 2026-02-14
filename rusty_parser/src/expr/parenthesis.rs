use rusty_pc::*;

use crate::expr::expression_pos_p;
use crate::input::StringView;
use crate::pc_specific::{OrExpected, WithPos, in_parenthesis};
use crate::{ParserError, *};

pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    in_parenthesis(expression_pos_p().or_expected("expression inside parenthesis"))
        .map(|child| Expression::Parenthesis(Box::new(child)))
        .with_pos()
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{assert_expression, *};

    #[test]
    fn test_whitespace_inside_parenthesis() {
        assert_expression!(
            "( 1 AND 2 )",
            Expression::Parenthesis(Box::new(
                Expression::BinaryExpression(
                    Operator::And,
                    Box::new(1.as_lit_expr(1, 9)),
                    Box::new(2.as_lit_expr(1, 15)),
                    ExpressionType::Unresolved
                )
                .at_rc(1, 11)
            ))
        );
    }
}
