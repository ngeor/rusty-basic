use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{OptParser, ParserBase, Tokenizer, Undo};

//
// And (with undo if the left parser supports it)
//

// Looks identical to `NonOptSeq2` but that one has already an implementation
// of Parser

binary_parser_declaration!(struct AndPC);

impl<A, B> ParserBase for AndPC<A, B>
where
    A: ParserBase,
    B: ParserBase,
{
    type Output = (A::Output, B::Output);
}

impl<A, B> OptParser for AndPC<A, B>
where
    A: OptParser,
    A::Output: Undo,
    B: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(left) => match self.1.parse(tokenizer)? {
                Some(right) => Ok(Some((left, right))),
                None => {
                    left.undo(tokenizer);
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }
}
