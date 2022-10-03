use crate::parser::pc::*;

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
