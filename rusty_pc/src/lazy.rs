use std::cell::RefCell;

use crate::Parser;

pub fn lazy<I, C, F, P>(factory: F) -> impl Parser<I, C, Output = P::Output, Error = P::Error>
where
    F: Fn() -> P,
    P: Parser<I, C>,
{
    LazyParser {
        factory,
        parser: RefCell::new(None),
    }
}

struct LazyParser<F, P> {
    factory: F,
    parser: RefCell<Option<P>>,
}

impl<I, C, F, P> Parser<I, C> for LazyParser<F, P>
where
    F: Fn() -> P,
    P: Parser<I, C>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: I) -> crate::ParseResult<I, Self::Output, Self::Error> {
        if self.parser.borrow().is_none() {
            let parser = (self.factory)();
            *self.parser.borrow_mut() = Some(parser);
            self.parser.borrow().as_ref().unwrap().parse(input)
        } else {
            self.parser.borrow().as_ref().unwrap().parse(input)
        }
    }

    fn set_context(&mut self, _ctx: C) {
        unimplemented!()
    }
}
