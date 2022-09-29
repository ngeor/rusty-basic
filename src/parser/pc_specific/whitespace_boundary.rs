use crate::common::QError;
use crate::parser::pc::{NonOptParser, OptParser, ParserBase, Token, Tokenizer, Undo};
use crate::parser::pc_specific::TokenType;
use crate::parser::ExpressionNode;

pub struct WhitespaceBoundary(Option<Token>);

impl Undo for WhitespaceBoundary {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.0.undo(tokenizer);
    }
}

pub fn whitespace_boundary_after_expr(expr: &ExpressionNode) -> WhitespaceBoundaryParser {
    whitespace_boundary(expr.is_parenthesis())
}

pub fn whitespace_boundary(is_optional: bool) -> WhitespaceBoundaryParser {
    WhitespaceBoundaryParser { is_optional }
}

pub struct WhitespaceBoundaryParser {
    is_optional: bool,
}

impl WhitespaceBoundaryParser {
    fn none(&self) -> Option<WhitespaceBoundary> {
        if self.is_optional {
            Some(WhitespaceBoundary(None))
        } else {
            None
        }
    }
}

impl ParserBase for WhitespaceBoundaryParser {
    type Output = WhitespaceBoundary;
}

impl OptParser for WhitespaceBoundaryParser {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) if token.kind == TokenType::Whitespace as i32 => {
                Ok(Some(WhitespaceBoundary(Some(token))))
            }
            Some(token) => {
                tokenizer.unread(token);
                Ok(self.none())
            }
            None => Ok(self.none()),
        }
    }
}

impl NonOptParser for WhitespaceBoundaryParser {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) if token.kind == TokenType::Whitespace as i32 => {
                Ok(WhitespaceBoundary(Some(token)))
            }
            Some(token) => {
                tokenizer.unread(token);
                self.none()
                    .ok_or(QError::syntax_error("Expected: whitespace"))
            }
            None => self
                .none()
                .ok_or(QError::syntax_error("Expected: whitespace")),
        }
    }
}
