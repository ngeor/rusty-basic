use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct AndDemandLookingBack<right_factory: RF>);

impl<L, F, R> ParserBase for AndDemandLookingBack<L, F>
where
    L: ParserBase,
    R: ParserBase,
    F: Fn(&L::Output) -> R,
{
    type Output = (L::Output, R::Output);
}

impl<L, F, R> OptParser for AndDemandLookingBack<L, F>
where
    L: OptParser,
    R: NonOptParser,
    F: Fn(&L::Output) -> R,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(first) => {
                let right_parser = (self.right_factory)(&first);
                let second = right_parser.parse_non_opt(tokenizer)?;
                Ok(Some((first, second)))
            }
            None => Ok(None),
        }
    }
}
