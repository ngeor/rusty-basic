use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;

/// Comma separated list of items.
/// When used as a parser, returns one or more items.
/// When used as a non-opt-parser, returns zero or more items.
pub fn csv<L: OptParser>(
    parser: L,
) -> impl OptParser<Output = Vec<L::Output>> + NonOptParser<Output = Vec<L::Output>> {
    delimited_by(parser, comma_surrounded_by_opt_ws(), trailing_comma_error())
}

/// Comma separated list of items.
/// Returns at least one item.
pub fn csv_non_opt<L: NonOptParser>(parser: L) -> impl NonOptParser<Output = Vec<L::Output>> {
    delimited_by_non_opt(parser, comma_surrounded_by_opt_ws(), trailing_comma_error())
}

pub fn trailing_comma_error() -> QError {
    QError::syntax_error("Error: trailing comma")
}

pub fn comma_surrounded_by_opt_ws() -> CommaSurroundedByOptWhitespace {
    CommaSurroundedByOptWhitespace
}

pub struct CommaSurroundedByOptWhitespace;

impl ParserBase for CommaSurroundedByOptWhitespace {
    type Output = (Option<Token>, Token, Option<Token>);
}

impl OptParser for CommaSurroundedByOptWhitespace {
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
