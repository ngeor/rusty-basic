use crate::common::{AtLocation, HasLocation, Locatable};
use crate::parser::pc::{Reader, ReaderResult, Undo};
use crate::parser::pc_specific::{
    is_letter, is_non_leading_identifier_with_dot, is_not_whole_keyword,
};
use crate::parser::Keyword;
use std::convert::TryFrom;
use std::marker::PhantomData;

pub trait Parser<R>
where
    R: Reader,
{
    type Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err>;

    fn convert_to_fn(self) -> Box<dyn Fn(R) -> ReaderResult<R, Self::Output, R::Err>>
    where
        Self: Sized + 'static,
    {
        let x = self;
        Box::new(move |reader| x.parse(reader))
    }

    fn and<B>(self, other: B) -> AndParser<Self, B>
    where
        Self: Sized + 'static,
        R: Undo<Self::Output>,
        B: Parser<R> + Sized + 'static,
    {
        AndParser {
            first: self,
            second: other,
        }
    }

    fn and_opt<B>(self, other: B) -> AndOptParser<Self, B>
    where
        Self: Sized,
        B: Sized + Parser<R>,
    {
        AndOptParser(self, other)
    }

    fn and_demand<B, F>(self, other: B, err_fn: F) -> AndDemand<Self, B, F>
    where
        Self: Sized,
        B: Sized + Parser<R>,
        F: Fn() -> R::Err,
    {
        AndDemand(self, other, err_fn)
    }

    fn and_rollback_if<B>(self, other: B) -> RollbackFirstIfSecond<Self, B>
    where
        R: Undo<Self::Output> + Undo<B::Output>,
        Self: Sized,
        B: Sized + Parser<R>,
    {
        RollbackFirstIfSecond(self, other)
    }

    fn map<F, U>(self, map: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Output) -> U,
    {
        Map { source: self, map }
    }

    fn keep_left<T, U>(self) -> KeepLeft<Self>
    where
        Self: Sized + Parser<R, Output = (T, U)>,
    {
        KeepLeft(self)
    }

    fn keep_right<T, U>(self) -> KeepRight<Self>
    where
        Self: Sized + Parser<R, Output = (T, U)>,
    {
        KeepRight(self)
    }

    fn keep_middle<A, B, C>(self) -> KeepMiddle<Self>
    where
        Self: Sized + Parser<R, Output = ((A, B), C)>,
    {
        KeepMiddle(self)
    }

    fn validate<F>(self, validation: F) -> Validate<Self, F>
    where
        Self: Sized,
        R: Undo<Self::Output>,
        F: Fn(&Self::Output) -> Result<bool, R::Err>,
    {
        Validate(self, validation)
    }

    fn with_pos(self) -> WithPos<Self>
    where
        Self: Sized,
    {
        WithPos(self)
    }

    fn or_throw<F>(self, f: F) -> OrThrow<Self, F>
    where
        Self: Sized,
        F: Fn() -> R::Err,
    {
        OrThrow(self, f)
    }
}

pub mod str {
    use super::*;

    //
    // Find a string
    //

    pub struct ReadString<R: Reader<Item = char>> {
        needle: &'static str,
        reader: PhantomData<R>,
    }

    pub fn read_string_p<R: Reader<Item = char>>(needle: &'static str) -> ReadString<R> {
        ReadString {
            needle,
            reader: PhantomData,
        }
    }

    impl<R> Parser<R> for ReadString<R>
    where
        R: Reader<Item = char>,
    {
        type Output = String;
        fn parse(&self, r: R) -> ReaderResult<R, String, R::Err> {
            let mut reader = r;
            let mut result = String::new();
            for n in self.needle.chars() {
                let res = reader.read();
                match res {
                    Ok((r, Some(ch))) => {
                        result.push(ch);
                        if ch.to_ascii_uppercase() == n.to_ascii_uppercase() {
                            reader = r;
                        } else {
                            return Ok((r.undo(result), None));
                        }
                    }
                    Ok((r, None)) => {
                        // EOF before matching all characters, undo collected and return None
                        return Ok((r.undo(result), None));
                    }
                    Err((r, err)) => {
                        // Error occurred, exit fast
                        return Err((r, err));
                    }
                }
            }
            Ok((reader, Some(result)))
        }
    }

    //
    // Read one or more characters that meet the predicate
    //

    pub struct ReadOneOrMoreWhile<F>(F);

    impl<R, F> Parser<R> for ReadOneOrMoreWhile<F>
    where
        R: Reader<Item = char>,
        F: Fn(char) -> bool,
    {
        type Output = String;
        fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
            let mut result = String::new();
            let mut r = reader;
            let mut has_more = true;
            while has_more {
                let (tmp, opt_item) = r.read()?;
                r = tmp;
                match opt_item {
                    Some(item) => {
                        if (self.0)(item) {
                            result.push(item);
                        } else {
                            r = r.undo_item(item);
                            has_more = false;
                        }
                    }
                    None => {
                        has_more = false;
                    }
                }
            }
            if result.is_empty() {
                Ok((r, None))
            } else {
                Ok((r, Some(result)))
            }
        }
    }

    pub fn read_one_or_more_while_p<F>(predicate: F) -> ReadOneOrMoreWhile<F>
    where
        F: Fn(char) -> bool,
    {
        ReadOneOrMoreWhile(predicate)
    }
}

