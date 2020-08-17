use super::{Locatable, Transactional};

//
// # Parser combinators that are applicable to any parser
//

//
// ## Combine two parsers into one
//

/// Creates a new parsers that zips together the result of two parsers.
///
/// The new parser will only return `Some(Ok)` if both parsers do that.
pub fn and<I, A, B, FA, FB, E>(
    first_parser: FA,
    second_parser: FB,
) -> impl Fn(&mut I) -> Option<Result<(A, B), E>>
where
    FA: Fn(&mut I) -> Option<Result<A, E>>,
    FB: Fn(&mut I) -> Option<Result<B, E>>,
{
    move |input| match first_parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(first)) => match second_parser(input) {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(second)) => Some(Ok((first, second))),
        },
    }
}

/// Creates a new parser that zips together the result of two parsers.
///
/// The second parser may return None.
///
/// The new parser will return:
///
/// - None, if the first parser returns None
/// - Some(Err), if any of the parsers return Some(Err)
pub fn zip_allow_right_none<I, A, B, FA, FB, E>(
    first_parser: FA,
    second_parser: FB,
) -> impl Fn(&mut I) -> Option<Result<(A, Option<B>), E>>
where
    FA: Fn(&mut I) -> Option<Result<A, E>>,
    FB: Fn(&mut I) -> Option<Result<B, E>>,
{
    move |input| match first_parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(first)) => match second_parser(input) {
            None => Some(Ok((first, None))),
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(second)) => Some(Ok((first, Some(second)))),
        },
    }
}

/// Creates a new parser that zips together the result of two parsers.
///
/// The first parser may return None.
///
/// The new parser will return:
///
/// - None, if the second parser returns None
/// - Some(Err), if any of the parsers return Some(Err)
pub fn zip_allow_left_none<I, A, B, FA, FB, E>(
    first_parser: FA,
    second_parser: FB,
) -> impl Fn(&mut I) -> Option<Result<(Option<A>, B), E>>
where
    FA: Fn(&mut I) -> Option<Result<A, E>>,
    FB: Fn(&mut I) -> Option<Result<B, E>>,
{
    move |input| match first_parser(input) {
        None => match second_parser(input) {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(second)) => Some(Ok((None, second))),
        },
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(first)) => match second_parser(input) {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(second)) => Some(Ok((Some(first), second))),
        },
    }
}

//
// ## Transform the result of a parser
//

/// Creates a new parser that maps the result of the given parser with the
/// specified function.
pub fn apply<I, T, U, FMap, FPC, E>(f: FMap, parser: FPC) -> impl Fn(&mut I) -> Option<Result<U, E>>
where
    FMap: Fn(T) -> U,
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(x)) => Some(Ok(f(x))),
    }
}

/// Creates a new parser that maps the result of the given parser with the
/// standard `From` trait.
pub fn apply_from<I, T, U, FPC, E>(parser: FPC) -> impl Fn(&mut I) -> Option<Result<U, E>>
where
    U: From<T>,
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(x)) => Some(Ok(U::from(x))),
    }
}

/// Creates a new parser that replaces the result of the given parser
/// with the result of the specified function.
///
/// None and Some(Err) results are never replaced. The mapper function only
/// processes Some(Ok) results.
pub fn switch<I, T, E, U, FMap, FPC>(
    f: FMap,
    parser: FPC,
) -> impl Fn(&mut I) -> Option<Result<U, E>>
where
    FMap: Fn(T) -> Option<U>,
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(x)) => f(x).map(|opt| Ok(opt)),
    }
}

// Creates a new parser that replaces the result of the given parser with the
// result of the specified function. The function can even return `None` or
// `Err`.
pub fn switch_err<I, T, E, U, FMap, FPC>(
    f: FMap,
    parser: FPC,
) -> impl Fn(&mut I) -> Option<Result<U, E>>
where
    FMap: Fn(T) -> Option<Result<U, E>>,
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(x)) => f(x),
    }
}

/// Creates a parser that returns `None` if the given predicate is not satisfied.
pub fn filter<I, T, E, FMap, FPC>(f: FMap, parser: FPC) -> impl Fn(&mut I) -> Option<Result<T, E>>
where
    FMap: Fn(&T) -> bool,
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    switch(move |x| if f(&x) { Some(x) } else { None }, parser)
}

/// Creates a parser that returns `None` if the given predicate is satisfied.
pub fn exclude<I, T, E, FMap, FPC>(f: FMap, parser: FPC) -> impl Fn(&mut I) -> Option<Result<T, E>>
where
    FMap: Fn(&T) -> bool,
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    switch(move |x| if f(&x) { None } else { Some(x) }, parser)
}

//
// ## Other
//

