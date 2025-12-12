use crate::{pc::*, ParserErrorTrait};

pub struct SurroundParser<L, P, R> {
    left: L,
    parser: P,
    right: R,
}

impl<L, P, R> SurroundParser<L, P, R> {
    pub fn new(left: L, parser: P, right: R) -> Self {
        Self {
            left,
            parser,
            right,
        }
    }
}

impl<I: Tokenizer + 'static, L, P, R> Parser<I> for SurroundParser<L, P, R>
where
    L: Parser<I>,
    L::Output: Undo,
    P: Parser<I>,
    R: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> Result<Self::Output, crate::ParseError> {
        let left = self.left.parse(tokenizer)?;

        match self.parser.parse(tokenizer) {
            Ok(value) => {
                self.right.parse(tokenizer)?;
                Ok(value)
            }
            Err(err) => {
                if err.is_incomplete() {
                    left.undo(tokenizer);
                }
                Err(err)
            }
        }
    }
}
