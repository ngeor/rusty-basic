use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser, TokenPredicate, TokenPredicateParser};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::base::undo_pc::Undo;
use crate::parser::specific::{TokenKindParser, TokenType};

#[deprecated]
pub fn whitespace() -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser(TokenType::Whitespace).parser()
}

pub struct LeadingWhitespace<P> {
    parser: P,
    needs_whitespace: bool,
}

impl<P> LeadingWhitespace<P> {
    pub fn new(parser: P, needs_whitespace: bool) -> Self {
        Self {
            parser,
            needs_whitespace,
        }
    }
}

impl<P> HasOutput for LeadingWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;
}

impl<P> Parser for LeadingWhitespace<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_space = whitespace().parse(tokenizer)?;
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

pub struct TrailingWhitespace<P> {
    parser: P,
    needs_whitespace: bool,
}

impl<P> TrailingWhitespace<P> {
    pub fn new(parser: P, needs_whitespace: bool) -> Self {
        Self {
            parser,
            needs_whitespace,
        }
    }
}

impl<P> HasOutput for TrailingWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;
}

impl<P> Parser for TrailingWhitespace<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.parser.parse(tokenizer)? {
            Some(value) => {
                if self.needs_whitespace {
                    match whitespace().parse(tokenizer)? {
                        Some(_) => Ok(Some(value)),
                        None => Err(QError::syntax_error("Expected whitespace")),
                    }
                } else {
                    whitespace().parse(tokenizer)?;
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }
}

// TODO refactor so that LeadingWhitespace becomes a smaller type that depends on this one
pub struct LeadingWhitespacePreserving<P>(P);

impl<P> HasOutput for LeadingWhitespacePreserving<P>
where
    P: HasOutput,
{
    type Output = (Option<Token>, P::Output);
}

impl<P> Parser for LeadingWhitespacePreserving<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_ws = whitespace().parse(tokenizer)?;
        match self.0.parse(tokenizer)? {
            Some(value) => Ok(Some((opt_ws, value))),
            None => {
                opt_ws.undo(tokenizer);
                Ok(None)
            }
        }
    }
}

pub struct SurroundedByWhitespacePreserving<P> {
    leading_parser: LeadingWhitespacePreserving<P>,
}

impl<P> HasOutput for SurroundedByWhitespacePreserving<P>
where
    P: HasOutput,
{
    type Output = (Option<Token>, P::Output, Option<Token>);
}

impl<P> Parser for SurroundedByWhitespacePreserving<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.leading_parser.parse(tokenizer)? {
            Some((opt_lead_ws, value)) => {
                let opt_trail_ws = whitespace().parse(tokenizer)?;
                Ok(Some((opt_lead_ws, value, opt_trail_ws)))
            }
            None => Ok(None),
        }
    }
}

pub struct SurroundedByWhitespace<P> {
    parser: SurroundedByWhitespacePreserving<P>,
}

impl<P> HasOutput for SurroundedByWhitespace<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<P> Parser for SurroundedByWhitespace<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.parser
            .parse(tokenizer)
            .map(|opt_result| opt_result.map(|(l, m, r)| m))
    }
}

pub trait WhitespaceTrait {
    fn preceded_by_ws(self, mandatory: bool) -> LeadingWhitespace<Self>
    where
        Self: Sized;

    fn preceded_by_opt_ws(self) -> LeadingWhitespace<Self>
    where
        Self: Sized,
    {
        self.preceded_by_ws(false)
    }

    fn preceded_by_req_ws(self) -> LeadingWhitespace<Self>
    where
        Self: Sized,
    {
        self.preceded_by_ws(true)
    }

    fn followed_by_ws(self, mandatory: bool) -> TrailingWhitespace<Self>
    where
        Self: Sized;

    fn followed_by_opt_ws(self) -> TrailingWhitespace<Self>
    where
        Self: Sized,
    {
        self.followed_by_ws(false)
    }

    fn followed_by_req_ws(self) -> TrailingWhitespace<Self>
    where
        Self: Sized,
    {
        self.followed_by_ws(true)
    }

    fn preceded_by_ws_preserving(self) -> LeadingWhitespacePreserving<Self>
    where
        Self: Sized;

    fn surrounded_by_ws_preserving(self) -> SurroundedByWhitespacePreserving<Self>
    where
        Self: Sized;

    fn surrounded_by_opt_ws(self) -> SurroundedByWhitespace<Self>
    where
        Self: Sized;
}

impl<P> WhitespaceTrait for P {
    fn preceded_by_ws(self, mandatory: bool) -> LeadingWhitespace<Self>
    where
        Self: Sized,
    {
        LeadingWhitespace {
            parser: self,
            needs_whitespace: mandatory,
        }
    }

    fn followed_by_ws(self, mandatory: bool) -> TrailingWhitespace<Self>
    where
        Self: Sized,
    {
        TrailingWhitespace {
            parser: self,
            needs_whitespace: mandatory,
        }
    }

    fn preceded_by_ws_preserving(self) -> LeadingWhitespacePreserving<Self>
    where
        Self: Sized,
    {
        LeadingWhitespacePreserving(self)
    }

    fn surrounded_by_ws_preserving(self) -> SurroundedByWhitespacePreserving<Self>
    where
        Self: Sized,
    {
        SurroundedByWhitespacePreserving {
            leading_parser: self.preceded_by_ws_preserving(),
        }
    }

    fn surrounded_by_opt_ws(self) -> SurroundedByWhitespace<Self>
    where
        Self: Sized,
    {
        SurroundedByWhitespace {
            parser: self.surrounded_by_ws_preserving(),
        }
    }
}
