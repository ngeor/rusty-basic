use rusty_common::{HasPos, Position};

//
// ErrorEnvelope
//

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorEnvelope<T> {
    NoPos(T),
    Pos(T, Position),
    Stacktrace(T, Vec<Position>),
}

impl<T> AsRef<T> for ErrorEnvelope<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::NoPos(t) | Self::Pos(t, _) | Self::Stacktrace(t, _) => t,
        }
    }
}

impl<T> ErrorEnvelope<T> {
    pub fn into_err(self) -> T {
        match self {
            Self::NoPos(t) | Self::Pos(t, _) | Self::Stacktrace(t, _) => t,
        }
    }

    /// Patches the envelope with the given position.
    /// If the object already has a position or a stacktrace,
    /// it is returned as-is.
    pub fn patch_pos(self, pos: Position) -> Self {
        match self {
            Self::NoPos(t) => Self::Pos(t, pos),
            _ => self,
        }
    }

    pub fn patch_stacktrace(self, v_new: &mut Vec<Position>) -> Self {
        let mut v_old: Vec<Position> = match &self {
            Self::NoPos(_) => vec![],
            Self::Pos(_, p) => vec![*p],
            Self::Stacktrace(_, v) => v.clone(),
        };
        v_old.append(v_new);
        let body = self.into_err();
        if v_old.is_empty() {
            Self::NoPos(body)
        } else if v_old.len() == 1 {
            Self::Pos(body, v_old.pop().unwrap())
        } else {
            Self::Stacktrace(body, v_old)
        }
    }
}

//
// result.with_err_no_pos()
//

pub trait WithErrNoPos<TResult> {
    fn with_err_no_pos(self) -> TResult;
}

impl<T, E> WithErrNoPos<Result<T, ErrorEnvelope<E>>> for Result<T, E> {
    fn with_err_no_pos(self) -> Result<T, ErrorEnvelope<E>> {
        self.map_err(|e| ErrorEnvelope::NoPos(e))
    }
}

//
// result.with_err_at()
//

pub trait WithErrAt<Pos, TResult> {
    fn with_err_at(self, p: Pos) -> TResult;
}

impl<Pos: HasPos, T, E> WithErrAt<&Pos, Result<T, ErrorEnvelope<E>>> for Result<T, E> {
    fn with_err_at(self, p: &Pos) -> Result<T, ErrorEnvelope<E>> {
        self.map_err(|e| ErrorEnvelope::Pos(e, p.pos()))
    }
}

//
// result.patch_err_pos()
//

pub trait PatchErrPos<Pos, TResult> {
    fn patch_err_pos(self, p: Pos) -> TResult;
}

impl<Pos: HasPos, T, E> PatchErrPos<&Pos, Self> for Result<T, ErrorEnvelope<E>> {
    fn patch_err_pos(self, p: &Pos) -> Self {
        self.map_err(|e| e.patch_pos(p.pos()))
    }
}
