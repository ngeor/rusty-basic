use crate::pc::{ParseResult, Parser, Tokenizer, Undo};
use crate::ParseError;

//
// And (with undo)
//

pub struct And<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> And<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I: Tokenizer + 'static, L, R, F, O> Parser<I> for And<L, R, F>
where
    L: Parser<I>,
    L::Output: Undo,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.left.parse(tokenizer) {
            ParseResult::Ok(left) => match self.right.parse(tokenizer) {
                ParseResult::Ok(right) => ParseResult::Ok((self.combiner)(left, right)),
                ParseResult::None => {
                    left.undo(tokenizer);
                    ParseResult::None
                }
                ParseResult::Expected(s) => {
                    left.undo(tokenizer);
                    ParseResult::Expected(s)
                }
                ParseResult::Err(err) => ParseResult::Err(err),
            },
            ParseResult::None => ParseResult::None,
            ParseResult::Expected(s) => ParseResult::Expected(s),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}

pub struct AndWithoutUndo<L, R, F> {
    left: L,
    right: R,
    combiner: F,
}
impl<L, R, F> AndWithoutUndo<L, R, F> {
    pub fn new(left: L, right: R, combiner: F) -> Self {
        Self {
            left,
            right,
            combiner,
        }
    }
}

impl<I: Tokenizer + 'static, L, R, F, O> Parser<I> for AndWithoutUndo<L, R, F>
where
    L: Parser<I>,
    R: Parser<I>,
    F: Fn(L::Output, R::Output) -> O,
{
    type Output = O;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        self.left.parse(tokenizer).flat_map(|left| {
            self.right
                .parse(tokenizer)
                .map(|right| (self.combiner)(left, right))
        })
    }
}
