use crate::common::{ParserErrorTrait, QError};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser_declaration;

pub fn whitespace() -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser::new(TokenType::Whitespace).parser()
}

pub struct LeadingWhitespace<P>
where
    P: ParserBase,
{
    parser: P,
    needs_whitespace: bool,
}

impl<P> LeadingWhitespace<P>
where
    P: ParserBase,
{
    pub fn new(parser: P, needs_whitespace: bool) -> Self {
        Self {
            parser,
            needs_whitespace,
        }
    }
}

impl<P> ParserBase for LeadingWhitespace<P>
where
    P: ParserBase,
{
    type Output = P::Output;
}

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

// TODO refactor so that LeadingWhitespace becomes a smaller type that depends on this one
parser_declaration!(struct LeadingWhitespacePreserving);

impl<P> ParserBase for LeadingWhitespacePreserving<P>
where
    P: ParserBase,
{
    type Output = (Option<Token>, P::Output);
}

// TODO this is like OptAnd and LeadingWhitespace is identical to this only it drops the whitespace from the output

impl<P> Parser for LeadingWhitespacePreserving<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let opt_space = parse_opt_space(tokenizer, false)?;
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok((opt_space, value)),
            Err(err) => {
                if err.is_incomplete() {
                    opt_space.undo(tokenizer);
                }
                Err(err)
            }
        }
    }
}

pub struct SurroundedByWhitespacePreserving<P>
where
    P: ParserBase,
{
    leading_parser: LeadingWhitespacePreserving<P>,
}

impl<P> ParserBase for SurroundedByWhitespacePreserving<P>
where
    P: ParserBase,
{
    type Output = (Option<Token>, P::Output, Option<Token>);
}

impl<P> Parser for SurroundedByWhitespacePreserving<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let (opt_leading, value) = self.leading_parser.parse(tokenizer)?;
        let opt_trailing = parse_opt_space(tokenizer, false)?;
        Ok((opt_leading, value, opt_trailing))
    }
}

pub struct SurroundedByWhitespace<P>
where
    P: ParserBase,
{
    parser: SurroundedByWhitespacePreserving<P>,
}

impl<P> ParserBase for SurroundedByWhitespace<P>
where
    P: ParserBase,
{
    type Output = P::Output;
}

impl<P> Parser for SurroundedByWhitespace<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parser.parse(tokenizer).map(|(_, m, _)| m)
    }
}

// TODO delete the `preceded_by_req_ws and preceded_by_opt_ws` traits
pub trait WhitespaceTrait
where
    Self: Sized + ParserBase,
{
    fn preceded_by_ws(self, mandatory: bool) -> LeadingWhitespace<Self>;

    fn preceded_by_opt_ws(self) -> LeadingWhitespace<Self> {
        self.preceded_by_ws(false)
    }

    // TODO #[deprecated]
    fn preceded_by_req_ws(self) -> LeadingWhitespace<Self> {
        self.preceded_by_ws(true)
    }

    fn preceded_by_ws_preserving(self) -> LeadingWhitespacePreserving<Self>;

    fn surrounded_by_ws_preserving(self) -> SurroundedByWhitespacePreserving<Self>;

    fn surrounded_by_opt_ws(self) -> SurroundedByWhitespace<Self>;
}

impl<P> WhitespaceTrait for P
where
    P: Sized + ParserBase,
{
    fn preceded_by_ws(self, mandatory: bool) -> LeadingWhitespace<Self> {
        LeadingWhitespace {
            parser: self,
            needs_whitespace: mandatory,
        }
    }

    fn preceded_by_ws_preserving(self) -> LeadingWhitespacePreserving<Self> {
        LeadingWhitespacePreserving::new(self)
    }

    fn surrounded_by_ws_preserving(self) -> SurroundedByWhitespacePreserving<Self> {
        SurroundedByWhitespacePreserving {
            leading_parser: self.preceded_by_ws_preserving(),
        }
    }

    fn surrounded_by_opt_ws(self) -> SurroundedByWhitespace<Self> {
        SurroundedByWhitespace {
            parser: self.surrounded_by_ws_preserving(),
        }
    }
}

// TODO get rid of the preceding ws trait
