use super::{AtLocation, HasLocation, Locatable, Location};

//
// result.or_try_read(f)
//

pub trait ChainTryRead: Sized {
    fn or_try_read<F>(self, op: F) -> Self
    where
        F: FnMut() -> Self;
}

impl<T, E> ChainTryRead for Result<Option<T>, E> {
    fn or_try_read<F>(self, mut op: F) -> Self
    where
        F: FnMut() -> Self,
    {
        match &self {
            Ok(opt) => match opt {
                Some(_) => self,
                None => op(),
            },
            Err(_) => self,
        }
    }
}

//
// result.or_read(f)
//

pub trait TerminalTryRead<T>: Sized {
    fn or_read<F>(self, op: F) -> T
    where
        F: FnMut() -> T;
}

impl<T, E> TerminalTryRead<Result<T, E>> for Result<Option<T>, E> {
    fn or_read<F>(self, mut op: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
    {
        match self {
            Ok(opt) => match opt {
                Some(x) => Ok(x),
                None => op(),
            },
            Err(e) => Err(e),
        }
    }
}

//
// result.with_ok_pos(pos)
//

pub trait ToLocatableOk<TLocation, TResult> {
    fn with_ok_pos(self, p: TLocation) -> TResult;
}

impl<TLocation: HasLocation, T, E> ToLocatableOk<&TLocation, Result<Locatable<T>, E>>
    for Result<T, E>
{
    fn with_ok_pos(self, p: &TLocation) -> Result<Locatable<T>, E> {
        self.map(|e| e.at(p.pos()))
    }
}

impl<T, E> ToLocatableOk<Location, Result<Locatable<T>, E>> for Result<T, E> {
    fn with_ok_pos(self, pos: Location) -> Result<Locatable<T>, E> {
        self.map(|e| e.at(pos))
    }
}

impl<T, E> ToLocatableOk<Location, Option<Result<Locatable<T>, E>>> for Option<Result<T, E>> {
    fn with_ok_pos(self, pos: Location) -> Option<Result<Locatable<T>, E>> {
        self.map(|r| r.map(|x| x.at(pos)))
    }
}

impl<T, E> ToLocatableOk<Option<Location>, Option<Result<Locatable<T>, E>>>
    for Option<Result<T, E>>
{
    fn with_ok_pos(self, opt_pos: Option<Location>) -> Option<Result<Locatable<T>, E>> {
        match opt_pos {
            Some(pos) => self.with_ok_pos(pos),
            None => None,
        }
    }
}

/// Chains on Ok(Some) values

pub trait ChainResultOption<T, U, E> {
    fn and_then_opt<F>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<Option<U>, E>;
}

impl<T, U, E> ChainResultOption<T, U, E> for Result<Option<T>, E> {
    fn and_then_opt<F>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<Option<U>, E>,
    {
        match self {
            Ok(None) => Ok(None),
            Err(err) => Err(err),
            Ok(Some(x)) => f(x),
        }
    }
}

/// Maps an Option Result
pub trait MapOptionResult<T, U, E> {
    /// Maps the value of a `Some(Ok(_))` through the given function.
    fn map_ok<F>(self, f: F) -> Option<Result<U, E>>
    where
        F: FnOnce(T) -> U;

    /// Applies the given function to the value of a `Some(Ok(_))` and
    /// returns the result.
    fn and_then_ok<F>(self, f: F) -> Option<Result<U, E>>
    where
        F: FnOnce(T) -> Option<Result<U, E>>;
}

impl<T, U, E> MapOptionResult<T, U, E> for Option<Result<T, E>> {
    /// Applies the given function to the value of a `Some(Ok(_))`
    fn map_ok<F>(self, f: F) -> Option<Result<U, E>>
    where
        F: FnOnce(T) -> U,
    {
        self.map(|r| r.map(|x| f(x)))
    }

    fn and_then_ok<F>(self, f: F) -> Option<Result<U, E>>
    where
        F: FnOnce(T) -> Option<Result<U, E>>,
    {
        match self {
            Some(Ok(x)) => f(x),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

/// Zips two Option Result together
pub trait ZipOptionResult<T, X, U, E> {
    /// Zips two `Option<Result<>>` together.
    ///
    /// If either is None, None will be returned.
    /// If either is an Err, the Err will be returned.
    fn zip(self, other: X) -> Option<Result<(T, U), E>>;
}

/// Zips two `Option<Result<>>` together.
impl<T, U, E> ZipOptionResult<T, Option<Result<U, E>>, U, E> for Option<Result<T, E>> {
    fn zip(self, other: Option<Result<U, E>>) -> Option<Result<(T, U), E>> {
        match self {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(left)) => match other {
                None => None,
                Some(Err(err)) => Some(Err(err)),
                Some(Ok(right)) => Some(Ok((left, right))),
            },
        }
    }
}

/// Zips two `Option<Result<<>>` together. The second is lazily provided by a function.
impl<T, U, E, F> ZipOptionResult<T, F, U, E> for Option<Result<T, E>>
where
    F: FnOnce() -> Option<Result<U, E>>,
{
    fn zip(self, other: F) -> Option<Result<(T, U), E>> {
        match self {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(left)) => match other() {
                None => None,
                Some(Err(err)) => Some(Err(err)),
                Some(Ok(right)) => Some(Ok((left, right))),
            },
        }
    }
}
