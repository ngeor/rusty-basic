//
// Many
//

use crate::common::QError;
use crate::parser::pc::{Parser, ParserBase, Tokenizer};

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

impl<P> Parser for ManyParser<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = Vec::new();
        loop {
            match self.parser.parse_opt(tokenizer)? {
                Some(value) => {
                    result.push(value);
                }
                None => {
                    break;
                }
            }
        }
        if result.is_empty() && !self.allow_empty {
            Err(QError::Incomplete)
        } else {
            Ok(result)
        }
    }
}
