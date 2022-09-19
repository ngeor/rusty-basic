use crate::common::QError;
use crate::parser::base::parsers::{
    HasOutput, NonOptParser, Parser, TokenPredicate, TokenPredicateParser,
};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::base::undo_pc::Undo;
use crate::parser::specific::{TokenKindParser, TokenType};

#[deprecated]
pub fn whitespace() -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser(TokenType::Whitespace).parser()
}

pub struct LeadingWhitespace<P>
where
    P: HasOutput,
{
    parser: P,
    needs_whitespace: bool,
}

impl<P> LeadingWhitespace<P>
where
    P: HasOutput,
{
    pub fn new(parser: P, needs_whitespace: bool) -> Self {
        Self {
            parser,
            needs_whitespace,
        }
    }
}

impl<P> HasOutput for LeadingWhitespace<P>
where
    P: HasOutput,
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

impl<P> NonOptParser for LeadingWhitespace<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let opt_space = whitespace().parse(tokenizer)?;
        if self.needs_whitespace && opt_space.is_none() {
            Err(QError::syntax_error("Expected whitespace"))
        } else {
            self.parser.parse_non_opt(tokenizer)
        }
    }
}

pub struct TrailingWhitespace<P>
where
    P: HasOutput,
{
    parser: P,
    needs_whitespace: bool,
}

impl<P> TrailingWhitespace<P>
where
    P: HasOutput,
{
    pub fn new(parser: P, needs_whitespace: bool) -> Self {
        Self {
            parser,
            needs_whitespace,
        }
    }
}

impl<P> HasOutput for TrailingWhitespace<P>
where
    P: HasOutput,
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

impl<P> NonOptParser for TrailingWhitespace<P>
where P : NonOptParser {
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let result = self.parser.parse_non_opt(tokenizer)?;
        if self.needs_whitespace {
            whitespace().parse_non_opt(tokenizer)?;
        } else {
            whitespace().parse(tokenizer)?;
        }
        Ok(result)
    }
}

// TODO refactor so that LeadingWhitespace becomes a smaller type that depends on this one
pub struct LeadingWhitespacePreserving<P>(P)
where
    P: HasOutput;

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

pub struct SurroundedByWhitespacePreserving<P>
where
    P: HasOutput,
{
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

pub struct SurroundedByWhitespace<P>
where
    P: HasOutput,
{
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

pub trait WhitespaceTrait
where
    Self: Sized + HasOutput,
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
    P: Sized + HasOutput,
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
