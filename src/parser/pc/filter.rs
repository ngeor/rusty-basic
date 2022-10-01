use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer, Undo};
use crate::parser_declaration;

parser_declaration!(struct FilterParser<predicate: F>);

impl<P, F> Parser for FilterParser<P, F>
where
    P: Parser,
    F: Fn(&P::Output) -> bool,
    P::Output: Undo,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let result = self.parser.parse(tokenizer)?;
        if (self.predicate)(&result) {
            Ok(result)
        } else {
            result.undo(tokenizer);
            Err(QError::Incomplete)
        }
    }
}
