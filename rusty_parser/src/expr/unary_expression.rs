use rusty_common::Positioned;
use rusty_pc::*;

use crate::expr::{expression_pos_p, ws_expr_pos_p};
use crate::input::StringView;
use crate::pc_specific::{WithPos, keyword};
use crate::tokens::minus_sign;
use crate::{ExpressionPos, ExpressionPosTrait, Keyword, ParserError, UnaryOperator};

pub(super) fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    unary_minus()
        .or(unary_not())
        .map(|(Positioned { element: op, pos }, expr)| expr.apply_unary_priority_order(op, pos))
}

fn unary_minus()
-> impl Parser<StringView, Output = (Positioned<UnaryOperator>, ExpressionPos), Error = ParserError>
{
    minus_sign()
        .map(|_| UnaryOperator::Minus)
        .with_pos()
        .and_tuple(expression_pos_p().or_expected("expression after -"))
}

fn unary_not()
-> impl Parser<StringView, Output = (Positioned<UnaryOperator>, ExpressionPos), Error = ParserError>
{
    keyword(Keyword::Not)
        .map(|_| UnaryOperator::Not)
        .with_pos()
        .and_tuple(ws_expr_pos_p().or_expected("expression after NOT"))
}

#[cfg(test)]
mod tests {
    use rusty_common::AtPos;

    use crate::test_utils::*;
    use crate::{assert_expression, *};

    #[test]
    fn test_negated_variable() {
        assert_expression!(
            "-N",
            Expression::UnaryExpression(UnaryOperator::Minus, Box::new("N".as_var_expr(1, 8)))
        );
    }

    #[test]
    fn test_negated_number_literal_resolved_eagerly_during_parsing() {
        assert_expression!("-42", Expression::IntegerLiteral(-42));
    }

    #[test]
    fn test_negated_function_call() {
        assert_expression!(
            "-Fib(-N)",
            Expression::UnaryExpression(
                UnaryOperator::Minus,
                Box::new(
                    Expression::func(
                        "Fib",
                        vec![
                            Expression::UnaryExpression(
                                UnaryOperator::Minus,
                                Box::new("N".as_var_expr(1, 13)),
                            )
                            .at_rc(1, 12)
                        ],
                    )
                    .at_rc(1, 8)
                )
            )
        );
    }
}
