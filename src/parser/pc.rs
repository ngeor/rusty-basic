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
// Undo support
// ========================================================

pub mod undo {
    use super::{Reader, Undo};
    use crate::common::Locatable;

    impl<R: Reader<Item = char>> Undo<char> for R {
        fn undo(self, item: char) -> Self {
            self.undo_item(item)
        }
    }

    impl<R: Reader<Item = char>> Undo<Locatable<char>> for R {
        fn undo(self, item: Locatable<char>) -> Self {
            self.undo_item(item.element)
        }
    }

    impl<R: Reader<Item = char>> Undo<String> for R {
        fn undo(self, s: String) -> Self {
            let mut result = self;
            for ch in s.chars().rev() {
                result = result.undo_item(ch);
            }
            result
        }
    }

    impl<R: Reader<Item = char>> Undo<(String, Locatable<char>)> for R {
        fn undo(self, item: (String, Locatable<char>)) -> Self {
            let (a, b) = item;
            self.undo(b).undo(a)
        }
    }

    // undo char followed by opt ws
    impl<R: Reader<Item = char>> Undo<(char, Option<String>)> for R {
        fn undo(self, item: (char, Option<String>)) -> Self {
            let (a, b) = item;
            self.undo(b.unwrap_or_default()).undo_item(a)
        }
    }

    // undo char preceded by opt ws
    impl<B, R: Reader<Item = char> + Undo<String> + Undo<B>> Undo<(Option<String>, B)> for R {
        fn undo(self, item: (Option<String>, B)) -> Self {
            let (a, b) = item;
            self.undo(b).undo(a.unwrap_or_default())
        }
    }
}

// ========================================================
// when Item : Copy
// ========================================================

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
    map::source_and_then_some(source, move |reader, ch| {
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
    filter(R::read, predicate)
}

pub fn read<R, T>(needle: T) -> Box<dyn Fn(R) -> ReaderResult<R, R::Item, R::Err>>
where
    R: Reader<Item = T> + Undo<T> + 'static,
    T: Copy + PartialEq + 'static,
{
    read_if(move |ch| ch == needle)
}

// ========================================================
// Parsers that apply a function to a source
// ========================================================

pub mod map {
    use super::*;

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
// simple parsing functions
// ========================================================

pub mod common {
    use super::map::*;
    use super::*;

    // ========================================================
    // simple parsing combinators
    // ========================================================

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

    pub fn seq2<R, S1, S2, T1, T2, E>(
        first: S1,
        second: S2,
        tag: &'static str,
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
            None => panic!(
                "`seq2` second function returned None, wrap it in a `demand` (tag: {})",
                tag
            ),
        })
    }

    pub fn seq3<R, S1, S2, S3, T1, T2, T3, E>(
        first: S1,
        second: S2,
        third: S3,
        tag: &'static str,
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
        map(seq2(first, seq2(second, third, tag), tag), |(a, (b, c))| {
            (a, b, c)
        })
    }

    pub fn seq4<R, S1, S2, S3, S4, T1, T2, T3, T4, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        tag: &'static str,
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
            seq2(first, seq3(second, third, fourth, tag), tag),
            |(a, (b, c, d))| (a, b, c, d),
        )
    }

    pub fn seq5<R, S1, S2, S3, S4, S5, T1, T2, T3, T4, T5, E>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        fifth: S5,
        tag: &'static str,
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
            seq2(first, seq4(second, third, fourth, fifth, tag), tag),
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
        tag: &'static str,
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
            seq2(first, seq5(second, third, fourth, fifth, sixth, tag), tag),
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
        tag: &'static str,
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
            seq2(
                first,
                seq6(second, third, fourth, fifth, sixth, seventh, tag),
                tag,
            ),
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

    pub fn many<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, Vec<T>, E>>
    where
        R: Reader + 'static,
        S: Fn(R) -> ReaderResult<R, T, E> + 'static,
        E: 'static,
    {
        Box::new(move |char_reader| {
            let mut result: Vec<T> = vec![];
            let mut cr: R = char_reader;
            loop {
                match source(cr) {
                    Ok((next_cr, opt_res)) => {
                        cr = next_cr;
                        match opt_res {
                            Some(t) => {
                                result.push(t);
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
// dealing with characters and strings
// ========================================================

pub mod str {
    use super::*;

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
        tag: &'static str,
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
                tag,
            ),
            |(l, _, r)| (l, r),
        )
    }

    pub fn seq4<R, S1, S2, S3, S4, T1, T2, T3, T4, E, FE>(
        first: S1,
        second: S2,
        third: S3,
        fourth: S4,
        err_fn_fn_expected_whitespace: FE,
        tag: &'static str,
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
                tag,
            ),
            |(a, _, b, _, c, _, d)| (a, b, c, d),
        )
    }
}
