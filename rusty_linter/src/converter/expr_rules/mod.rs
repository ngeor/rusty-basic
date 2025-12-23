mod binary;
mod built_in_function;
mod function;
mod property;
mod unary;
mod variable;

use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::error::LintErrorPos;
use crate::LintPosResult;
use expr_state::ExprState;
use pos_expr_state::PosExprState;
use rusty_common::*;
use rusty_parser::built_ins::built_in_function::BuiltInFunction;
use rusty_parser::specific::*;

mod expr_state {
    use crate::converter::context::Context;
    use crate::converter::traits::FromParentContext;
    use crate::converter::types::ExprContext;
    use std::ops::{Deref, DerefMut};

    /// A context that is used when converting an [ExpressionPos].
    /// Enhances the parent [Context] with an [ExprContext].
    pub struct ExprState<'a> {
        ctx: &'a mut Context,
        expr_context: ExprContext,
    }

    impl<'a> ExprState<'a> {
        pub fn new(ctx: &'a mut Context, expr_context: ExprContext) -> Self {
            Self { ctx, expr_context }
        }

        pub fn expr_context(&self) -> ExprContext {
            self.expr_context
        }
    }

    impl<'a> FromParentContext<'a, Context, ExprContext> for ExprState<'a> {
        fn create_from_parent_context(parent: &'a mut Context, value: ExprContext) -> Self {
            Self::new(parent, value)
        }
    }

    impl<'a> Deref for ExprState<'a> {
        type Target = Context;

        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a> DerefMut for ExprState<'a> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }
}

mod pos_expr_state {

    //
    // PosExprState
    //

    use crate::converter::expr_rules::expr_state::ExprState;
    use crate::converter::traits::FromParentContext;
    use rusty_common::{HasPos, Position};
    use std::ops::{Deref, DerefMut};

    pub struct PosExprState<'a, 'b> {
        ctx: &'a mut ExprState<'b>,
        pos: Position,
    }

    impl<'a, 'b> PosExprState<'a, 'b> {
        pub fn new(ctx: &'a mut ExprState<'b>, pos: Position) -> Self {
            Self { ctx, pos }
        }
    }

    impl<'a, 'b> FromParentContext<'a, ExprState<'b>, Position> for PosExprState<'a, 'b> {
        fn create_from_parent_context(parent: &'a mut ExprState<'b>, value: Position) -> Self {
            Self::new(parent, value)
        }
    }

    impl<'a, 'b> Deref for PosExprState<'a, 'b> {
        type Target = ExprState<'b>;

        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a, 'b> DerefMut for PosExprState<'a, 'b> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }

    impl<'a, 'b> HasPos for PosExprState<'a, 'b> {
        fn pos(&self) -> Position {
            self.pos
        }
    }
}

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
