use crate::binary_parser_declaration;
use crate::common::QError;
use crate::parser::pc::{Parser, ParserBase, Tokenizer, Undo};

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

impl<A, B> Parser for AndPC<A, B>
where
    A: Parser,
    A::Output: Undo,
    B: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let left = self.0.parse(tokenizer)?;
        if let Some(right) = self.1.parse_opt(tokenizer)? {
            Ok((left, right))
        } else {
            left.undo(tokenizer);
            Err(QError::Incomplete)
        }
    }
}
