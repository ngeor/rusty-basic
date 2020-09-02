// ========================================================
// types
// ========================================================

// the R is needed in the error in order to be able to get error location

pub type ReaderResult<R, T, E> = Result<(R, Option<T>), (R, E)>;

// ========================================================
// traits
// ========================================================

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

pub trait Reader: Sized {
    type Item;
    type Err;
    fn read(self) -> ReaderResult<Self, Self::Item, Self::Err>;
    fn undo_item(self, item: Self::Item) -> Self;
}

// ========================================================
// Parsers that apply a function to a source
// ========================================================

pub mod map {
    use super::*;

    /// Gets the result of the source and maps it with the given function.
    /// The mapping function has access to the reader and is called for Ok(Some) and Ok(None) values.
    pub fn source_map<R, S, T, E, U, F>(source: S, f: F) -> Box<dyn Fn(R) -> ReaderResult<R, U, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
        F: Fn(R, Option<T>) -> (R, Option<U>) + 'static,
    {
        Box::new(move |reader| source(reader).map(|(r, opt_res)| f(r, opt_res)))
    }

    /// Gets the result of the source and then switches it to the result of the given function.
    /// The mapping function has access to the reader and is called for Ok(Some) and Ok(None) values.
    pub fn source_and_then<R, S, T, E, U, F>(
        source: S,
        f: F,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, U, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        F: Fn(R, Option<T>) -> ReaderResult<R, U, E> + 'static,
    {
        Box::new(move |reader| source(reader).and_then(|(r, opt_res)| f(r, opt_res)))
    }

    /// Gets the result of the source and if it had some value it switches it to the result of the given function.
    pub fn source_and_then_some<R, S, T, E, U, F>(
        source: S,
        f_ok: F,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, U, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        F: Fn(R, T) -> ReaderResult<R, U, E> + 'static,
    {
        source_and_then(source, move |reader, opt_res| match opt_res {
            Some(ch) => f_ok(reader, ch),
            None => Ok((reader, None)),
        })
    }

    pub fn and_then_none<R, S, T, E, F>(source: S, f: F) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        F: Fn() -> Result<T, E> + 'static,
    {
        source_and_then(source, move |reader, opt_res| match opt_res {
            Some(ch) => Ok((reader, Some(ch))),
            None => match f() {
                Ok(ch) => Ok((reader, Some(ch))),
                Err(e) => Err((reader, e)),
            },
        })
    }

    /// Applies the given mapping function to the successful result of the given source.
    /// The mapping function does not have access to the reader and can return Ok or Err.
    pub fn and_then<R, S, T, E, F, U>(source: S, map: F) -> Box<dyn Fn(R) -> ReaderResult<R, U, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        F: Fn(T) -> Result<U, E> + 'static,
    {
        source_and_then_some(source, move |reader, x| match map(x) {
            Ok(y) => Ok((reader, Some(y))),
            Err(e) => Err((reader, e)),
        })
    }

    /// Applies the given mapping function to the successful result of the given source.
    /// The mapping function does not have access to the reader and can return Some or None.
    pub fn opt_map<R, S, T, E, F, U>(source: S, map: F) -> Box<dyn Fn(R) -> ReaderResult<R, U, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        F: Fn(T) -> Option<U> + 'static,
    {
        source_and_then_some(source, move |reader, x| Ok((reader, map(x))))
    }

    /// Applies the given mapping function to the successful result of the given source.
    pub fn map<R, S, T, E, F, U>(source: S, map: F) -> Box<dyn Fn(R) -> ReaderResult<R, U, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        F: Fn(T) -> U + 'static,
    {
        opt_map(source, move |x| Some(map(x)))
    }
}

// ========================================================
// Combine two or more sources
// ========================================================

pub mod combine {
    use super::map::*;
    use super::*;

    /// Combines the two given sources, letting the second use the value returned by the first one.
    /// The second source is only used if the first result was `Ok(Some)`.
    /// Errors from any source have priority.
    pub fn combine_some<R, S1, PS2, T1, T2, E>(
        first: S1,
        second: PS2,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, Option<T2>), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        PS2: Fn(&T1) -> Box<dyn Fn(R) -> ReaderResult<R, T2, E>> + 'static,
    {
        source_and_then_some(first, move |reader, r1| {
            second(&r1)(reader).and_then(|(reader, opt_r2)| Ok((reader, Some((r1, opt_r2)))))
        })
    }
}

// ========================================================
// simple parsing functions
// ========================================================

pub mod common {
    use super::map::*;
    use super::*;

    /// Returns a function that gets the next item from a reader.
    pub fn read<R: Reader + 'static>() -> impl Fn(R) -> ReaderResult<R, R::Item, R::Err> {
        |reader| reader.read()
    }

    // ========================================================
    // simple parsing combinators
    // ========================================================

    /// Creates a parsing function which will get a result by creating a different
    /// function at runtime. The function is provided by the given factory.
    /// This can be used to solve recursive structures that cause stack overflow.
    pub fn lazy<R, LS, T, E>(lazy_source: LS) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader + 'static,
        LS: Fn() -> Box<dyn Fn(R) -> ReaderResult<R, T, E>> + 'static,
    {
        Box::new(move |reader| lazy_source()(reader))
    }

    /// Returns a function that ensures that we don't get a Not Found result from
    /// the given source.
    ///
    /// Not found results are converted to the error provided from the given function.
    pub fn demand<R, S, T, E, FE>(source: S, err_fn: FE) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        E: 'static,
        FE: Fn() -> E + 'static,
    {
        and_then_none(source, move || Err(err_fn()))
    }

    /// Map the Ok result of the given source to Not Found, if it is equal to the default value
    /// for that type (e.g. empty string, empty vector).
    pub fn map_default_to_not_found<R, S, T, E>(
        source: S,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: Default + PartialEq<T> + 'static,
    {
        opt_map(source, |x| if x == T::default() { None } else { Some(x) })
    }

    /// Combines the results of the two given sources into one tuple.
    ///
    /// If either source returns a fatal error, the error will be returned.
    /// If the second source returns a Not Found error, the first result will be still returned.
    pub fn opt_seq2<R, S1, S2, T1, T2, E>(
        first: S1,
        second: S2,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, Option<T2>), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        T1: 'static,
        T2: 'static,
        E: 'static,
    {
        source_and_then_some(first, move |reader, r1| {
            second(reader).and_then(|(reader, opt_r2)| Ok((reader, Some((r1, opt_r2)))))
        })
    }

    pub fn opt_seq3<R, S1, S2, S3, T1, T2, T3, E>(
        first: S1,
        second: S2,
        third: S3,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, Option<T2>, Option<T3>), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        E: 'static,
    {
        map(opt_seq2(opt_seq2(first, second), third), |((a, b), c)| {
            (a, b, c)
        })
    }

    pub fn seq2<R, S1, S2, T1, T2, E>(
        first: S1,
        second: S2,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        T1: 'static,
        T2: 'static,
        E: 'static,
    {
        opt_map(opt_seq2(first, second), move |(r1, opt_r2)| match opt_r2 {
            Some(r2) => Some((r1, r2)),
            None => panic!("`seq2` second function returned None, wrap it in a `demand`"),
        })
    }

    pub fn seq3<R, S1, S2, S3, T1, T2, T3, E>(
        first: S1,
        second: S2,
        third: S3,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        E: 'static,
    {
        map(seq2(first, seq2(second, third)), |(a, (b, c))| (a, b, c))
    }

    pub fn seq4<R, S1, S2, S3, S4, T1, T2, T3, T4, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3, T4), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        S4: Fn(R) -> ReaderResult<R, T4, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        E: 'static,
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
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3, T4, T5), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        S4: Fn(R) -> ReaderResult<R, T4, E> + 'static,
        S5: Fn(R) -> ReaderResult<R, T5, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        E: 'static,
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
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3, T4, T5, T6), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        S4: Fn(R) -> ReaderResult<R, T4, E> + 'static,
        S5: Fn(R) -> ReaderResult<R, T5, E> + 'static,
        S6: Fn(R) -> ReaderResult<R, T6, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        T6: 'static,
        E: 'static,
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
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3, T4, T5, T6, T7), E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        S4: Fn(R) -> ReaderResult<R, T4, E> + 'static,
        S5: Fn(R) -> ReaderResult<R, T5, E> + 'static,
        S6: Fn(R) -> ReaderResult<R, T6, E> + 'static,
        S7: Fn(R) -> ReaderResult<R, T7, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        T6: 'static,
        T7: 'static,
        E: 'static,
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
    pub fn and<R, S1, S2, T1, T2, E>(
        first: S1,
        second: S2,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2), E>>
    where
        R: Reader + Undo<T1> + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        T1: 'static,
        T2: 'static,
        E: 'static,
    {
        source_and_then_some(
            opt_seq2(first, second),
            move |reader, (r1, opt_r2)| match opt_r2 {
                Some(r2) => Ok((reader, Some((r1, r2)))),
                None => Ok((reader.undo(r1), None)),
            },
        )
    }

