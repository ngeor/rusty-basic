use crate::converter::common::Convertible;
use crate::converter::expr_rules::state::ExprState;
use crate::converter::expr_rules::state::PosExprState;
use crate::converter::expr_rules::{
    binary, built_in_function, function, property, unary, variable,
};
use crate::core::LintErrorPos;
use crate::core::LintPosResult;
use rusty_common::*;
use rusty_parser::specific::*;

//
// ExpressionPos Convertible
//

impl<'a> Convertible<ExprState<'a>> for ExpressionPos {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, LintErrorPos> {
        let Self { element: expr, pos } = self;
        expr.convert_in(ctx, pos)
            .map(|expr| expr.at_pos(pos))
            .patch_err_pos(&pos)
    }
}

impl<'a> Convertible<ExprState<'a>> for Box<ExpressionPos> {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, LintErrorPos> {
        let unboxed = *self;
        unboxed.convert(ctx).map(Self::new)
    }
}

//
// Expression Convertible
//

impl<'a, 'b> Convertible<PosExprState<'a, 'b>> for Expression {
    fn convert(self, ctx: &mut PosExprState<'a, 'b>) -> Result<Self, LintErrorPos> {
        match self {
            // literals
            Self::SingleLiteral(_)
            | Self::DoubleLiteral(_)
            | Self::StringLiteral(_)
            | Self::IntegerLiteral(_)
            | Self::LongLiteral(_) => Ok(self),
            // parenthesis
            Self::Parenthesis(box_child) => box_child.convert(ctx).map(Expression::Parenthesis),
            // unary
            Self::UnaryExpression(unary_operator, box_child) => {
                unary::convert(ctx, unary_operator, *box_child)
            }
            // binary
            Self::BinaryExpression(binary_operator, left, right, _expr_type) => {
                binary::convert(ctx, binary_operator, *left, *right)
            }
            // variables
            Self::Variable(name, variable_info) => variable::convert(ctx, name, variable_info),
            Self::ArrayElement(_name, _indices, _variable_info) => {
                panic!(
                    "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
                )
            }
            Self::Property(box_left_side, property_name, _expr_type) => {
                property::convert(ctx, box_left_side, property_name)
            }
            // function call
            Self::FunctionCall(name, args) => function::convert(ctx, name, args),
            Self::BuiltInFunctionCall(built_in_function, args) => {
                built_in_function::convert(ctx, built_in_function, args)
            }
        }
    }
}
