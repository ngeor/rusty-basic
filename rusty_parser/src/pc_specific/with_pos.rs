use rusty_common::{AtPos, HasPos, Positioned};
use rusty_pc::{InputTrait, Parser, SetContext};

pub trait WithPos<I, C>: Parser<I, C>
where
    Self: Sized,
    I: HasPos + InputTrait,
{
    fn with_pos(self) -> impl Parser<I, C, Output = Positioned<Self::Output>, Error = Self::Error> {
        WithPosMapper::new(self)
    }
}
impl<I, C, P> WithPos<I, C> for P
where
    P: Parser<I, C>,
    I: HasPos + InputTrait,
{
}

struct WithPosMapper<P> {
    parser: P,
}

impl<P> WithPosMapper<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> Parser<I, C> for WithPosMapper<P>
where
    P: Parser<I, C>,
    I: HasPos + InputTrait,
{
    type Output = Positioned<P::Output>;
    type Error = P::Error;
    fn parse(&mut self, tokenizer: &mut I) -> Result<Self::Output, Self::Error> {
        let pos = tokenizer.pos();
        self.parser.parse(tokenizer).map(|x| x.at_pos(pos))
    }
}

impl<C, P> SetContext<C> for WithPosMapper<P>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.parser.set_context(ctx)
    }
}
