use super::{HasLocation, Location};

//
// ErrorEnvelope
//

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorEnvelope<T> {
    NoPos(T),
    Pos(T, Location),
    Stacktrace(T, Vec<Location>),
}

impl<T> ErrorEnvelope<T> {
    /// Returns the topmost error
    pub fn pos(&self) -> Option<Location> {
        match self {
            Self::NoPos(_) => None,
            Self::Pos(_, pos) => Some(*pos),
            Self::Stacktrace(_, s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(s[0])
                }
            }
        }
    }

    pub fn map<F, U>(self, f: F) -> ErrorEnvelope<U>
    where
        F: Fn(T) -> U,
    {
        match self {
            Self::NoPos(x) => ErrorEnvelope::NoPos(f(x)),
            Self::Pos(x, pos) => ErrorEnvelope::Pos(f(x), pos),
            Self::Stacktrace(x, s) => ErrorEnvelope::Stacktrace(f(x), s),
        }
    }

    pub fn into_err(self) -> T {
        match self {
            Self::NoPos(t) | Self::Pos(t, _) | Self::Stacktrace(t, _) => t,
        }
    }

    /// Patches the envelope with the given position.
    /// If the object already has a position or a stacktrace,
    /// it is returned as-is.
    pub fn patch_pos(self, pos: Location) -> Self {
        match self {
            Self::NoPos(t) => Self::Pos(t, pos),
            _ => self,
        }
    }

    pub fn patch_stacktrace(self, s: &Vec<Location>) -> Self {
        let mut v_old: Vec<Location> = match &self {
            Self::NoPos(_) => vec![],
            Self::Pos(_, p) => vec![*p],
            Self::Stacktrace(_, v) => v.clone(),
        };
        let mut v_new: Vec<Location> = s.clone();
        v_old.append(&mut v_new);
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

impl<T> AsRef<T> for ErrorEnvelope<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::NoPos(t) | Self::Pos(t, _) | Self::Stacktrace(t, _) => t,
        }
    }
}

//
// result.with_err_no_pos()
//

pub trait ToErrorEnvelopeNoPos<TResult> {
    fn with_err_no_pos(self) -> TResult;
}

impl<T, E> ToErrorEnvelopeNoPos<Result<T, ErrorEnvelope<E>>> for Result<T, E> {
    fn with_err_no_pos(self) -> Result<T, ErrorEnvelope<E>> {
        self.map_err(|e| ErrorEnvelope::NoPos(e))
    }
}

//
// result.with_err_at()
//

pub trait ToLocatableError<TLocation, TResult> {
    fn with_err_at(self, p: TLocation) -> TResult;
}

impl<TLocation: HasLocation, T, E> ToLocatableError<&TLocation, Result<T, ErrorEnvelope<E>>>
    for Result<T, E>
{
    fn with_err_at(self, p: &TLocation) -> Result<T, ErrorEnvelope<E>> {
        self.map_err(|e| ErrorEnvelope::Pos(e, p.pos()))
    }
}

impl<T, E> ToLocatableError<Location, Result<T, ErrorEnvelope<E>>> for Result<T, E> {
    fn with_err_at(self, pos: Location) -> Result<T, ErrorEnvelope<E>> {
        self.map_err(|e| ErrorEnvelope::Pos(e, pos))
    }
}

//
// result.patch_err_pos()
//

pub trait PatchErrPos<TLocation, TResult> {
    fn patch_err_pos(self, p: TLocation) -> TResult;
}

impl<TLocation: HasLocation, T, E> PatchErrPos<&TLocation, Result<T, ErrorEnvelope<E>>>
    for Result<T, ErrorEnvelope<E>>
{
    fn patch_err_pos(self, p: &TLocation) -> Self {
        self.map_err(|e| e.patch_pos(p.pos()))
    }
}

impl<T, E> PatchErrPos<Location, Result<T, ErrorEnvelope<E>>> for Result<T, ErrorEnvelope<E>> {
    fn patch_err_pos(self, pos: Location) -> Self {
        self.map_err(|e| e.patch_pos(pos))
    }
}

//
// shorthand functions
//

pub fn err_no_pos<T, E>(body: E) -> Result<T, ErrorEnvelope<E>> {
    Err(ErrorEnvelope::NoPos(body))
}
