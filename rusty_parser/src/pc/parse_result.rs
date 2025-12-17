pub enum ParseResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> ParseResult<T, E> {
    pub fn map<F, U>(self, f: F) -> ParseResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            ParseResult::Ok(t) => ParseResult::Ok(f(t)),
            ParseResult::Err(e) => ParseResult::Err(e),
        }
    }

    pub fn flat_map<F, U>(self, f: F) -> ParseResult<U, E>
    where
        F: FnOnce(T) -> ParseResult<U, E>,
    {
        match self {
            ParseResult::Ok(t) => f(t),
            ParseResult::Err(e) => ParseResult::Err(e),
        }
    }
}
