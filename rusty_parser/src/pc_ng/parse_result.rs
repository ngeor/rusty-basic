/// The successful result of a parsing operation.
/// Consists of two elements:
/// - I: the remaining input
/// - O: the parsed value
pub type ParseOk<I, O> = (I, O);

/// The unsuccessful result of a parsing operation.
/// Consists of two elements:
/// - A flag indicating if the error is fatal or not.
/// - E: the error.
pub type ParseErr<E> = (bool, E);

/// Creates a failed result containing the default parse error (non fatal).
pub fn default_parse_error<T, E>() -> Result<T, ParseErr<E>>
where
    E: Default,
{
    Err((false, E::default()))
}

/// The parse result is an alias for the standard Result type,
/// where the Ok is a [ParseOk]
/// and the Err is a [ParseErr].
pub type ParseResult<I, O, E> = Result<ParseOk<I, O>, ParseErr<E>>;

pub trait ParseResultTrait<I, O, E> {
    fn map_ok<F, U>(self, mapper: F) -> ParseResult<I, U, E>
    where
        F: FnOnce(O) -> U;

    fn flat_map<F, U>(self, mapper: F) -> ParseResult<I, U, E>
    where
        F: FnOnce(I, O) -> ParseResult<I, U, E>;

    fn filter<F>(self, predicate: F) -> ParseResult<I, O, E>
    where
        F: FnOnce(&O) -> bool,
        E: Default;
}

impl<I, O, E> ParseResultTrait<I, O, E> for ParseResult<I, O, E> {
    fn map_ok<F, U>(self, mapper: F) -> ParseResult<I, U, E>
    where
        F: FnOnce(O) -> U,
    {
        match self {
            Ok((i, o)) => Ok((i, mapper(o))),
            Err(e) => Err(e),
        }
    }

    fn flat_map<F, U>(self, mapper: F) -> ParseResult<I, U, E>
    where
        F: FnOnce(I, O) -> ParseResult<I, U, E>,
    {
        match self {
            Ok((i, o)) => mapper(i, o),
            Err(e) => Err(e),
        }
    }

    fn filter<F>(self, predicate: F) -> ParseResult<I, O, E>
    where
        F: FnOnce(&O) -> bool,
        E: Default,
    {
        match self {
            Ok((i, o)) => {
                if predicate(&o) {
                    Ok((i, o))
                } else {
                    default_parse_error()
                }
            }
            Err(e) => Err(e),
        }
    }
}
