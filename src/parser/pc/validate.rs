use crate::common::QError;
use crate::parser::pc::*;
use crate::parser_declaration;

parser_declaration!(struct ValidateParser<validator: F>);

impl<P, F> Parser for ValidateParser<P, F>
where
    P: Parser,
    P::Output: Undo,
    F: Fn(&P::Output) -> Result<bool, QError>,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let value = self.parser.parse(tokenizer)?;
        let should_keep: bool = (self.validator)(&value)?;
        if should_keep {
            Ok(value)
        } else {
            value.undo(tokenizer);
            Err(QError::Incomplete)
        }
    }
}
