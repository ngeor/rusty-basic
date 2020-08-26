// ========================================================
// traits
// ========================================================

pub mod traits {
    pub trait IsNotFoundErr {
        fn is_not_found_err(&self) -> bool;
    }

    pub trait NotFoundErr: IsNotFoundErr {
        fn not_found_err() -> Self;
    }

    pub trait Undo<T> {
        fn undo(self, item: T) -> Self;
    }

    pub trait Reader: Sized {
        type Item;
        type Err;
        fn read(self) -> (Self, Result<Self::Item, Self::Err>);
        fn undo_item(self, item: Self::Item) -> Self;

        fn undo_and_err_not_found<T, R>(self, item: T) -> (Self, Result<R, Self::Err>)
        where
            Self: Undo<T>,
            Self::Err: NotFoundErr,
        {
            (self.undo(item), Err(Self::Err::not_found_err()))
        }
    }
}

// ========================================================
// simple parsing functions
// ========================================================

pub mod common {
    use super::traits::*;

    /// Returns a function that gets the next item from a reader.
    pub fn read_any<R: Reader + 'static>() -> impl Fn(R) -> (R, Result<R::Item, R::Err>) {
        |reader| reader.read()
    }

    // ========================================================
    // simple parsing combinators
    // ========================================================

    /// Returns a function that ensures that we don't get a Not Found result from
    /// the given source.
    ///
    /// Not found results are converted to the error provided from the given function.
    pub fn demand<R, S, T, E, FE>(source: S, err_fn: FE) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Err = E> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: IsNotFoundErr + 'static,
        FE: Fn() -> E + 'static,
    {
        Box::new(move |reader| {
            let (reader, next) = source(reader);
            match next {
                Ok(x) => (reader, Ok(x)),
                Err(err) => {
                    if err.is_not_found_err() {
                        (reader, Err(err_fn()))
                    } else {
                        (reader, Err(err))
                    }
                }
            }
        })
    }

    /// Returns a function that filters the given source with the given predicate.
    /// If the predicate returns `true`, the value of the source is returned as-is.
    /// Otherwise, a Not Found error will be returned.
    pub fn filter_any<R, S, T, E, F>(source: S, predicate: F) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Err = E> + Undo<T> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: NotFoundErr + 'static,
        F: Fn(&T) -> bool + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => {
                    if predicate(&ch) {
                        (reader, Ok(ch))
                    } else {
                        (reader.undo(ch), Err(E::not_found_err()))
                    }
                }
                Err(err) => (reader, Err(err)),
            }
        })
    }

    pub fn filter_some<R, S, T, E, F, FE>(
        source: S,
        predicate: F,
        err_fn: FE,
    ) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Err = E> + Undo<T> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: 'static,
        E: NotFoundErr + 'static,
        F: Fn(&T) -> bool + 'static,
        FE: Fn() -> E + 'static,
    {
        demand(filter_any(source, predicate), err_fn)
    }
}

// ========================================================
// when Item : Copy
// ========================================================

pub mod copy {
    use super::common;
    use super::traits::*;

    /// Returns a function that filters the given source with the given predicate.
    /// If the predicate returns `true`, the value of the source is returned as-is.
    /// Otherwise, a Not Found error will be returned.
    pub fn filter_any<R, S, T, E, F>(source: S, predicate: F) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Err = E> + Undo<T> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: Copy,
        E: NotFoundErr,
        F: Fn(T) -> bool + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => {
                    if predicate(ch) {
                        (reader, Ok(ch))
                    } else {
                        (reader.undo(ch), Err(E::not_found_err()))
                    }
                }
                Err(err) => (reader, Err(err)),
            }
        })
    }

    pub fn filter_some<R, S, T, E, F, FE>(
        source: S,
        predicate: F,
        err_fn: FE,
    ) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Err = E> + Undo<T> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: Copy + 'static,
        E: NotFoundErr + 'static,
        F: Fn(T) -> bool + 'static,
        FE: Fn() -> E + 'static,
    {
        common::demand(filter_any(source, predicate), err_fn)
    }

    pub fn read_any_if<R, T, F>(predicate: F) -> Box<dyn Fn(R) -> (R, Result<R::Item, R::Err>)>
    where
        R: Reader<Item = T> + Undo<T> + 'static,
        T: Copy,
        R::Err: NotFoundErr,
        F: Fn(T) -> bool + 'static,
    {
        filter_any(common::read_any(), predicate)
    }

    pub fn try_read<R, T>(needle: T) -> Box<dyn Fn(R) -> (R, Result<R::Item, R::Err>)>
    where
        R: Reader<Item = T> + Undo<T> + 'static,
        T: Copy + PartialEq + 'static,
        R::Err: NotFoundErr,
    {
        read_any_if(move |ch| ch == needle)
    }

    /// Undoes the read item if it was successful but still returns it.
    #[deprecated]
    pub fn undo_if_ok<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + Undo<T> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: Copy + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => (reader.undo(ch), Ok(ch)),
                Err(err) => (reader, Err(err)),
            }
        })
    }
}

// ========================================================
// when Item = char
// ========================================================

// ========================================================
// when Reader + HasLocation
// ========================================================

pub mod loc {
    use super::traits::*;
    use crate::common::{AtLocation, HasLocation, Locatable};

    /// Creates a function that maps the result of the source into a locatable result,
    /// using the position of the reader just before invoking the source.
    pub fn with_pos<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<Locatable<T>, E>)>
    where
        R: Reader<Err = E> + HasLocation + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
    {
        Box::new(move |reader| {
            // capture pos before invoking source
            let pos = reader.pos();
            let (reader, result) = source(reader);
            let loc_result = result.map(|x| x.at(pos));
            (reader, loc_result)
        })
    }
}

// ========================================================
// Converting error to error at a position
// ========================================================

pub mod err {
    use super::traits::*;

    use crate::common::{ErrorEnvelope, HasLocation};

    pub fn with_err_at<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, ErrorEnvelope<E>>)>
    where
        R: Reader<Err = E> + HasLocation + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(x) => (reader, Ok(x)),
                Err(err) => {
                    // capture pos after invoking source
                    let pos = reader.pos();
                    (reader, Err(ErrorEnvelope::Pos(err, pos)))
                }
            }
        })
    }
}
