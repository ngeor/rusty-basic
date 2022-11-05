use crate::converter::context::Context;
use rusty_common::QErrorPos;

pub trait Convertible<C = Context, O = Self>: Sized {
    fn convert(self, ctx: &mut C) -> Result<O, QErrorPos>;

    fn convert_in<'a, ParentContext, U>(
        self,
        parent_ctx: &'a mut ParentContext,
        value: U,
    ) -> Result<O, QErrorPos>
    where
        C: FromParentContext<'a, ParentContext, U>,
    {
        let mut child_state = C::create_from_parent_context(parent_ctx, value);
        self.convert(&mut child_state)
    }

    fn convert_in_default<'a, ParentContext, U>(
        self,
        parent_ctx: &'a mut ParentContext,
    ) -> Result<O, QErrorPos>
    where
        C: FromParentContext<'a, ParentContext, U>,
        U: Default,
    {
        self.convert_in(parent_ctx, U::default())
    }
}

impl<C, T> Convertible<C> for Option<T>
where
    T: Convertible<C, T>,
{
    fn convert(self, ctx: &mut C) -> Result<Self, QErrorPos> {
        match self {
            Some(t) => t.convert(ctx).map(Some),
            None => Ok(None),
        }
    }
}

impl<C, T> Convertible<C> for Vec<T>
where
    T: Convertible<C, T>,
{
    fn convert(self, ctx: &mut C) -> Result<Self, QErrorPos> {
        self.into_iter().map(|t| t.convert(ctx)).collect()
    }
}

pub trait FromParentContext<'a, T, U> {
    fn create_from_parent_context(parent: &'a mut T, value: U) -> Self;
}
