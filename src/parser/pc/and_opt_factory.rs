//
// AndOptFactory
//

use crate::common::QError;
use crate::parser::pc::{Parser, ParserBase, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct AndOptFactoryPC<right_factory: RF>);

impl<L, RF, R> ParserBase for AndOptFactoryPC<L, RF>
where
    L: Parser,
    RF: Fn(&L::Output) -> R,
    R: Parser,
{
    type Output = (L::Output, Option<R::Output>);
}

impl<L, RF, R> Parser for AndOptFactoryPC<L, RF>
where
    L: Parser,
    RF: Fn(&L::Output) -> R,
    R: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let first = self.parser.parse(tokenizer)?;
        let second_parser = (self.right_factory)(&first);
        let opt_second = second_parser.parse_opt(tokenizer)?;
        Ok((first, opt_second))
    }
}
