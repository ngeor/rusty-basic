use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct AndDemandLookingBack<right_factory: RF>);

impl<L, F, R> Parser for AndDemandLookingBack<L, F>
where
    L: Parser,
    R: Parser,
    F: Fn(&L::Output) -> R,
{
    type Output = (L::Output, R::Output);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.parser.parse(tokenizer)?;
        let right_parser = (self.right_factory)(&first);
        let second = right_parser.parse(tokenizer).no_incomplete()?;
        Ok((first, second))
    }
}
