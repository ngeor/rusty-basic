use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::FromParentContext;
use crate::linter::converter::types::DimContext;
use std::ops::{Deref, DerefMut};

pub struct DimListState<'a> {
    ctx: &'a mut Context,
    dim_context: DimContext,
}

impl<'a> DimListState<'a> {
    pub fn new(ctx: &'a mut Context, dim_context: DimContext) -> Self {
        Self { ctx, dim_context }
    }

    pub fn dim_context(&self) -> DimContext {
        self.dim_context
    }
}

impl<'a> FromParentContext<'a, Context, DimContext> for DimListState<'a> {
    fn create_from_parent_context(parent: &'a mut Context, value: DimContext) -> Self {
        Self::new(parent, value)
    }
}

impl<'a> Deref for DimListState<'a> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a> DerefMut for DimListState<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}
