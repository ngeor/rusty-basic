use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::*;

binary_parser_declaration!(struct GuardPC);

impl<L, R> ParserBase for GuardPC<L, R>
where
    R: ParserBase,
{
    type Output = R::Output;
}

impl<L, R> OptParser for GuardPC<L, R>
where
    L: OptParser,
    R: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(_) => self.1.parse(tokenizer).map(Some),
            None => Ok(None),
        }
    }
}
