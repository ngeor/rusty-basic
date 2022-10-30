use crate::converter::context::Context;
use crate::converter::traits::FromParentContext;
use rusty_common::{HasLocation, Location};
use std::ops::{Deref, DerefMut};

pub struct PosContext<'a> {
    ctx: &'a mut Context,
    pos: Location,
}

impl<'a> FromParentContext<'a, Context, Location> for PosContext<'a> {
    fn create_from_parent_context(ctx: &'a mut Context, pos: Location) -> Self {
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

impl<'a> HasLocation for PosContext<'a> {
    fn pos(&self) -> Location {
        self.pos
    }
}
