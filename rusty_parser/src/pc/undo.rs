use crate::pc::{Token, TokenList, Tokenizer};

pub trait Undo {
    fn undo(self, tokenizer: &mut impl Tokenizer);
}

impl Undo for Token {
    fn undo(self, tokenizer: &mut impl Tokenizer) {
        tokenizer.unread();
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
        while let Some(_) = x.pop() {
            tokenizer.unread();
        }
    }
}
