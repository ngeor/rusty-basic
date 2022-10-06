use crate::parser::pc::{Token, TokenList, Tokenizer};

pub trait Undo {
    fn undo(self, tokenizer: &mut impl Tokenizer);
}

impl Undo for Token {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        tokenizer.unread(self);
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

impl Undo for TokenList {
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
