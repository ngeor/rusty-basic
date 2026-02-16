use crate::{InputTrait, Parser, ParserErrorTrait};

/// A parser decorator that maps the successful result
/// and optionally the soft error
/// of the decorated parser.
pub trait MapDecorator<I, C>
where
    I: InputTrait,
{
    type OriginalOutput;
    type Output;
    type Error: ParserErrorTrait;

    /// Gets the decorated parser.
    fn decorated(
        &mut self,
    ) -> &mut impl Parser<I, C, Output = Self::OriginalOutput, Error = Self::Error>;

    /// Maps the successful result of the parser.
    fn map_ok(&self, ok: Self::OriginalOutput) -> Result<Self::Output, Self::Error>;

    /// Maps the soft error of the parser.
    /// By default, the error is returned as-is. However, it is possible to override this behavior.
    fn map_soft_error(&self, err: Self::Error) -> Result<Self::Output, Self::Error> {
        Err(err)
    }
}

/// Marker trait for `MapDecorator`.
/// Allows for blanket trait implementation of `Parser`.
pub trait MapDecoratorMarker {}

impl<I, C, D> Parser<I, C> for D
where
    I: InputTrait,
    D: MapDecoratorMarker + MapDecorator<I, C>,
{
    type Output = D::Output;
    type Error = D::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.decorated().parse(input) {
            Ok(ok) => self.map_ok(ok),
            Err(err) if err.is_soft() => self.map_soft_error(err),
            Err(err) => Err(err),
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.decorated().set_context(ctx)
    }
}
