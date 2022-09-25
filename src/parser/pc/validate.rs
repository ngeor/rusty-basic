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

// TODO make a macro for the traits too
// TODO change these blanket implementations so that not every single type gets the methods

pub trait ValidateTrait<F>
where
    Self: Sized + HasOutput,
    F: Fn(&Self::Output) -> Result<bool, QError>,
{
    fn validate(self, f: F) -> ValidateParser<Self, F>;
}

impl<P, F> ValidateTrait<F> for P
where
    P: HasOutput,
    F: Fn(&P::Output) -> Result<bool, QError>,
{
    fn validate(self, f: F) -> ValidateParser<Self, F> {
        ValidateParser::new(self, f)
    }
}
