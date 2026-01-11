use crate::{ParseResult, Parser};

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
    /// If the main content is missing, a non-fatal result is returned,
    /// and the left boundary is reverted (if one existed).
    ///
    /// Typical use case: padded by optional whitespace.
    Optional,

    /// The boundaries are mandatory.
    /// If the left boundary is missing, a non-fatal error is returned.
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
    I: Clone,
    P: Parser<I, C>,
    L: Parser<I, C, Error = P::Error>,
    R: Parser<I, C, Error = P::Error>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        let original_input = input.clone();

        // parse the left boundary
        let input = match self.left.parse(input) {
            Ok((input, _)) => input,
            Err((false, input, err)) => {
                // if boundaries are optional, allow it
                if self.mode == SurroundMode::Optional {
                    input
                } else {
                    return Err((false, input, err));
                }
            }
            Err(err) => {
                return Err(err);
            }
        };

        // parse the main content
        let (input, result) = match self.parser.parse(input) {
            Ok((input, result)) => (input, result),
            Err((false, input, err)) => {
                if self.mode == SurroundMode::Mandatory {
                    // the content is mandatory, convert to fatal
                    return Err((true, input, err));
                } else {
                    // the content is optional, return original input (undo parsing left boundary)
                    return Err((false, original_input, err));
                }
            }
            Err(err) => {
                return Err(err);
            }
        };

        // parse the right boundary
        let input = match self.right.parse(input) {
            Ok((input, _)) => input,
            Err((false, input, err)) => {
                if self.mode == SurroundMode::Optional {
                    // allow missing optional boundary
                    input
                } else {
                    // convert the missing right boundary into a fatal error!
                    return Err((true, input, err));
                }
            }
            Err(err) => {
                return Err(err);
            }
        };

        Ok((input, result))
    }
}
