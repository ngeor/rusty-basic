use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Tokenizer};
use crate::parser_declaration;

parser_declaration!(struct LoopWhile<predicate: F>);

impl<P, F> ParserBase for LoopWhile<P, F>
where
    P: ParserBase,
{
    type Output = Vec<P::Output>;
}

impl<P, F> NonOptParser for LoopWhile<P, F>
where
    P: OptParser,
    F: Fn(&P::Output) -> bool,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            match self.parser.parse(tokenizer)? {
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
        Ok(result)
    }
}

impl<P, F> OptParser for LoopWhile<P, F>
where
    P: OptParser,
    F: Fn(&P::Output) -> bool,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let result = NonOptParser::parse(self, tokenizer)?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

//
// non opt
//

parser_declaration!(struct LoopWhileNonOpt<predicate: F>);

impl<P, F> ParserBase for LoopWhileNonOpt<P, F>
where
    P: ParserBase,
{
    type Output = Vec<P::Output>;
}

impl<P, F> NonOptParser for LoopWhileNonOpt<P, F>
where
    P: NonOptParser,
    F: Fn(&P::Output) -> bool,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut result: Vec<P::Output> = vec![];
        let mut keep_going = true;
        while keep_going {
            let item = self.parser.parse(tokenizer)?;
            keep_going = (self.predicate)(&item);
            // push to the list regardless
            result.push(item);
        }
        Ok(result)
    }
}