//
// And
//

pub struct AndParser<A, B> {
    first: A,
    second: B,
}

impl<R, A, B> Parser<R> for AndParser<A, B>
where
    R: Reader + Undo<A::Output>,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.first.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.second.parse(reader)?;
                match opt_b {
                    Some(b) => Ok((reader, Some((a, b)))),
                    None => Ok((reader.undo(a), None)),
                }
            }
            None => Ok((reader, None)),
        }
    }
}

//
// AndOpt
//

pub struct AndOptParser<A, B>(A, B);

impl<R, A, B> Parser<R> for AndOptParser<A, B>
where
    R: Reader,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = (A::Output, Option<B::Output>);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                Ok((reader, Some((a, opt_b))))
            }
            None => Ok((reader, None)),
        }
    }
}

//
// AndDemand
//

pub struct AndDemand<A, B, F>(A, B, F);

impl<R, A, B, F> Parser<R> for AndDemand<A, B, F>
where
    R: Reader,
    A: Parser<R>,
    B: Parser<R>,
    F: Fn() -> R::Err,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                match opt_b {
                    Some(b) => Ok((reader, Some((a, b)))),
                    _ => Err((reader, (self.2)())),
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

//
// RollbackFirstIfSecond
//

pub struct RollbackFirstIfSecond<A, B>(A, B);

impl<R, A, B> Parser<R> for RollbackFirstIfSecond<A, B>
where
    R: Reader + Undo<A::Output> + Undo<B::Output>,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = A::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                match opt_b {
                    Some(b) => Ok((reader.undo(b).undo(a), None)),
                    None => Ok((reader, Some(a))),
                }
            }
            None => Ok((reader, None)),
        }
    }
}

//
// ReadOneIf
//

pub struct ReadOneIf<R: Reader, F> {
    reader: PhantomData<R>,
    predicate: F,
}

impl<R: Reader, F> ReadOneIf<R, F> {
    pub fn new(predicate: F) -> Self {
        Self {
            reader: PhantomData,
            predicate,
        }
    }
}

impl<R, F> Parser<R> for ReadOneIf<R, F>
where
    R: Reader,
    R::Item: Copy,
    F: Fn(R::Item) -> bool,
{
    type Output = R::Item;
    fn parse(&self, reader: R) -> ReaderResult<R, R::Item, R::Err> {
        let (reader, opt_item) = reader.read()?;
        match opt_item {
            Some(item) => {
                if (self.predicate)(item) {
                    Ok((reader, Some(item)))
                } else {
                    Ok((reader.undo_item(item), None))
                }
            }
            None => Ok((reader, None)),
        }
    }
}

pub fn read_one_if_p<R: Reader, F>(predicate: F) -> ReadOneIf<R, F> {
    ReadOneIf::new(predicate)
}

//
// ReadOne
//

pub struct ReadOne<R: Reader>(R::Item);

impl<R> Parser<R> for ReadOne<R>
where
    R: Reader,
    R::Item: Copy + Eq,
{
    type Output = R::Item;
    fn parse(&self, reader: R) -> ReaderResult<R, R::Item, R::Err> {
        let (reader, opt_item) = reader.read()?;
        match opt_item {
            Some(item) => {
                if self.0 == item {
                    Ok((reader, Some(item)))
                } else {
                    Ok((reader.undo_item(item), None))
                }
            }
            None => Ok((reader, None)),
        }
    }
}

pub fn read_one_p<R: Reader>(item: R::Item) -> ReadOne<R> {
    ReadOne(item)
}

//
// ReadOneIfTryFrom
//

pub struct ReadOneIfTryFrom<R, T>(PhantomData<R>, PhantomData<T>);

impl<R, T> Parser<R> for ReadOneIfTryFrom<R, T>
where
    R: Reader,
    R::Item: Copy,
    T: TryFrom<R::Item>,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, T, R::Err> {
        let (reader, opt_item) = reader.read()?;
        match opt_item {
            Some(item) => match T::try_from(item) {
                Ok(t) => Ok((reader, Some(t))),
                _ => Ok((reader.undo_item(item), None)),
            },
            None => Ok((reader, None)),
        }
    }
}

pub fn read_one_if_try_from_p<R, T>() -> ReadOneIfTryFrom<R, T>
where
    R: Reader,
    R::Item: Copy,
    T: TryFrom<R::Item>,
{
    ReadOneIfTryFrom(PhantomData, PhantomData)
}

//
// ReadOneOrMoreWhile
//

pub struct ReadOneOrMoreWhile<R: Reader, F> {
    reader: PhantomData<R>,
    predicate: F,
}

