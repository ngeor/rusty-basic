use rusty_common::HasPos;
use rusty_common::Position;

use crate::converter::common::Context;
use crate::converter::common::ExprContext;
use crate::converter::common::FromParentContext;
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

//
// PosExprState
//

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
