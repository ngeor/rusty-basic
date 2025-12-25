use rusty_common::{HasPos, Position};

/// ErrorEnvelope wraps an error with stacktrace information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorEnvelope<T>(T, Vec<Position>);

impl<T> ErrorEnvelope<T> {
    pub fn new(err: T, pos: Position) -> Self {
        Self(err, vec![pos])
    }

    pub fn new_with_stacktrace(err: T, stacktrace: Vec<Position>) -> Self {
        Self(err, stacktrace)
    }

    pub fn new_draining_stacktrace(err: T, stacktrace: &mut Vec<Position>) -> Self {
        debug_assert!(!stacktrace.is_empty());
        if stacktrace.len() == 1 {
            Self::new(err, stacktrace.pop().unwrap())
        } else {
            let new_stacktrace = stacktrace.drain(0..stacktrace.len()).collect();
            Self(err, new_stacktrace)
        }
    }

    pub fn err(&self) -> &T {
        &self.0
    }

    pub fn appen_draining_stacktrace(self, stacktrace: &mut Vec<Position>) -> Self {
        let Self(err, mut old_stacktrace) = self;
        old_stacktrace.append(stacktrace);
        Self(err, old_stacktrace)
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
        self.map_err(|e| ErrorEnvelope::new(e, p.pos()))
    }
}

pub trait WithStacktrace<O = Self> {
    /// Adds the given stacktrace to the error.
    /// The given stacktrace is emptied, to avoid adding the same items twice.
    fn with_stacktrace(self, stacktrace: &mut Vec<Position>) -> O;
}

impl<T> WithStacktrace for ErrorEnvelope<T> {
    fn with_stacktrace(self, stacktrace: &mut Vec<Position>) -> Self {
        self.appen_draining_stacktrace(stacktrace)
    }
}

impl<T, E> WithStacktrace<Result<T, ErrorEnvelope<E>>> for Result<T, E> {
    fn with_stacktrace(self, stacktrace: &mut Vec<Position>) -> Result<T, ErrorEnvelope<E>> {
        self.map_err(|err| ErrorEnvelope::new_draining_stacktrace(err, stacktrace))
    }
}
