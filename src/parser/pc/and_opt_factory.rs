//
// AndOptFactory
//

use crate::common::QError;
use crate::parser::pc::{OptParser, ParserBase, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct AndOptFactoryPC<right_factory: RF>);

impl<L, RF, R> ParserBase for AndOptFactoryPC<L, RF>
where
    L: OptParser,
    RF: Fn(&L::Output) -> R,
    R: OptParser,
{
    type Output = (L::Output, Option<R::Output>);
}

impl<L, RF, R> OptParser for AndOptFactoryPC<L, RF>
where
    L: OptParser,
    RF: Fn(&L::Output) -> R,
    R: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(first) => {
                let second_parser = (self.right_factory)(&first);
                let opt_second = second_parser.parse(tokenizer)?;
                Ok(Some((first, opt_second)))
            }
            None => Ok(None),
        }
    }
}
