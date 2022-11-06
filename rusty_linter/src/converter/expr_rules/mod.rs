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
use rusty_parser::*;

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
        let Positioned { element: expr, pos } = self;
        expr.convert_in(ctx, pos)
            .map(|expr| expr.at_pos(pos))
            .patch_err_pos(&pos)
    }
}

impl<'a> Convertible<ExprState<'a>> for Box<ExpressionPos> {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, LintErrorPos> {
        let unboxed = *self;
        unboxed.convert(ctx).map(Box::new)
    }
}

//
// Expression Convertible
//

impl<'a, 'b> Convertible<PosExprState<'a, 'b>> for Expression {
    fn convert(self, ctx: &mut PosExprState<'a, 'b>) -> Result<Self, LintErrorPos> {
        match self {
            // literals
            Expression::SingleLiteral(_)
            | Expression::DoubleLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::LongLiteral(_) => Ok(self),
            // parenthesis
            Expression::Parenthesis(box_child) => {
                box_child.convert(ctx).map(Expression::Parenthesis)
            }
            // unary
            Expression::UnaryExpression(unary_operator, box_child) => {
                unary::convert(ctx, unary_operator, *box_child)
            }
            // binary
            Expression::BinaryExpression(binary_operator, left, right, _expr_type) => {
                binary::convert(ctx, binary_operator, *left, *right)
            }
            // variables
            Expression::Variable(name, variable_info) => {
                variable::convert(ctx, name, variable_info)
            }
            Expression::ArrayElement(_name, _indices, _variable_info) => {
                panic!(
                    "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
                )
            }
            Expression::Property(box_left_side, property_name, _expr_type) => {
                property::convert(ctx, box_left_side, property_name)
            }
            // function call
            Expression::FunctionCall(name, args) => function::convert(ctx, name, args),
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                built_in_function::convert(ctx, built_in_function, args)
            }
        }
    }
}