    /// Drops the left part of a tuple result.
    pub fn drop_left<R, S, T1, T2, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T2, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, (T1, T2), E> + 'static,
    {
        map(source, |(_, r)| r)
    }

    /// Drops the right part of a tuple result.
    pub fn drop_right<R, S, T1, T2, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T1, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, (T1, T2), E> + 'static,
    {
        map(source, |(l, _)| l)
    }

    /// Returns a function that filters the given source with the given predicate.
    /// If the predicate returns `true`, the value of the source is returned as-is.
    /// Otherwise, a Not Found error will be returned.
    pub fn filter<R, S, T, E, F>(source: S, predicate: F) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader<Err = E> + Undo<T> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        E: 'static,
        F: Fn(&T) -> bool + 'static,
    {
        source_and_then_some(source, move |reader, ch| {
            if predicate(&ch) {
                Ok((reader, Some(ch)))
            } else {
                Ok((reader.undo(ch), None))
            }
        })
    }

    /// Reverses the result of the given source. If the source returns a successful
    /// result, it returns a Not Found result. If the source returns a Not Found
    /// result, it returns an Ok result.
    pub fn negate<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, (), E>>
    where
        R: Reader + Undo<T> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
    {
        source_map(source, |reader, res| match res {
            Some(x) => (reader.undo(x), None),
            None => (reader, Some(())),
        })
    }

