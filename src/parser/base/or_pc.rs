use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::Tokenizer;

//
// Or
//

pub struct OrPC<L, R>(L, R);

impl<L, R> HasOutput for OrPC<L, R>
where
    L: HasOutput,
    R: HasOutput<Output = L::Output>,
{
    type Output = L::Output;
}

impl<L, R> Parser for OrPC<L, R>
where
    L: Parser,
    R: Parser<Output = L::Output>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        or_parse(&self.0, &self.1, tokenizer)
    }
}

fn or_parse<A, B>(a: &A, b: &B, tokenizer: &mut impl Tokenizer) -> Result<Option<A::Output>, QError>
where
    A: Parser,
    B: Parser<Output = A::Output>,
{
    match a.parse(tokenizer)? {
        Some(first) => Ok(Some(first)),
        None => b.parse(tokenizer),
    }
}

// Or3

pub struct Or3PC<A, B, C>(A, B, C);

impl<A, B, C> HasOutput for Or3PC<A, B, C>
where
    A: HasOutput,
    B: HasOutput<Output = A::Output>,
    C: HasOutput<Output = A::Output>,
{
    type Output = A::Output;
}

impl<A, B, C> Parser for Or3PC<A, B, C>
where
    A: Parser,
    B: Parser<Output = A::Output>,
    C: Parser<Output = A::Output>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(first) => Ok(Some(first)),
            None => or_parse(&self.1, &self.2, tokenizer),
        }
    }
}

// OrTrait

pub trait OrTrait<P> {
    fn or(self, other: P) -> OrPC<Self, P>
    where
        Self: Sized;
}

impl<S, P> OrTrait<P> for S
where
    S: Parser,
    P: Parser<Output = S::Output>,
{
    fn or(self, other: P) -> OrPC<Self, P> {
        OrPC(self, other)
    }
}

pub fn alt2<L, R>(left: L, right: R) -> OrPC<L, R> {
    OrPC(left, right)
}

pub fn alt3<A, B, C>(a: A, b: B, c: C) -> Or3PC<A, B, C> {
    Or3PC(a, b, c)
}

pub fn alt4<A, B, C, D>(a: A, b: B, c: C, d: D) -> OrPC<A, Or3PC<B, C, D>> {
    alt2(a, alt3(b, c, d))
}

pub fn alt5<A, B, C, D, E>(a: A, b: B, c: C, d: D, e: E) -> OrPC<OrPC<A, B>, Or3PC<C, D, E>> {
    alt2(alt2(a, b), alt3(c, d, e))
}

pub fn alt6<A, B, C, D, E, F>(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
) -> OrPC<Or3PC<A, B, C>, Or3PC<D, E, F>> {
    alt2(alt3(a, b, c), alt3(d, e, f))
}
