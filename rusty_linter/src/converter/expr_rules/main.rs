use rusty_common::*;
use rusty_parser::*;

use crate::converter::common::{Context, ConvertibleIn, ExprContext, ExprContextPos};
use crate::converter::expr_rules::{
    binary, built_in_function, function, property, unary, variable
};
use crate::core::LintErrorPos;

//
// ExpressionPos ConvertibleIn
//

impl ConvertibleIn<ExprContext> for ExpressionPos {
    fn convert_in(
        self,
        ctx: &mut Context,
        expr_context: ExprContext,
    ) -> Result<Self, LintErrorPos> {
        let Self { element: expr, pos } = self;
        expr.convert_in(ctx, expr_context.at_pos(pos))
            .map(|expr| expr.at_pos(pos))
    }
}

impl ConvertibleIn<ExprContext> for Box<ExpressionPos> {
    fn convert_in(
        self,
        ctx: &mut Context,
        expr_context: ExprContext,
    ) -> Result<Self, LintErrorPos> {
        let unboxed = *self;
        unboxed.convert_in(ctx, expr_context).map(Self::new)
    }
}

//
// Expression ConvertibleIn
//

impl ConvertibleIn<ExprContextPos> for Expression {
    fn convert_in(self, ctx: &mut Context, extra: ExprContextPos) -> Result<Self, LintErrorPos> {
        match self {
            // literals
            Self::SingleLiteral(_)
            | Self::DoubleLiteral(_)
            | Self::StringLiteral(_)
            | Self::IntegerLiteral(_)
            | Self::LongLiteral(_) => Ok(self),
            // parenthesis
            Self::Parenthesis(box_child) => box_child
                .convert_in(ctx, extra.element)
                .map(Expression::Parenthesis),
            // unary
            Self::UnaryExpression(unary_operator, box_child) => {
                unary::convert(ctx, extra, unary_operator, *box_child)
            }
            // binary
            Self::BinaryExpression(binary_operator, left, right, _expr_type) => {
                binary::convert(ctx, extra, binary_operator, *left, *right)
            }
            // variables
            Self::Variable(name, variable_info) => {
                variable::convert(ctx, extra, name, variable_info)
            }
            Self::ArrayElement(_name, _indices, _variable_info) => {
                panic!(
                    "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
                )
            }
            Self::Property(box_left_side, property_name, _expr_type) => {
                property::convert(ctx, extra, box_left_side, property_name)
            }
            // function call
            Self::FunctionCall(name, args) => function::convert(ctx, extra, name, args),
            Self::BuiltInFunctionCall(built_in_function, args) => {
                built_in_function::convert(ctx, built_in_function, extra.pos, args)
            }
        }
    }
}
