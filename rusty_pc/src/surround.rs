use crate::{InputTrait, Parser, ParserErrorTrait};

pub fn surround<L, P, R>(
    left: L,
    main: P,
    right: R,
    mode: SurroundMode,
) -> SurroundParser<P, L, R> {
    SurroundParser::new(main, left, right, mode)
}

pub struct SurroundParser<P, L, R> {
    parser: P,
    left: L,
    right: R,
    mode: SurroundMode,
}

#[derive(PartialEq, Eq)]
pub enum SurroundMode {
    /// The boundaries are optional.
    /// Parsing the main content will be attempted even if the left boundary is missing.
    /// If the main content is missing, a soft error is returned,
    /// and the left boundary is reverted (if one existed).
    ///
    /// Typical use case: padded by optional whitespace.
    Optional,

    /// The boundaries are mandatory.
    /// If the left boundary is missing, a soft error is returned.
    /// If the right boundary is missing, a fatal error is returned.
    /// If the main content is missing, a fatal error is returned.
    Mandatory,
    // TODO add MandatoryOrDefault to return the default content if it is missing
}

impl<P, L, R> SurroundParser<P, L, R> {
    pub fn new(parser: P, left: L, right: R, mode: SurroundMode) -> Self {
        Self {
            parser,
            left,
            right,
            mode,
        }
    }
}

impl<I, C, P, L, R> Parser<I, C> for SurroundParser<P, L, R>
where
    I: InputTrait,
    I: InputTrait,
    P: Parser<I, C>,
    L: Parser<I, C, Error = P::Error>,
    R: Parser<I, C, Error = P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let original_input = input.get_position();

        // parse the left boundary
        if let Err(err) = self.left.parse(input)
            && (err.is_fatal() || self.mode == SurroundMode::Mandatory)
        {
            return Err(err);
        }

        // parse the main content
        let result = match self.parser.parse(input) {
            Ok(result) => result,
            Err(err) => {
                if err.is_fatal() || self.mode == SurroundMode::Mandatory {
                    // the content is mandatory, convert to fatal
                    return Err(err.to_fatal());
                } else {
                    // the content is optional, return original input (undo parsing left boundary)
                    input.set_position(original_input);
                    return Err(err);
                }
            }
        };

        // parse the right boundary
        if let Err(err) = self.right.parse(input)
            && (err.is_fatal() || self.mode == SurroundMode::Mandatory)
        {
            // convert the missing right boundary into a fatal error!
            return Err(err.to_fatal());
        }

        Ok(result)
    }
}