    pub fn or<R, S1, S2, T, E>(first: S1, second: S2) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader + 'static,
        S1: Fn(R) -> ReaderResult<R, T, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T, E> + 'static,
        E: 'static,
    {
        source_and_then(first, move |reader, opt_res1| match opt_res1 {
            Some(ch) => Ok((reader, Some(ch))),
            None => second(reader),
        })
    }

    pub fn or_vec<R, T, E, F>(mut sources: Vec<F>) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader + 'static,
        T: 'static,
        E: 'static,
        F: Fn(R) -> ReaderResult<R, T, E> + 'static,
    {
        if sources.len() > 2 {
            let first = sources.remove(0);
            or(first, or_vec(sources))
        } else if sources.len() == 2 {
            let second = sources.pop().unwrap();
            let first = sources.pop().unwrap();
            or(first, second)
        } else {
            panic!("or_vec must have at least two functions to choose from");
        }
    }

    pub fn zero_or_more<R, S, T1, T2, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, Vec<T1>, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, (T1, Option<T2>), E> + 'static,
        E: 'static,
    {
        Box::new(move |char_reader| {
            let mut result: Vec<T1> = vec![];
            let mut cr: R = char_reader;
            loop {
                match source(cr) {
                    Ok((next_cr, opt_res)) => {
                        cr = next_cr;
                        match opt_res {
                            Some((t1, opt_t2)) => {
                                let last = opt_t2.is_none();
                                result.push(t1);
                                if last {
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            Ok((cr, Some(result)))
        })
    }
}

// ========================================================
// when Item : Copy
// ========================================================

pub mod copy {
    use super::common;
    use super::map::source_and_then_some;
    use super::*;

    /// Returns a function that filters the given source with the given predicate.
    /// If the predicate returns `true`, the value of the source is returned as-is.
    /// Otherwise, a Not Found error will be returned.
    pub fn filter<R, S, T, E, F>(source: S, predicate: F) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader<Err = E> + Undo<T> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: Copy,
        F: Fn(T) -> bool + 'static,
    {
        source_and_then_some(source, move |reader, ch| {
            if predicate(ch) {
                Ok((reader, Some(ch)))
            } else {
                Ok((reader.undo(ch), None))
            }
        })
    }

    pub fn read_if<R, T, F>(predicate: F) -> Box<dyn Fn(R) -> ReaderResult<R, R::Item, R::Err>>
    where
        R: Reader<Item = T> + Undo<T> + 'static,
        T: Copy,
        F: Fn(T) -> bool + 'static,
    {
        filter(common::read(), predicate)
    }

    pub fn try_read<R, T>(needle: T) -> Box<dyn Fn(R) -> ReaderResult<R, R::Item, R::Err>>
    where
        R: Reader<Item = T> + Undo<T> + 'static,
        T: Copy + PartialEq + 'static,
    {
        read_if(move |ch| ch == needle)
    }

    pub fn peek<R, T>(needle: T) -> Box<dyn Fn(R) -> ReaderResult<R, R::Item, R::Err>>
    where
        R: Reader<Item = T> + Undo<T> + 'static,
        T: Copy + PartialEq + 'static,
    {
        source_and_then_some(try_read(needle), |reader: R, c| {
            Ok((reader.undo(c), Some(c)))
        })
    }
}

// ========================================================
// when Reader + HasLocation
// ========================================================

pub mod loc {
    use super::*;
    use crate::common::{AtLocation, HasLocation, Locatable};

    /// Creates a function that maps the result of the source into a locatable result,
    /// using the position of the reader just before invoking the source.
    pub fn with_pos<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, Locatable<T>, E>>
    where
        R: Reader<Err = E> + HasLocation + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
    {
        Box::new(move |reader| {
            // capture pos before invoking source
            let pos = reader.pos();
            source(reader).and_then(|(reader, ok_res)| Ok((reader, ok_res.map(|x| x.at(pos)))))
        })
    }
}

// ========================================================
// Converting error to error at a position
// ========================================================

pub mod err {
    use super::*;

    use crate::common::{ErrorEnvelope, HasLocation};

    pub fn with_err_at<R, S, T, E>(
        source: S,
    ) -> Box<dyn Fn(R) -> Result<(R, Option<T>), ErrorEnvelope<E>>>
    where
        R: Reader<Err = E> + HasLocation + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
    {
        Box::new(move |reader| {
            let result = source(reader);
            match result {
                Ok(x) => Ok(x),
                Err((reader, err)) => {
                    // capture pos after invoking source
                    let pos = reader.pos();
                    Err(ErrorEnvelope::Pos(err, pos))
                }
            }
        })
    }
}

// ========================================================
// dealing with characters and strings
// ========================================================

pub mod str {
    use super::common;
    use super::map::{map, source_and_then_some};
    use super::*;
    use std::str::FromStr;

    /// Reads characters into a string as long as they satisfy the predicates.
    ///
    /// The first character must satisfy the `leading_predicate` and the remaining
    /// characters must satisfy the `remaining_predicate`.
    ///
    /// This function will return an empty string if no characters match.
    pub fn zero_or_more_if_leading_remaining<R, E, F1, F2>(
        leading_predicate: F1,
        remaining_predicate: F2,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        F1: Fn(char) -> bool + 'static,
        F2: Fn(char) -> bool + 'static,
    {
        Box::new(move |char_reader| {
            let mut result: String = String::new();
            let mut cr: R = char_reader;
            loop {
                match cr.read() {
                    Ok((x, opt_res)) => {
                        cr = x;
                        match opt_res {
                            Some(ch) => {
                                if (result.is_empty() && leading_predicate(ch))
                                    || (!result.is_empty() && remaining_predicate(ch))
                                {
                                    result.push(ch);
                                } else {
                                    cr = cr.undo_item(ch);
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            Ok((cr, Some(result)))
        })
    }

    /// Reads characters into a string as long as they satisfy the predicate.
    ///
    /// This function will return an empty string if no characters match.
    pub fn zero_or_more_if<R, E, F>(predicate: F) -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
        F: Fn(char) -> bool + 'static,
    {
        Box::new(move |char_reader| {
            let mut result: String = String::new();
            let mut cr: R = char_reader;
            loop {
                match cr.read() {
                    Ok((x, opt_res)) => {
                        cr = x;
                        match opt_res {
                            Some(ch) => {
                                if predicate(ch) {
                                    result.push(ch);
                                } else {
                                    cr = cr.undo_item(ch);
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            Ok((cr, Some(result)))
        })
    }

    /// Reads characters into a string as long as they satisfy the predicate.
    ///
    /// This function will return a Not Found result if no characters match.
    pub fn one_or_more_if<R, E, F>(predicate: F) -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
        F: Fn(char) -> bool + 'static,
    {
        common::map_default_to_not_found(zero_or_more_if(predicate))
    }

    pub fn switch_from_str<R, S, T, E>(
        source: S,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T, String), E>>
    where
        R: Reader + Undo<String> + 'static,
        S: Fn(R) -> ReaderResult<R, String, E> + 'static,
        T: FromStr + 'static,
    {
        source_and_then_some(source, |reader, s| match T::from_str(&s) {
            Ok(u) => Ok((reader, Some((u, s)))),
            Err(_) => Ok((reader.undo(s), None)),
        })
    }

    pub fn map_to_str<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: std::fmt::Display + 'static,
        E: 'static,
    {
        map(source, |x| x.to_string())
    }
}

// ========================================================
// Dealing with whitespace
// ========================================================

pub mod ws {
    use super::common;
    use super::map::map;
    use super::str::*;
    use super::*;

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
    pub fn one_or_more<R>() -> Box<dyn Fn(R) -> ReaderResult<R, String, R::Err>>
    where
        R: Reader<Item = char> + 'static,
    {
        common::map_default_to_not_found(zero_or_more())
    }

    /// Reads some whitespace before the source and then returns the result of the source.
    ///
    /// If no whitespace exists before the source, the source will not be invoked and
    /// a Not Found result will be returned.
    pub fn one_or_more_leading<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
    {
        common::drop_left(common::and(one_or_more(), source))
    }

    /// Reads any whitespace.
    ///
    /// If no whitespace is found, it results to an Ok empty string.
    pub fn zero_or_more<R>() -> Box<dyn Fn(R) -> ReaderResult<R, String, R::Err>>
    where
        R: Reader<Item = char> + 'static,
    {
        zero_or_more_if(is_whitespace)
    }

    /// Skips any whitespace before the source and returns the result of the source.
    pub fn zero_or_more_leading<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
    {
        common::drop_left(common::and(zero_or_more(), source))
    }

    /// Skips any whitespace after the source and returns the result of the source.
    pub fn zero_or_more_trailing<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
    {
        common::drop_right(common::opt_seq2(source, zero_or_more()))
    }

    /// Skips any whitespace around the source and returns the source's result.
    pub fn zero_or_more_around<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, E>>
    where
        R: Reader<Item = char, Err = E> + Undo<String> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
    {
        zero_or_more_trailing(zero_or_more_leading(source))
    }

    pub fn seq2<R, S1, S2, T1, T2, E, FE>(
        first: S1,
        second: S2,
        err_fn_expected_whitespace: FE,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2), E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        T1: 'static,
        T2: 'static,
        E: 'static,
        FE: Fn() -> E + 'static,
    {
        map(
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
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3), E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        E: 'static,
        FE: Fn() -> Box<dyn Fn() -> E> + 'static,
    {
        map(
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
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (T1, T2, T3, T4), E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S1: Fn(R) -> ReaderResult<R, T1, E> + 'static,
        S2: Fn(R) -> ReaderResult<R, T2, E> + 'static,
        S3: Fn(R) -> ReaderResult<R, T3, E> + 'static,
        S4: Fn(R) -> ReaderResult<R, T4, E> + 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        E: 'static,
        FE: Fn() -> Box<dyn Fn() -> E> + 'static,
    {
        map(
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

pub mod misc {
    use super::common::{
        and, demand, drop_left, map_default_to_not_found, opt_seq2, seq3, zero_or_more,
    };
    use super::copy::{read_if, try_read};
    use super::map::{map, source_and_then_some};
    use super::*;
    use crate::common::QError;
    use crate::parser::types::Keyword;
    use std::str::FromStr;

    fn is_letter(ch: char) -> bool {
        (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
    }

    fn is_non_leading_identifier(ch: char) -> bool {
        (ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || (ch == '.')
    }

    fn is_digit(ch: char) -> bool {
        ch >= '0' && ch <= '9'
    }

    fn is_symbol(ch: char) -> bool {
        (ch > ' ' && ch < '0')
            || (ch > '9' && ch < 'A')
            || (ch > 'Z' && ch < 'a')
            || (ch > 'z' && ch <= '~')
    }

    pub fn read_any_symbol<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, char, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
    {
        read_if(is_symbol)
    }

    pub fn read_any_letter<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, char, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
    {
        read_if(is_letter)
    }

    /// Reads any identifier. Note that the result might be a keyword.
    /// An identifier must start with a letter and consists of letters, numbers and the dot.
    pub fn read_any_identifier<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
    {
        map_default_to_not_found(super::str::zero_or_more_if_leading_remaining(
            is_letter,
            is_non_leading_identifier,
        ))
    }

    /// Reads any keyword.
    pub fn read_any_keyword<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
    {
        crate::parser::pc::str::switch_from_str(read_any_identifier())
    }

    /// Reads any word, i.e. any identifier which is not a keyword.
    pub fn read_any_word<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
    {
        source_and_then_some(
            read_any_identifier(),
            |reader: R, s| match Keyword::from_str(&s) {
                Ok(_) => Ok((reader.undo(s), None)),
                Err(_) => Ok((reader, Some(s))),
            },
        )
    }

    pub fn read_keyword_if<R, E, F>(
        predicate: F,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        F: Fn(Keyword) -> bool + 'static,
        E: 'static,
    {
        super::common::filter(read_any_keyword(), move |(k, _)| predicate(*k))
    }

    // TODO optimize
    pub fn try_read_keyword<R, E>(
        needle: Keyword,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
    {
        read_keyword_if(move |k| k == needle)
    }

    pub fn demand_keyword<R>(
        needle: Keyword,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), QError>>
    where
        R: Reader<Item = char, Err = QError> + 'static,
    {
        demand(
            try_read_keyword(needle),
            QError::syntax_error_fn(format!("Expected: {}", needle)),
        )
    }

    pub fn demand_guarded_keyword<R>(
        needle: Keyword,
    ) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), QError>>
    where
        R: Reader<Item = char, Err = QError> + 'static,
    {
        drop_left(and(
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn(format!("Expected: whitespace before {}", needle)),
            ),
            demand_keyword(needle),
        ))
    }

    pub fn read_any_digits<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        E: 'static,
    {
        super::str::one_or_more_if(is_digit)
    }

    //
    // Modify the result of a parser
    //

    //
    // Take multiple items
    //

    pub fn csv_zero_or_more<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, Vec<T>, E>>
    where
        R: Reader<Item = char, Err = E> + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        T: 'static,
        E: 'static,
    {
        zero_or_more(opt_seq2(
            source,
            crate::parser::pc::ws::zero_or_more_around(try_read(',')),
        ))
    }

    pub fn in_parenthesis<R, S, T>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, QError>>
    where
        R: Reader<Item = char, Err = QError> + 'static,
        S: Fn(R) -> ReaderResult<R, T, QError> + 'static,
        T: 'static,
    {
        map(
            seq3(
                try_read('('),
                source,
                demand(
                    try_read(')'),
                    QError::syntax_error_fn("Expected: closing parenthesis"),
                ),
            ),
            |(_, r, _)| r,
        )
    }
}
