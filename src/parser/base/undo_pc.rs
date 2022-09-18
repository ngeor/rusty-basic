use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, Parser};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use std::marker::PhantomData;

pub trait Undo {
    fn undo(self, tokenizer: &mut impl Tokenizer);
}

impl Undo for Option<Token> {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        if let Some(token) = self {
            tokenizer.unread(token);
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

pub struct UndoParser<P, F, O>(P, F, O);

impl<P, F, O> HasOutput for UndoParser<P, F, O>
where
    P: HasOutput<Output = O>,
    O: Undo,
{
    type Output = P::Output;
}

impl<P, F, O> Parser for UndoParser<P, F, O>
where
    P: Parser<Output = O>,
    O: Undo,
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

pub trait UndoTrait<F, O> {
    fn validate(self, f: F) -> UndoParser<Self, F, O>
    where
        Self: Sized + Parser<Output = O>,
        F: Fn(&O) -> Result<bool, QError>,
        O: Undo;
}

impl<P, F, O> UndoTrait<F, O> for P
where
    P: Parser<Output = O>,
    F: Fn(&O) -> Result<bool, QError>,
    O: Undo,
{
    fn validate(self, f: F) -> UndoParser<Self, F, O> {
        UndoParser(self, f, PhantomData)
    }
}
