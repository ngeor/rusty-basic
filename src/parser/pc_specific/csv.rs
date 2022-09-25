use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;

type CsvParser<P> = DelimitedPC<P, CommaSurroundedByOptWhitespace>;
// allow missing between delimiters
type CsvAllowMissingParser<P> = DelimitedAllowMissingPC<P, CommaSurroundedByOptWhitespace>;
type NonOptCsvParser<P> = NonOptDelimitedPC<P, CommaSurroundedByOptWhitespace>;

pub trait CsvTrait
where
    Self: Sized,
{
    /// Returns one or more items when used as a `Parser`,
    /// or zero or more when used as a `NonOptParser`.
    fn csv(self) -> CsvParser<Self>;

    fn csv_allow_missing(self) -> CsvAllowMissingParser<Self>;

    fn csv_demand(self) -> NonOptCsvParser<Self>;
}

impl<S> CsvTrait for S {
    fn csv(self) -> CsvParser<Self> {
        self.one_or_more_delimited_by(
            comma_surrounded_by_opt_ws(),
            QError::syntax_error("Trailing comma"),
        )
    }

    fn csv_allow_missing(self) -> CsvAllowMissingParser<Self> {
        self.one_or_more_delimited_by_allow_missing(comma_surrounded_by_opt_ws())
    }

    fn csv_demand(self) -> NonOptCsvParser<Self> {
        self.one_or_more_delimited_by_non_opt(comma_surrounded_by_opt_ws())
    }
}

pub fn comma_surrounded_by_opt_ws() -> CommaSurroundedByOptWhitespace {
    CommaSurroundedByOptWhitespace
}

pub struct CommaSurroundedByOptWhitespace;

impl HasOutput for CommaSurroundedByOptWhitespace {
    type Output = (Option<Token>, Token, Option<Token>);
}

impl Parser for CommaSurroundedByOptWhitespace {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut opt_leading_space: Option<Token> = None;
        let mut comma_token: Option<Token> = None;
        while let Some(token) = tokenizer.read()? {
            if token.kind == TokenType::Whitespace as i32 {
                opt_leading_space = Some(token);
            } else if token.kind == TokenType::Comma as i32 {
                comma_token = Some(token);
                break;
            } else {
                tokenizer.unread(token);
                break;
            }
        }
        if comma_token.is_some() {
            let opt_trailing_space = if let Some(token) = tokenizer.read()? {
                if token.kind == TokenType::Whitespace as i32 {
                    Some(token)
                } else {
                    tokenizer.unread(token);
                    None
                }
            } else {
                None
            };
            Ok(Some((
                opt_leading_space,
                comma_token.unwrap(),
                opt_trailing_space,
            )))
        } else {
            opt_leading_space.undo(tokenizer);
            Ok(None)
        }
    }
}

impl NonOptParser for CommaSurroundedByOptWhitespace {
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.parse(tokenizer)?
            .ok_or(QError::syntax_error("Expected: ,"))
    }
}
