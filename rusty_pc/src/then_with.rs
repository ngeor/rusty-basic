use crate::{ParseResult, Parser, SetContext};

pub trait ThenWithContext<I, C>: Parser<I, C> {
    fn then_with_in_context<F, R, RP, CR>(
        self,
        ctx_projection: F,
        other: R,
    ) -> ThenWithContextParser<Self, R, F>
    where
        Self: Sized,
        F: Fn(&Self::Output) -> CR,
        R: Fn() -> RP,
        RP: Parser<I, CR, Error = Self::Error>,
    {
        ThenWithContextParser::new(self, other, ctx_projection)
    }
}

impl<I, C, P> ThenWithContext<I, C> for P where P: Parser<I, C> {}

pub struct ThenWithContextParser<L, R, F> {
    left: L,
    right: R,
    ctx_projection: F,
}
impl<L, R, F> ThenWithContextParser<L, R, F> {
    pub fn new(left: L, right: R, ctx_projection: F) -> Self {
        Self {
            left,
            right,
            ctx_projection,
        }
    }
}

impl<I, C, L, R, RP, F, CR> Parser<I, C> for ThenWithContextParser<L, R, F>
where
    L: Parser<I, C>,
    R: Fn() -> RP,
    RP: Parser<I, CR, Error = L::Error> + SetContext<CR>,
    F: Fn(&L::Output) -> CR,
{
    type Output = (L::Output, RP::Output);
    type Error = L::Error;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.left.parse(input) {
            Ok((input, left)) => {
                let ctx = (self.ctx_projection)(&left);
                let mut right = (self.right)();
                right.set_context(ctx);
                match right.parse(input) {
                    Ok((input, right)) => Ok((input, (left, right))),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }
}
