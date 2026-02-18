use crate::map_decorator::{MapDecorator, MapDecoratorMarker};
use crate::{InputTrait, Parser};

pub fn lazy<I, C, F, P>(factory: F) -> impl Parser<I, C, Output = P::Output, Error = P::Error>
where
    F: Fn() -> P,
    I: InputTrait,
    P: Parser<I, C>,
{
    LazyParser {
        factory,
        parser_holder: None,
    }
}

struct LazyParser<F, P> {
    factory: F,
    parser_holder: Option<P>,
}

impl<I, C, F, P> MapDecorator<I, C> for LazyParser<F, P>
where
    F: Fn() -> P,
    I: InputTrait,
    P: Parser<I, C>,
{
    type OriginalOutput = P::Output;
    type Output = P::Output;
    type Error = P::Error;

    fn decorated(
        &mut self,
    ) -> &mut impl Parser<I, C, Output = Self::OriginalOutput, Error = Self::Error> {
        if self.parser_holder.is_none() {
            let parser = (self.factory)();
            self.parser_holder = Some(parser);
        }
        self.parser_holder.as_mut().unwrap()
    }

    fn map_ok(&self, ok: Self::OriginalOutput) -> Result<Self::Output, Self::Error> {
        Ok(ok)
    }
}

impl<F, P> MapDecoratorMarker for LazyParser<F, P> {}
