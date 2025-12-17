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

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, crate::ParseError> {
        self.left
            .parse(tokenizer)
            .flat_map(|left| match self.parser.parse(tokenizer) {
                ParseResult::Ok(value) => match self.right.parse(tokenizer) {
                    ParseResult::Err(err) => ParseResult::Err(err),
                    _ => ParseResult::Ok(value),
                },
                ParseResult::None => {
                    left.undo(tokenizer);
                    ParseResult::None
                }
                ParseResult::Err(err) => {
                    if err.is_incomplete() {
                        left.undo(tokenizer);
                    }
                    ParseResult::Err(err)
                }
            })
    }
}
