use crate::pc::{OrFailParser, Parser};
use crate::pc_specific::WithPosMapper;
use crate::ParseError;

pub trait SpecificTrait<I>: Parser<I>
where
    Self: Sized,
{
    fn or_syntax_error(self, msg: &str) -> OrFailParser<Self>;

    fn with_pos(self) -> WithPosMapper<Self>
    where
        Self: Sized;
}

impl<I, S> SpecificTrait<I> for S
where
    S: Parser<I>,
{
    fn or_syntax_error(self, msg: &str) -> OrFailParser<Self> {
        self.or_fail(ParseError::syntax_error(msg))
    }

    fn with_pos(self) -> WithPosMapper<Self> {
        WithPosMapper::new(self)
    }
}
