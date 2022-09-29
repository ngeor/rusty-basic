//
// Many
//

use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Tokenizer};

pub struct ManyParser<P> {
    parser: P,
    allow_empty: bool,
}

impl<P> ManyParser<P> {
    pub fn new(parser: P, allow_empty: bool) -> Self {
        Self {
            parser,
            allow_empty,
        }
    }
}

impl<P> ParserBase for ManyParser<P>
where
    P: ParserBase,
{
    type Output = Vec<P::Output>;
}

impl<P> OptParser for ManyParser<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse(tokenizer)? {
                Some(value) => {
                    result.push(value);
                }
                None => {
                    break;
                }
            }
        }
        if result.is_empty() && !self.allow_empty {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

impl<P> NonOptParser for ManyParser<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse(tokenizer)? {
                Some(value) => {
                    result.push(value);
                }
                None => {
                    break;
                }
            }
        }
        if result.is_empty() && !self.allow_empty {
            Err(QError::ArgumentCountMismatch)
        } else {
            Ok(result)
        }
    }
}
