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

    /// Creates a parsing function which will get a result by creating a different
    /// function at runtime. The function is provided by the given factory.
    /// This can be used to solve recursive structures that cause stack overflow.
    pub fn lazy<R, S, T, E>(lazy_source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn() -> Box<dyn Fn(R) -> (R, Result<T, E>)> + 'static,
    {
        Box::new(move |reader| lazy_source()(reader))
    }

    pub fn map_fully<R, S, T, E, U, F1, F2, F3>(
        source: S,
        f_ok: F1,
        f_not_found: F2,
        f_err: F3,
    ) -> Box<dyn Fn(R) -> (R, U)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: IsNotFoundErr + 'static,
        F1: Fn(R, T) -> (R, U) + 'static,
        F2: Fn(E) -> U + 'static,
        F3: Fn(E) -> U + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => f_ok(reader, ch),
                Err(err) => {
                    if err.is_not_found_err() {
                        (reader, f_not_found(err))
                    } else {
                        (reader, f_err(err))
                    }
                }
            }
        })
    }

    pub fn map_fully_ok_or_not_found<R, S, T, E, U, F1, F2>(
        source: S,
        f_ok: F1,
        f_not_found: F2,
    ) -> Box<dyn Fn(R) -> (R, Result<U, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: IsNotFoundErr + 'static,
        F1: Fn(R, T) -> (R, Result<U, E>) + 'static,
        F2: Fn(E) -> Result<U, E> + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => f_ok(reader, ch),
                Err(err) => {
                    if err.is_not_found_err() {
                        (reader, f_not_found(err))
                    } else {
                        (reader, Err(err))
                    }
                }
            }
        })
    }

    pub fn map_fully_ok<R, S, T, E, U, F>(source: S, f_ok: F) -> Box<dyn Fn(R) -> (R, Result<U, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        F: Fn(R, T) -> (R, Result<U, E>) + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => f_ok(reader, ch),
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
        map_fully_ok(source, move |reader, x| (reader, map(x)))
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
        and_then(source, move |x| Ok(map(x)))
    }

    pub fn map_fully_err<R, S, T, E, F>(source: S, f_err: F) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        F: Fn(R, E) -> (R, Result<T, E>) + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => (reader, Ok(ch)),
                Err(err) => f_err(reader, err),
            }
        })
    }

    pub fn or_else<R, S, T, E, F>(source: S, map: F) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        F: Fn(E) -> Result<T, E> + 'static,
    {
        map_fully_err(source, move |reader, err| (reader, map(err)))
    }

    pub fn map_fully_not_found_err<R, S, T, E, F>(
        source: S,
        f_err: F,
    ) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: IsNotFoundErr,
        F: Fn(R, E) -> (R, Result<T, E>) + 'static,
    {
        Box::new(move |reader| {
            let (reader, result) = source(reader);
            match result {
                Ok(ch) => (reader, Ok(ch)),
                Err(err) => {
                    if err.is_not_found_err() {
                        f_err(reader, err)
                    } else {
                        (reader, Err(err))
                    }
                }
            }
        })
    }

    pub fn or_else_if_not_found<R, S, T, E, F>(
        source: S,
        map: F,
    ) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: IsNotFoundErr + 'static,
        F: Fn(E) -> Result<T, E> + 'static,
    {
        map_fully_not_found_err(source, move |reader, err| (reader, map(err)))
    }

    /// Returns a function that ensures that we don't get a Not Found result from
    /// the given source.
    ///
    /// Not found results are converted to the error provided from the given function.
    pub fn demand<R, S, T, E, FE>(source: S, err_fn: FE) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        E: IsNotFoundErr + 'static,
        FE: Fn() -> E + 'static,
    {
        or_else_if_not_found(source, move |_| Err(err_fn()))
    }

    /// Map the Ok result of the given source to Not Found, if it is equal to the default value
    /// for that type (e.g. empty string, empty vector).
    pub fn map_default_to_not_found<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: Default + PartialEq<T> + 'static,
        E: NotFoundErr + 'static,
    {
        and_then(source, |x| {
            if x == T::default() {
                Err(E::not_found_err())
            } else {
                Ok(x)
            }
        })
    }

    /// Combines the results of the two given sources into one tuple.
    ///
    /// If either source returns a fatal error, the error will be returned.
    /// If the second source returns a Not Found error, the first result will be still returned.
    pub fn if_first_maybe_second<R, F1, F2, T1, T2, E>(
        first: F1,
        second: F2,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, Option<T2>), E>)>
    where
        R: Reader + 'static,
        F1: Fn(R) -> (R, Result<T1, E>) + 'static,
        F2: Fn(R) -> (R, Result<T2, E>) + 'static,
        T1: 'static,
        T2: 'static,
        E: IsNotFoundErr + 'static,
    {
        map_fully_ok(first, move |reader, r1| {
            let (reader, res2) = second(reader);
            match res2 {
                Ok(r2) => (reader, Ok((r1, Some(r2)))),
                Err(err) => {
                    if err.is_not_found_err() {
                        (reader, Ok((r1, None)))
                    } else {
                        (reader, Err(err))
                    }
                }
            }
        })
    }

    pub fn seq2<R, S1, S2, T1, T2, E>(
        first: S1,
        second: S2,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2), E>)>
    where
        R: Reader + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        T1: 'static,
        T2: 'static,
        E: IsNotFoundErr + 'static,
    {
        and_then(
            if_first_maybe_second(first, second),
            move |(r1, opt_r2)| match opt_r2 {
                Some(r2) => Ok((r1, r2)),
                None => panic!("`seq2` second function returned None, wrap it in a `demand`"),
            },
        )
    }

    pub fn seq3<R, S1, S2, S3, T1, T2, T3, E>(
        first: S1,
        second: S2,
        third: S3,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3), E>)>
    where
        R: Reader + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        E: IsNotFoundErr + 'static,
    {
        map(seq2(first, seq2(second, third)), |(a, (b, c))| (a, b, c))
    }

    pub fn seq4<R, S1, S2, S3, S4, T1, T2, T3, T4, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3, T4), E>)>
    where
        R: Reader + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        S4: Fn(R) -> (R, Result<T4, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        E: IsNotFoundErr + 'static,
    {
        map(
            seq2(first, seq3(second, third, fourth)),
            |(a, (b, c, d))| (a, b, c, d),
        )
    }

    pub fn seq5<R, S1, S2, S3, S4, S5, T1, T2, T3, T4, T5, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        fifth: S5,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3, T4, T5), E>)>
    where
        R: Reader + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        S4: Fn(R) -> (R, Result<T4, E>) + 'static,
        S5: Fn(R) -> (R, Result<T5, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        E: IsNotFoundErr + 'static,
    {
        map(
            seq2(first, seq4(second, third, fourth, fifth)),
            |(a, (b, c, d, e))| (a, b, c, d, e),
        )
    }

    pub fn seq6<R, S1, S2, S3, S4, S5, S6, T1, T2, T3, T4, T5, T6, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        fifth: S5,
        sixth: S6,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3, T4, T5, T6), E>)>
    where
        R: Reader + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        S4: Fn(R) -> (R, Result<T4, E>) + 'static,
        S5: Fn(R) -> (R, Result<T5, E>) + 'static,
        S6: Fn(R) -> (R, Result<T6, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        T6: 'static,
        E: IsNotFoundErr + 'static,
    {
        map(
            seq2(first, seq5(second, third, fourth, fifth, sixth)),
            |(a, (b, c, d, e, f))| (a, b, c, d, e, f),
        )
    }

    pub fn seq7<R, S1, S2, S3, S4, S5, S6, S7, T1, T2, T3, T4, T5, T6, T7, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        fifth: S5,
        sixth: S6,
        seventh: S7,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3, T4, T5, T6, T7), E>)>
    where
        R: Reader + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        S4: Fn(R) -> (R, Result<T4, E>) + 'static,
        S5: Fn(R) -> (R, Result<T5, E>) + 'static,
        S6: Fn(R) -> (R, Result<T6, E>) + 'static,
        S7: Fn(R) -> (R, Result<T7, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        T6: 'static,
        T7: 'static,
        E: IsNotFoundErr + 'static,
    {
        map(
            seq2(first, seq6(second, third, fourth, fifth, sixth, seventh)),
            |(a, (b, c, d, e, f, g))| (a, b, c, d, e, f, g),
        )
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
        E: NotFoundErr + 'static,
    {
        map_fully_ok(
            if_first_maybe_second(first, second),
            move |reader, (r1, opt_r2)| match opt_r2 {
                Some(r2) => (reader, Ok((r1, r2))),
                None => (reader.undo(r1), Err(E::not_found_err())),
            },
        )
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
        map_fully_ok(source, move |reader, ch| {
            if predicate(&ch) {
                (reader, Ok(ch))
            } else {
                (reader.undo(ch), Err(E::not_found_err()))
            }
        })
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
        common::map_fully_ok(source, move |reader, ch| {
            if predicate(ch) {
                (reader, Ok(ch))
            } else {
                (reader.undo(ch), Err(E::not_found_err()))
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

    /// Reads characters into a string as long as they satisfy the predicates.
    ///
    /// The first character must satisfy the `leading_predicate` and the remaining
    /// characters must satisfy the `remaining_predicate`.
    ///
    /// This function will return an empty string if no characters match.
    pub fn zero_or_more_if_leading_remaining<R, E, F1, F2>(
        leading_predicate: F1,
        remaining_predicate: F2,
    ) -> Box<dyn Fn(R) -> (R, Result<String, E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: IsNotFoundErr + 'static,
        F1: Fn(char) -> bool + 'static,
        F2: Fn(char) -> bool + 'static,
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
                        if (result.is_empty() && leading_predicate(ch))
                            || (!result.is_empty() && remaining_predicate(ch))
                        {
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
    /// This function will return an empty string if no characters match.
    pub fn zero_or_more_if<R, E, F>(predicate: F) -> Box<dyn Fn(R) -> (R, Result<String, E>)>
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
    pub fn one_or_more_if<R, E, F>(predicate: F) -> Box<dyn Fn(R) -> (R, Result<String, E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: NotFoundErr + 'static,
        F: Fn(char) -> bool + 'static,
    {
        and_then(zero_or_more_if(predicate), |s| {
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
    use super::common;
    use super::str::*;
    use super::traits::*;

    pub fn is_whitespace(ch: char) -> bool {
        ch == ' ' || ch == '\t'
    }

    pub fn is_eol(ch: char) -> bool {
        ch == '\r' || ch == '\n'
    }

    pub fn is_eol_or_whitespace(ch: char) -> bool {
        is_eol(ch) || is_whitespace(ch)
    }

    /// Reads any whitespace.
    ///
    /// If no whitespace is found, it results to a Not Found result.
    pub fn one_or_more<R>() -> Box<dyn Fn(R) -> (R, Result<String, R::Err>)>
    where
        R: Reader<Item = char> + 'static,
        R::Err: NotFoundErr,
    {
        one_or_more_if(is_whitespace)
    }

    /// Reads some whitespace before the source and then returns the result of the source.
    ///
    /// If no whitespace exists before the source, the source will not be invoked and
    /// a Not Found result will be returned.
    pub fn one_or_more_leading<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: 'static,
        E: NotFoundErr + 'static,
    {
        common::drop_left(common::and(one_or_more(), source))
    }

    /// Reads any whitespace.
    ///
    /// If no whitespace is found, it results to an Ok empty string.
    pub fn zero_or_more<R>() -> Box<dyn Fn(R) -> (R, Result<String, R::Err>)>
    where
        R: Reader<Item = char> + 'static,
        R::Err: NotFoundErr,
    {
        zero_or_more_if(is_whitespace)
    }

    /// Skips any whitespace before the source and returns the result of the source.
    pub fn zero_or_more_leading<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: 'static,
        E: NotFoundErr + 'static,
    {
        common::drop_left(common::and(zero_or_more(), source))
    }

    /// Skips any whitespace after the source and returns the result of the source.
    pub fn zero_or_more_trailing<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: 'static,
        E: NotFoundErr + 'static,
    {
        common::drop_right(common::if_first_maybe_second(source, zero_or_more()))
    }

    /// Skips any whitespace around the source and returns the source's result.
    pub fn zero_or_more_around<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> (R, Result<T, E>)>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> (R, Result<T, E>) + 'static,
        T: 'static,
        E: NotFoundErr + 'static,
    {
        zero_or_more_trailing(zero_or_more_leading(source))
    }

    pub fn seq2<R, S1, S2, T1, T2, E, FE>(
        first: S1,
        second: S2,
        err_fn_expected_whitespace: FE,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2), E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        T1: 'static,
        T2: 'static,
        E: NotFoundErr + 'static,
        FE: Fn() -> E + 'static,
    {
        common::map(
            common::seq3(
                first,
                common::demand(one_or_more(), err_fn_expected_whitespace),
                second,
            ),
            |(l, _, r)| (l, r),
        )
    }

    pub fn seq3<R, S1, S2, S3, T1, T2, T3, E, FE>(
        first: S1,
        second: S2,
        third: S3,
        err_fn_fn_expected_whitespace: FE,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3), E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        E: NotFoundErr + 'static,
        FE: Fn() -> Box<dyn Fn() -> E> + 'static,
    {
        common::map(
            common::seq5(
                first,
                common::demand(one_or_more(), err_fn_fn_expected_whitespace()),
                second,
                common::demand(one_or_more(), err_fn_fn_expected_whitespace()),
                third,
            ),
            |(a, _, b, _, c)| (a, b, c),
        )
    }

    pub fn seq4<R, S1, S2, S3, S4, T1, T2, T3, T4, E, FE>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        err_fn_fn_expected_whitespace: FE,
    ) -> Box<dyn Fn(R) -> (R, Result<(T1, T2, T3, T4), E>)>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S1: Fn(R) -> (R, Result<T1, E>) + 'static,
        S2: Fn(R) -> (R, Result<T2, E>) + 'static,
        S3: Fn(R) -> (R, Result<T3, E>) + 'static,
        S4: Fn(R) -> (R, Result<T4, E>) + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        E: NotFoundErr + 'static,
        FE: Fn() -> Box<dyn Fn() -> E> + 'static,
    {
        common::map(
            common::seq7(
                first,
                common::demand(one_or_more(), err_fn_fn_expected_whitespace()),
                second,
                common::demand(one_or_more(), err_fn_fn_expected_whitespace()),
                third,
                common::demand(one_or_more(), err_fn_fn_expected_whitespace()),
                fourth,
            ),
            |(a, _, b, _, c, _, d)| (a, b, c, d),
        )
    }
}
