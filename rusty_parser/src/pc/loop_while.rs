use crate::pc::{default_parse_error, ParseResult, Parser};
use crate::{parser_declaration, ParseError};

parser_declaration!(pub struct LoopWhile<predicate: F>);

impl<I, P, F> Parser<I> for LoopWhile<P, F>
where
    P: Parser<I>,
    F: Fn(&P::Output) -> bool,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        let mut remaining = tokenizer;
        while keep_going {
            match self.parser.parse(remaining) {
                Ok((tokenizer, item)) => {
                    keep_going = (self.predicate)(&item);
                    // push to the list regardless
                    result.push(item);
                    remaining = tokenizer;
                }
                Err((false, i, _)) => {
                    remaining = i;
                    keep_going = false;
                }
                Err(err) => return Err(err),
            }
        }
        if result.is_empty() {
            default_parse_error(remaining)
        } else {
            Ok((remaining, result))
        }
    }
}
