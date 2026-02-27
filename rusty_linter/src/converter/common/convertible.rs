use rusty_common::{AtPos, Position, Positioned};

use crate::core::{LintErrorPos, LinterContext};

/// Convert from the current type into the target type O.
/// By default, O is the same as the current type.
pub trait Convertible<O = Self>: Sized {
    fn convert(self, ctx: &mut LinterContext) -> Result<O, LintErrorPos>;
}

// Blanket implementation for Option

impl<T, O> Convertible<Option<O>> for Option<T>
where
    T: Convertible<O>,
{
    fn convert(self, ctx: &mut LinterContext) -> Result<Option<O>, LintErrorPos> {
        match self {
            Some(t) => t.convert(ctx).map(Some),
            None => Ok(None),
        }
    }
}

// Blanket implementation for Vec

impl<T, O> Convertible<Vec<O>> for Vec<T>
where
    T: Convertible<O>,
{
    fn convert(self, ctx: &mut LinterContext) -> Result<Vec<O>, LintErrorPos> {
        self.into_iter().map(|t| t.convert(ctx)).collect()
    }
}

// Blanket implementation for Positioned in combination with the next trait

impl<T, O> Convertible<Positioned<O>> for Positioned<T>
where
    T: ConvertibleIn<Position, O>,
{
    fn convert(self, ctx: &mut LinterContext) -> Result<Positioned<O>, LintErrorPos> {
        let Self {
            element: statement,
            pos,
        } = self;
        statement
            .convert_in(ctx, pos)
            .map(|converted| converted.at_pos(pos))
    }
}

/// Convert from the current type into the target type O,
/// using additional information in the value U.
/// By default, O is the same as the current type.
pub trait ConvertibleIn<U, O = Self>: Sized {
    fn convert_in(self, ctx: &mut LinterContext, value: U) -> Result<O, LintErrorPos>;

    fn convert_in_default(self, ctx: &mut LinterContext) -> Result<O, LintErrorPos>
    where
        U: Default,
    {
        self.convert_in(ctx, U::default())
    }
}

// Blanket implementation for Option

impl<U, T, O> ConvertibleIn<U, Option<O>> for Option<T>
where
    T: ConvertibleIn<U, O>,
{
    fn convert_in(self, ctx: &mut LinterContext, extra: U) -> Result<Option<O>, LintErrorPos> {
        match self {
            Some(t) => t.convert_in(ctx, extra).map(Some),
            None => Ok(None),
        }
    }
}

// Blanket implementation for Vec

impl<U, T, O> ConvertibleIn<U, Vec<O>> for Vec<T>
where
    T: ConvertibleIn<U, O>,
    U: Clone,
{
    fn convert_in(self, ctx: &mut LinterContext, extra: U) -> Result<Vec<O>, LintErrorPos> {
        self.into_iter()
            .map(|t| t.convert_in(ctx, extra.clone()))
            .collect()
    }
}