impl<R: Reader, F> ReadOneOrMoreWhile<R, F> {
    pub fn new(predicate: F) -> Self {
        Self {
            reader: PhantomData,
            predicate,
        }
    }
}

impl<R, F> Parser<R> for ReadOneOrMoreWhile<R, F>
where
    R: Reader,
    R::Item: Copy,
    F: Fn(R::Item) -> bool,
{
    type Output = Vec<R::Item>;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let mut items: Vec<R::Item> = vec![];
        let mut r = reader;
        let mut has_more = true;
        while has_more {
            let (tmp, opt_item) = r.read()?;
            r = tmp;
            match opt_item {
                Some(item) => {
                    if (self.predicate)(item) {
                        items.push(item);
                    } else {
                        r = r.undo_item(item);
                        has_more = false;
                    }
                }
                None => {
                    has_more = false;
                }
            }
        }
        if items.is_empty() {
            Ok((r, None))
        } else {
            Ok((r, Some(items)))
        }
    }
}

pub fn read_one_or_more_while_p<R: Reader, F>(predicate: F) -> ReadOneOrMoreWhile<R, F> {
    ReadOneOrMoreWhile::new(predicate)
}

//
// map
//

pub struct Map<S, F> {
    source: S,
    map: F,
}

impl<S, R, U, F> Parser<R> for Map<S, F>
where
    R: Reader,
    S: Parser<R>,
    F: Fn(S::Output) -> U,
{
    type Output = U;
    fn parse(&self, reader: R) -> ReaderResult<R, U, R::Err> {
        let (reader, opt_item) = self.source.parse(reader)?;
        match opt_item {
            Some(item) => {
                let mapped_item = (self.map)(item);
                Ok((reader, Some(mapped_item)))
            }
            None => Ok((reader, None)),
        }
    }
}

//
// keep left
//

pub struct KeepLeft<S>(S);

impl<R, S, T, U> Parser<R> for KeepLeft<S>
where
    R: Reader,
    S: Parser<R, Output = (T, U)>,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        let mapped_opt_item = opt_item.map(|(t, _)| t);
        Ok((reader, mapped_opt_item))
    }
}

//
// keep right
//

pub struct KeepRight<S>(S);

impl<R, S, T, U> Parser<R> for KeepRight<S>
where
    R: Reader,
    S: Parser<R, Output = (T, U)>,
{
    type Output = U;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        let mapped_opt_item = opt_item.map(|(_, u)| u);
        Ok((reader, mapped_opt_item))
    }
}

//
// keep middle
//

pub struct KeepMiddle<S>(S);

impl<R, S, A, B, C> Parser<R> for KeepMiddle<S>
where
    R: Reader,
    S: Parser<R, Output = ((A, B), C)>,
{
    type Output = B;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        let mapped_opt_item = opt_item.map(|((_, b), _)| b);
        Ok((reader, mapped_opt_item))
    }
}

// with pos

pub struct WithPos<S>(S);

impl<S, R> Parser<R> for WithPos<S>
where
    R: Reader + HasLocation,
    S: Parser<R>,
{
    type Output = Locatable<S::Output>;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let pos = reader.pos();
        let (reader, opt_item) = self.0.parse(reader)?;
        Ok((reader, opt_item.map(|item| item.at(pos))))
    }
}

//
// validate
//

pub struct Validate<S, F>(S, F);

impl<R, S, F> Parser<R> for Validate<S, F>
where
    R: Reader + Undo<S::Output>,
    S: Parser<R>,
    F: Fn(&S::Output) -> Result<bool, R::Err>,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => match (self.1)(&item) {
                Ok(true) => Ok((reader, Some(item))),
                Ok(false) => Ok((reader.undo(item), None)),
                Err(err) => Err((reader, err)),
            },
            None => Ok((reader, None)),
        }
    }
}

//
// or_throw
//

pub struct OrThrow<S, F>(S, F);

impl<R, S, F> Parser<R> for OrThrow<S, F>
where
    R: Reader,
    S: Parser<R>,
    F: Fn() -> R::Err,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        if opt_item.is_some() {
            Ok((reader, opt_item))
        } else {
            Err((reader, (self.1)()))
        }
    }
}

//
// keyword
//

pub fn keyword_p<R>(keyword: Keyword) -> impl Parser<R, Output = (Keyword, String)>
where
    R: Reader<Item = char> + 'static,
{
    str::read_string_p(keyword.as_str())
        .and_rollback_if(read_one_if_p(is_not_whole_keyword))
        .map(move |keyword_as_str| (keyword, keyword_as_str))
}

//
// any_identifier_with_dot
//

pub fn any_identifier_with_dot_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char> + 'static,
{
    read_one_if_p(is_letter)
        .and_opt(str::read_one_or_more_while_p(
            is_non_leading_identifier_with_dot,
        ))
        .map(|(first_letter, opt_letters)| {
            let mut s: String = String::new();
            s.push(first_letter);
            if let Some(letters) = opt_letters {
                s.push_str(letters.as_ref());
            }
            s
        })
}
