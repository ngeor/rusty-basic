// Mixed type or

use crate::{ParseResult, Parser, SetContext};

pub enum ZipValue<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

impl<L, R> ZipValue<L, R> {
    #[allow(dead_code)]
    pub fn has_left(&self) -> bool {
        matches!(self, Self::Left(_) | Self::Both(_, _))
    }

    pub fn has_right(&self) -> bool {
        matches!(self, Self::Right(_) | Self::Both(_, _))
    }

    pub fn left(self) -> Option<L> {
        match self {
            Self::Left(left) | Self::Both(left, _) => Some(left),
            _ => None,
        }
    }

    pub fn right(self) -> Option<R> {
        match self {
            Self::Right(right) | Self::Both(_, right) => Some(right),
            _ => None,
        }
    }

    pub fn collect_right(items: Vec<Self>) -> Vec<R> {
        items
            .into_iter()
            .flat_map(|zip_value| zip_value.right().into_iter())
            .collect()
    }
}

pub struct OptZip<L, R> {
    left: L,
    right: R,
}
impl<L, R> OptZip<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}
impl<C, L, R> SetContext<C> for OptZip<L, R>
where
    C: Clone,
    L: SetContext<C>,
    R: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.left.set_context(ctx.clone());
        self.right.set_context(ctx);
    }
}

pub fn opt_zip<L, R>(left: L, right: R) -> OptZip<L, R> {
    OptZip::new(left, right)
}

impl<I, C, L, R> Parser<I, C> for OptZip<L, R>
where
    L: Parser<I, C>,
    R: Parser<I, C, Error = L::Error>,
    C: Clone,
{
    type Output = ZipValue<L::Output, R::Output>;
    type Error = L::Error;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        let (tokenizer, opt_left) = match self.left.parse(tokenizer) {
            Ok((input, x)) => (input, Some(x)),
            Err((false, input, _)) => (input, None),
            Err(err) => return Err(err),
        };
        match self.right.parse(tokenizer) {
            Ok((input, r)) => match opt_left {
                Some(l) => Ok((input, ZipValue::Both(l, r))),
                None => Ok((input, ZipValue::Right(r))),
            },
            Err((false, input, err)) => match opt_left {
                Some(l) => Ok((input, ZipValue::Left(l))),
                None => Err((false, input, err)),
            },
            Err(err) => Err(err),
        }
    }
}
