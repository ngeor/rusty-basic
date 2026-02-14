use std::marker::PhantomData;

use crate::many::ManyCombiner;
use crate::{InputTrait, Parser, ParserErrorTrait};

/// Collects multiple values from the underlying parser as long as parsing succeeds.
/// The context of the underlying parser is set after every iteration,
/// so that it is aware of the previously parsed element.
pub struct ManyCtxParser<P, F, G, O, CIn> {
    parser: P,
    combiner: F,
    context_projection: G,
    allow_none: bool,
    _marker: PhantomData<(O, CIn)>,
}

impl<P, F, G, O, CIn> ManyCtxParser<P, F, G, O, CIn> {
    pub fn new<I>(parser: P, combiner: F, context_projection: G, allow_none: bool) -> Self
    where
        I: InputTrait,
        P: Parser<I, CIn>,
        F: ManyCombiner<P::Output, O>,
        G: Fn(&P::Output) -> CIn,
        O: Default,
    {
        Self {
            parser,
            combiner,
            context_projection,
            allow_none,
            _marker: PhantomData,
        }
    }
}

impl<I, COut, CIn, P, F, G, O> Parser<I, COut> for ManyCtxParser<P, F, G, O, CIn>
where
    I: InputTrait,
    CIn: Default,
    P: Parser<I, CIn>,
    F: ManyCombiner<P::Output, O>,
    G: Fn(&P::Output) -> CIn,
    O: Default,
{
    type Output = O;
    type Error = P::Error;
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        // set the default context before parsing begins
        self.parser.set_context(CIn::default());
        match self.parser.parse(input) {
            Ok(first_value) => {
                // set the context to the underlying parser
                self.parser
                    .set_context((self.context_projection)(&first_value));
                // seed the result
                let mut result = self.combiner.seed(first_value);
                loop {
                    match self.parser.parse(input) {
                        Ok(value) => {
                            // set the context of the underlying parser to the current value
                            self.parser.set_context((self.context_projection)(&value));
                            // accumulate the result
                            result = self.combiner.accumulate(result, value);
                        }
                        Err(err) if err.is_soft() => {
                            break;
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
                Ok(result)
            }
            Err(err) if err.is_soft() => {
                if self.allow_none {
                    Ok(O::default())
                } else {
                    Err(err)
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set_context(&mut self, _ctx: COut) {}
}
