use crate::common::QError;
use crate::parser::pc::{OrFailParser, Parser};
use crate::parser::pc_specific::WithPosMapper;

pub trait SpecificTrait: Parser
where
    Self: Sized,
{
    fn or_syntax_error(self, msg: &str) -> OrFailParser<Self>;

    fn with_pos(self) -> WithPosMapper<Self>
    where
        Self: Sized;
}

impl<S> SpecificTrait for S
where
    S: Parser,
{
    fn or_syntax_error(self, msg: &str) -> OrFailParser<Self> {
        self.or_fail(QError::syntax_error(msg))
    }

    fn with_pos(self) -> WithPosMapper<Self> {
        WithPosMapper::new(self)
    }
}
