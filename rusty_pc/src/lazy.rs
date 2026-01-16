use crate::Parser;

pub fn lazy<I, C, F, P>(factory: F) -> impl Parser<I, C, Output = P::Output, Error = P::Error>
where
    F: Fn() -> P,
    P: Parser<I, C>,
{
    LazyParser {
        factory,
        parser: None,
    }
}

struct LazyParser<F, P> {
    factory: F,
    parser: Option<P>,
}

impl<I, C, F, P> Parser<I, C> for LazyParser<F, P>
where
    F: Fn() -> P,
    P: Parser<I, C>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: I) -> crate::ParseResult<I, Self::Output, Self::Error> {
        if self.parser.is_none() {
            let parser = (self.factory)();
            self.parser = Some(parser);
            self.parser.as_mut().unwrap().parse(input)
        } else {
            self.parser.as_mut().unwrap().parse(input)
        }
    }
}
