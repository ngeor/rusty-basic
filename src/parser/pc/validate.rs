use crate::common::QError;
use crate::parser::pc::*;
use crate::parser_decorator;

parser_decorator!(struct ValidateParser<validator: F>);

impl<P, F> Parser for ValidateParser<P, F>
where
    P: Parser,
    P::Output: Undo,
    F: Fn(&P::Output) -> Result<bool, QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => {
                let should_keep: bool = (self.1)(&value)?;
                if should_keep {
                    Ok(Some(value))
                } else {
                    value.undo(tokenizer);
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