/// Creates a new parser that calls the given parser multiple times
/// until it fails or returns `None`.
pub fn many<I, T, E, FPC>(parser: FPC) -> impl Fn(&mut I) -> Option<Result<Vec<T>, E>>
where
    FPC: Fn(&mut I) -> Option<Result<T, E>>,
{
    move |input| {
        let mut result: Vec<T> = vec![];
        loop {
            match parser(input) {
                Some(Ok(x)) => {
                    result.push(x);
                }
                Some(Err(err)) => return Some(Err(err)),
                None => {
                    break;
                }
            }
        }
        Some(Ok(result))
    }
}

//
// # Parser combinators for Locatable
//

/// Creates a parser that maps the contents of locatable nodes to different
/// contents by using the given mapper function.
pub fn map_locatable<I, T, U, E, F, P>(
    f: F,
    parser: P,
) -> impl Fn(&mut I) -> Option<Result<Locatable<U>, E>>
where
    F: Fn(T) -> U,
    P: Fn(&mut I) -> Option<Result<Locatable<T>, E>>,
{
    // we can't use the `apply` function because rust does not like the closures
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(locatable)) => {
            let Locatable { element, pos } = locatable;
            Some(Ok(Locatable::new(f(element), pos)))
        }
    }
}

/// Creates a parser that maps the contents of locatable nodes to different
/// contents by using the standard `From` trait.
pub fn map_from_locatable<I, T, U, E, P>(
    parser: P,
) -> impl Fn(&mut I) -> Option<Result<Locatable<U>, E>>
where
    U: From<T>,
    P: Fn(&mut I) -> Option<Result<Locatable<T>, E>>,
{
    // we can't use the `apply` function because rust does not like the closures
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(locatable)) => {
            let Locatable { element, pos } = locatable;
            Some(Ok(Locatable::new(U::from(element), pos)))
        }
    }
}

/// Creates a parser that drops the location of locatable elements.
pub fn drop_location<I, T, E, P>(parser: P) -> impl Fn(&mut I) -> Option<Result<T, E>>
where
    P: Fn(&mut I) -> Option<Result<Locatable<T>, E>>,
{
    // we can't use the `apply` function because rust does not like the closures
    move |input| match parser(input) {
        None => None,
        Some(Err(err)) => Some(Err(err)),
        Some(Ok(locatable)) => {
            let Locatable { element, .. } = locatable;
            Some(Ok(element))
        }
    }
}

//
// # Parser combinators for Transactional
//

/// Creates a new parser that wraps the given parser inside a transaction.
///
/// The transaction is committed if the result is Some(Ok),
/// otherwise it is rolled back.
pub fn in_transaction_pc<TR, T, FPC, E>(parser: FPC) -> impl Fn(&mut TR) -> Option<Result<T, E>>
where
    TR: Transactional,
    FPC: Fn(&mut TR) -> Option<Result<T, E>>,
{
    move |t| {
        t.begin_transaction();
        let result = parser(t);
        match &result {
            Some(Ok(_)) => t.commit_transaction(),
            _ => t.rollback_transaction(),
        };
        result
    }
}

/// Creates a new parsers that returns the successful result of one
/// of the two given parsers.
pub fn or<I, T, F1, F2, E>(
    first_parser: F1,
    second_parser: F2,
) -> impl Fn(&mut I) -> Option<Result<T, E>>
where
    I: Transactional,
    F1: Fn(&mut I) -> Option<Result<T, E>>,
    F2: Fn(&mut I) -> Option<Result<T, E>>,
{
    move |input| {
        input.begin_transaction();
        match first_parser(input) {
            None => {
                // TODO reduce duplication
                input.rollback_transaction();
                input.begin_transaction();
                let result = second_parser(input);
                match &result {
                    Some(Ok(_)) => input.commit_transaction(),
                    _ => input.rollback_transaction(),
                };
                result
            }
            Some(Err(err)) => {
                input.rollback_transaction();
                Some(Err(err))
            }
            Some(Ok(first)) => {
                input.commit_transaction();
                Some(Ok(first))
            }
        }
    }
}

/// Creates a new parser that returns the result from the first parser that
/// will return `Some()`.
pub fn or_vec<I, T, E>(
    parsers: Vec<Box<dyn Fn(&mut I) -> Option<Result<T, E>>>>,
) -> impl Fn(&mut I) -> Option<Result<T, E>>
where
    I: Transactional,
{
    move |input| {
        input.begin_transaction();
        for p in parsers.iter() {
            match p(input) {
                Some(Err(err)) => {
                    input.rollback_transaction();
                    return Some(Err(err));
                }
                Some(Ok(x)) => {
                    input.commit_transaction();
                    return Some(Ok(x));
                }
                _ => {
                    input.rollback_transaction();
                    input.begin_transaction();
                }
            }
        }
        input.rollback_transaction();
        None
    }
}
