use crate::common::QError;
use crate::parser::pc::{Parser, Tokenizer};
use crate::parser_declaration;

parser_declaration!(
    struct LoopWhile<predicate: F> {
        // TODO remove this
        allow_empty: bool,
    }
);

impl<P, F> Parser for LoopWhile<P, F>
where
    P: Parser,
    F: Fn(&P::Output) -> bool,
{
    type Output = Vec<P::Output>;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            match self.parser.parse_opt(tokenizer)? {
                Some(item) => {
                    keep_going = (self.predicate)(&item);
                    // push to the list regardless
                    result.push(item);
                }
                None => {
                    keep_going = false;
                }
            }
        }
        if result.is_empty() && !self.allow_empty {
            Err(QError::Incomplete)
        } else {
            Ok(result)
        }
    }
}
