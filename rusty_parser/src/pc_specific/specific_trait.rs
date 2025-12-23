use rusty_common::{HasPos, Positioned};

use crate::error::ParseError;
use crate::pc::{Errors, Parser};
use crate::pc_specific::WithPosMapper;

pub trait SpecificTrait<I: HasPos>: Parser<I>
where
    Self: Sized,
{
    fn or_syntax_error(self, msg: &str) -> impl Parser<I, Output = Self::Output> {
        self.or_fail(ParseError::syntax_error(msg))
    }

    fn with_pos(self) -> impl Parser<I, Output = Positioned<Self::Output>> {
        WithPosMapper::new(self)
    }
}

impl<I: HasPos, P> SpecificTrait<I> for P where P: Parser<I> {}
