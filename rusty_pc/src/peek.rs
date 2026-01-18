use crate::{InputTrait, Parser};

pub struct PeekParser<P> {
    parser: P,
}

impl<P> PeekParser<P> {
    pub fn new(parser: P) -> Self {
        Self { parser }
    }
}

impl<I, C, P> Parser<I, C> for PeekParser<P>
where
    I: InputTrait,
    I: InputTrait,
    P: Parser<I, C>,
{
    type Output = P::Output;
    type Error = P::Error;

    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error> {
        let original_position = input.get_position();
        match self.parser.parse(input) {
            Ok(value) => {
                input.set_position(original_position);
                Ok(value)
            }
            Err(err) => Err(err),
        }
    }
}
