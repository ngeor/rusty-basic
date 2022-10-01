use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser_declaration;

parser_declaration!(
    struct LeadingWhitespace {
        needs_whitespace: bool,
    }
);

fn parse_opt_space(
    tokenizer: &mut impl Tokenizer,
    needs_whitespace: bool,
) -> Result<Option<Token>, QError> {
    match whitespace().parse(tokenizer) {
        Ok(t) => Ok(Some(t)),
        Err(err) if err.is_incomplete() => {
            if needs_whitespace {
                Err(err)
            } else {
                Ok(None)
            }
        }
        Err(err) => Err(err),
    }
}

// TODO: this looks like OptAnd

impl<P> Parser for LeadingWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let opt_space = parse_opt_space(tokenizer, self.needs_whitespace)?;
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err(err) => {
                if err.is_incomplete() {
                    opt_space.undo(tokenizer);
                }
                Err(err)
            }
        }
    }
}

#[deprecated]
pub trait WhitespaceTrait
where
    Self: Sized + Parser,
{
    #[deprecated]
    fn preceded_by_ws(self, mandatory: bool) -> LeadingWhitespace<Self>;

    #[deprecated]
    fn preceded_by_opt_ws(self) -> LeadingWhitespace<Self> {
        self.preceded_by_ws(false)
    }

    #[deprecated]
    fn preceded_by_req_ws(self) -> LeadingWhitespace<Self> {
        self.preceded_by_ws(true)
    }
}

impl<P> WhitespaceTrait for P
where
    P: Sized + Parser,
{
    fn preceded_by_ws(self, mandatory: bool) -> LeadingWhitespace<Self> {
        LeadingWhitespace {
            parser: self,
            needs_whitespace: mandatory,
        }
    }
}
