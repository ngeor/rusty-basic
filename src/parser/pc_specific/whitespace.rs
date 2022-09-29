use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;

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

impl<P> OptParser for LeadingWhitespace<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_space = OptParser::parse(&whitespace(), tokenizer)?;
        if self.needs_whitespace && opt_space.is_none() {
            Ok(None)
        } else {
            match self.parser.parse(tokenizer)? {
                Some(value) => Ok(Some(value)),
                None => {
                    if let Some(space) = opt_space {
                        tokenizer.unread(space);
                    }
                    Ok(None)
                }
            }
        }
    }
}

impl<P> NonOptParser for LeadingWhitespace<P>
where
    P: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let opt_space = OptParser::parse(&whitespace(), tokenizer)?;
        if self.needs_whitespace && opt_space.is_none() {
            Err(QError::syntax_error("Expected: whitespace"))
        } else {
            self.parser.parse(tokenizer)
        }
    }
}

pub struct TrailingWhitespace<P>
where
    P: ParserBase,
{
    parser: P,
    needs_whitespace: bool,
}

impl<P> TrailingWhitespace<P>
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

impl<P> ParserBase for TrailingWhitespace<P>
where
    P: ParserBase,
{
    type Output = P::Output;
}

impl<P> OptParser for TrailingWhitespace<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(value) => {
                if self.needs_whitespace {
                    match OptParser::parse(&whitespace(), tokenizer)? {
                        Some(_) => Ok(Some(value)),
                        None => Err(QError::syntax_error("Expected: whitespace")),
                    }
                } else {
                    OptParser::parse(&whitespace(), tokenizer)?;
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }
}

impl<P> NonOptParser for TrailingWhitespace<P>
where
    P: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let result = self.parser.parse(tokenizer)?;
        if self.needs_whitespace {
            NonOptParser::parse(&whitespace(), tokenizer)?;
        } else {
            OptParser::parse(&whitespace(), tokenizer)?;
        }
        Ok(result)
    }
}

// TODO refactor so that LeadingWhitespace becomes a smaller type that depends on this one
pub struct LeadingWhitespacePreserving<P>(P)
where
    P: ParserBase;

impl<P> ParserBase for LeadingWhitespacePreserving<P>
where
    P: ParserBase,
{
    type Output = (Option<Token>, P::Output);
}

impl<P> OptParser for LeadingWhitespacePreserving<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_ws = OptParser::parse(&whitespace(), tokenizer)?;
        match self.0.parse(tokenizer)? {
            Some(value) => Ok(Some((opt_ws, value))),
            None => {
                opt_ws.undo(tokenizer);
                Ok(None)
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

impl<P> OptParser for SurroundedByWhitespacePreserving<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.leading_parser.parse(tokenizer)? {
            Some((opt_lead_ws, value)) => {
                let opt_trail_ws = OptParser::parse(&whitespace(), tokenizer)?;
                Ok(Some((opt_lead_ws, value, opt_trail_ws)))
            }
            None => Ok(None),
        }
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

impl<P> OptParser for SurroundedByWhitespace<P>
where
    P: OptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(_, m, _)| m))
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

    fn preceded_by_req_ws(self) -> LeadingWhitespace<Self> {
        self.preceded_by_ws(true)
    }

    fn followed_by_ws(self, mandatory: bool) -> TrailingWhitespace<Self>;

    fn followed_by_opt_ws(self) -> TrailingWhitespace<Self> {
        self.followed_by_ws(false)
    }

    fn followed_by_req_ws(self) -> TrailingWhitespace<Self> {
        self.followed_by_ws(true)
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

    fn followed_by_ws(self, mandatory: bool) -> TrailingWhitespace<Self> {
        TrailingWhitespace {
            parser: self,
            needs_whitespace: mandatory,
        }
    }

    fn preceded_by_ws_preserving(self) -> LeadingWhitespacePreserving<Self> {
        LeadingWhitespacePreserving(self)
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
