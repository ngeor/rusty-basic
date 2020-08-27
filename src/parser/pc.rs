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
    pub fn read<R: Reader + 'static>() -> impl Fn(R) -> (R, Result<R::Item, R::Err>) {
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

    /// Applies the given mapping function to the successful result of the given source.
    ///
    /// This is similar to `Result.and_then`
    ///
    /// Note that if the mapping function returns Not Found, no undo will take place.
    pub fn and_then<R, S, T, E, F, U>(source: S, map: F) -> Box<dyn Fn(R) -> (R, Result<U, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        F: Fn(T) -> Result<U, E> + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            // the following is equivalent to result = result.and_then(map),
            // but rust does not like the nested closures
            let result = match result {
                Ok(x) => map(x),
                Err(err) => Err(err),
            };
            (reader, result)
        })
    }

    /// Applies the given mapping function to the successful result of the given source.
    ///
    /// This is similar to `Result.map`
    pub fn map<R, S, T, E, F, U>(source: S, map: F) -> Box<dyn Fn(R) -> (R, Result<U, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        F: Fn(T) -> U + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            // the following is equivalent to result = result.map(map),
            // but rust does not like the nested closures
            let result = match result {
                Ok(x) => Ok(map(x)),
                Err(err) => Err(err),
            };
            (reader, result)
        })
    }

    /// Combines the results of the two given sources into one tuple.
    ///
    /// If either source returns an error, the error will be returned.
    /// If the first source returns an error, the second will not be called.
    /// If the second source returns a Not Found error, the first result will be undone.
    pub fn and<R, F1, F2, T1, T2, E>(
        first: F1,
        second: F2,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2), E>)>
    where
        R: Reader + Undo<T1> + 'static,
        F1: Fn(R) -> (R, Result<T1, E>) + 'static,
        F2: Fn(R) -> (R, Result<T2, E>) + 'static,
        T1: 'static,
        T2: 'static,
        E: IsNotFoundErr,
    {
        Box::new(move |reader| {
            let (reader, res1) = first(reader);
            match res1 {
                Ok(r1) => {
                    let (reader, res2) = second(reader);
                    match res2 {
                        Ok(r2) => (reader, Ok((r1, r2))),
                        Err(err) => {
                            if err.is_not_found_err() {
                                (reader.undo(r1), Err(err))
                            } else {
                                (reader, Err(err))
                            }
                        }
                    }
                }
                Err(err) => (reader, Err(err)),
            }
        })
    }

    /// Drops the left part of a tuple result.
    pub fn drop_left<R, S, T1, T2, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T2, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<(T1, T2), E>) + 'static,
    {
        map(source, |(_, r)| r)
    }

    /// Drops the right part of a tuple result.
    pub fn drop_right<R, S, T1, T2, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T1, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<(T1, T2), E>) + 'static,
    {
        map(source, |(l, _)| l)
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

    pub fn read_any_if<R, T, F>(predicate: F) -> Box<dyn Fn(R) -> (R, Result<R::Item, R::Err>)>
    where
        R: Reader<Item = T> + Undo<T> + 'static,
        T: Copy,
        R::Err: NotFoundErr,
        F: Fn(T) -> bool + 'static,
    {
        filter_any(common::read(), predicate)
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

// ========================================================
// dealing with characters and strings
// ========================================================

pub mod str {
    use super::common::and_then;
    use super::traits::*;

    /// Reads characters into a string as long as they satisfy the predicate.
    ///
    /// This function will return an empty string if no characters match.
    pub fn take_zero_or_more<R, E, F>(predicate: F) -> Box<dyn Fn(R) -> (R, Result<String, E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: IsNotFoundErr + 'static,
        F: Fn(char) -> bool + 'static,
    {
        Box::new(move |char_reader| {
            let mut result: String = String::new();
            let mut cr: R = char_reader;
            loop {
                let (x, next) = cr.read();
                cr = x;
                match next {
                    Err(err) => {
                        if err.is_not_found_err() {
                            break;
                        } else {
                            return (cr, Err(err));
                        }
                    }
                    Ok(ch) => {
                        if predicate(ch) {
                            result.push(ch);
                        } else {
                            cr = cr.undo_item(ch);
                            break;
                        }
                    }
                }
            }
            (cr, Ok(result))
        })
    }

    /// Reads characters into a string as long as they satisfy the predicate.
    ///
    /// This function will return a Not Found result if no characters match.
    pub fn take_one_or_more<R, E, F>(predicate: F) -> Box<dyn Fn(R) -> (R, Result<String, E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: NotFoundErr + 'static,
        F: Fn(char) -> bool + 'static,
    {
        and_then(take_zero_or_more(predicate), |s| {
            if s.is_empty() {
                Err(E::not_found_err())
            } else {
                Ok(s)
            }
        })
    }
}

// ========================================================
// Dealing with whitespace
// ========================================================

pub mod ws {
    use super::common::{and, drop_left};
    use super::str::*;
    use super::traits::*;

    pub fn is_whitespace(ch: char) -> bool {
        ch == ' ' || ch == '\t'
    }

    /// Reads any whitespace.
    ///
    /// If no whitespace is found, it results to a Not Found result.
    pub fn read_any<R>() -> Box<dyn Fn(R) -> (R, Result<String, R::Err>)>
    where
        R: Reader<Item = char> + 'static,
        R::Err: NotFoundErr,
    {
        take_one_or_more(is_whitespace)
    }

    /// Reads some whitespace before the source and then returns the result of the source.
    ///
    /// If no whitespace exists before the source, the source will not be invoked and
    /// a Not Found result will be returned.
    pub fn with_leading<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: 'static,
        E: NotFoundErr + 'static,
    {
        drop_left(and(read_any(), source))
    }
}
