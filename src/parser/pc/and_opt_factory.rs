//
// AndOptFactory
//

use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(pub struct AndOptFactoryPC<right_factory: RF>);

impl<L, RF, R> Parser for AndOptFactoryPC<L, RF>
where
    L: Parser,
    RF: Fn(&L::Output) -> R,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.parser.parse(tokenizer)?;
        let second_parser = (self.right_factory)(&first);
        let opt_second = second_parser.parse_opt(tokenizer)?;
        Ok((first, opt_second))
    }
}
