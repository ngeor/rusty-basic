use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::{Token, Tokenizer};

pub trait Undo {
    fn undo(self, tokenizer: &mut impl Tokenizer);
}

impl Undo for Token {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        tokenizer.unread(self);
    }
}

impl Undo for (Option<Token>, Token, Option<Token>) {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.2.undo(tokenizer);
        self.1.undo(tokenizer);
        self.0.undo(tokenizer);
    }
}

impl<T> Undo for Option<T>
where
    T: Undo,
{
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        if let Some(token) = self {
            token.undo(tokenizer);
        }
    }
}

impl Undo for Vec<Token> {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        let mut x = self;
        loop {
            match x.pop() {
                Some(token) => {
                    tokenizer.unread(token);
                }
                None => {
                    break;
                }
            }
        }
    }
}

impl<A, B> Undo for (A, B)
where
    A: Undo,
    B: Undo,
{
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        self.1.undo(tokenizer);
        self.0.undo(tokenizer);
    }
}

pub struct UndoParser<P, F>(P, F);

impl<P, F> HasOutput for UndoParser<P, F>
where
    P: HasOutput,
    P::Output: Undo,
{
    type Output = P::Output;
}

impl<P, F> Parser for UndoParser<P, F>
where
    P: Parser,
    P::Output: Undo,
    F: Fn(&P::Output) -> Result<bool, QError>,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => {
                let should_keep: bool = (self.1)(&value)?;
                if should_keep {
                    Ok(Some(value))
                } else {
                    value.undo(tokenizer);
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

pub trait UndoTrait<F> {
    fn validate(self, f: F) -> UndoParser<Self, F>
    where
        Self: Sized + Parser,
        F: Fn(&Self::Output) -> Result<bool, QError>,
        Self::Output: Undo;
}

impl<P, F> UndoTrait<F> for P
where
    P: Parser,
    P::Output: Undo,
    F: Fn(&P::Output) -> Result<bool, QError>,
{
    fn validate(self, f: F) -> UndoParser<Self, F> {
        UndoParser(self, f)
    }
}
