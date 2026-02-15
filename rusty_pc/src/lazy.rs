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

impl<I, C, F, P> Parser<I, C> for LazyParser<F, P>
where
    F: Fn() -> P,
    I: InputTrait,
    P: Parser<I, C>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        match self.parser_holder.as_mut() {
            Some(parser) => parser.parse(input),
            None => {
                let mut parser = (self.factory)();
                let result = parser.parse(input);
                self.parser_holder = Some(parser);
                result
            }
        }
    }

    fn set_context(&mut self, _ctx: &C) {
        unimplemented!()
    }
}
