use crate::common::*;

//
// Result and error of this module
//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinterError {
    ArgumentCountMismatch,
    ArgumentTypeMismatch,
    TypeMismatch,
    NextWithoutFor,
    DuplicateDefinition,
    InvalidAssignment,
    InvalidConstant,
    SubprogramNotDefined,
    LabelNotFound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error(LinterError, Option<Location>);

impl Error {
    pub fn with_pos(self, pos: Location) -> Self {
        match self.1 {
            Some(x) => Self(self.0, Some(x)),
            None => Self(self.0, Some(pos)),
        }
    }

    #[cfg(test)]
    pub fn consume(self) -> (LinterError, Option<Location>) {
        (self.0, self.1)
    }
}

// To be able to use the ? operator on Result<Locatable<LinterError>>
impl From<Locatable<LinterError>> for Error {
    fn from(e: Locatable<LinterError>) -> Error {
        let (l, pos) = e.consume();
        Error(l, Some(pos))
    }
}

// To be able to use LinterError::x.into()
impl From<LinterError> for Error {
    fn from(l: LinterError) -> Error {
        Error(l, None)
    }
}

pub fn err<T>(l: LinterError, pos: Location) -> Result<T, Error> {
    Err(Error(l, Some(pos)))
}

pub fn err_l<T, U: HasLocation>(l: LinterError, locatable: &U) -> Result<T, Error> {
    err(l, locatable.location())
}

pub fn err_no_pos<T>(l: LinterError) -> Result<T, Error> {
    Err(Error(l, None))
}

pub trait WithErrPos<U> {
    fn with_err_pos(self, pos: Location) -> U;
}

impl<T> WithErrPos<Result<T, Error>> for Result<T, Error> {
    fn with_err_pos(self, pos: Location) -> Self {
        self.map_err(|x| x.with_pos(pos))
    }
}

pub trait WithPos<U> {
    fn with_pos(self, pos: Location) -> U;
}

impl<T: std::fmt::Debug + Sized> WithPos<Result<Locatable<T>, Error>> for Result<T, Error> {
    fn with_pos(self, pos: Location) -> Result<Locatable<T>, Error> {
        self.map(|x| x.at(pos))
    }
}
