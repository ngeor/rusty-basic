use crate::converter::context::Context;
use crate::converter::traits::FromParentContext;
use rusty_common::{HasPos, Position};
use std::ops::{Deref, DerefMut};

pub struct PosContext<'a> {
    ctx: &'a mut Context,
    pos: Position,
}

impl<'a> FromParentContext<'a, Context, Position> for PosContext<'a> {
    fn create_from_parent_context(ctx: &'a mut Context, pos: Position) -> Self {
        Self { ctx, pos }
    }
}

impl<'a> Deref for PosContext<'a> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a> DerefMut for PosContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}

impl<'a> HasPos for PosContext<'a> {
    fn pos(&self) -> Position {
        self.pos
    }
}
