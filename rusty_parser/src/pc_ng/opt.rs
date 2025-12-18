use crate::pc_ng::*;

pub struct Opt<P> {
    parser: P,
}

impl<P> Opt<P> {
    pub fn new(parser: P) -> Self {
        Opt { parser }
    }
}

impl<P> Parser for Opt<P>
where
    P: Parser,
{
    type Input = P::Input;
    type Output = Option<P::Output>;
    type Error = P::Error;

    fn parse(&self, input: Self::Input) -> ParseResult<Self::Input, Self::Output, Self::Error> {
        match self.parser.parse(input) {
            ParseResult::Ok(i, result) => ParseResult::Ok(i, Some(result)),
            ParseResult::None(i) | ParseResult::Expected(i, _) => ParseResult::Ok(i, None),
            ParseResult::Err(i, err) => ParseResult::Err(i, err),
        }
    }
}
