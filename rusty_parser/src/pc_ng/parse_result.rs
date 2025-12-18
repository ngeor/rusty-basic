pub enum ParseResult<I, O, E> {
    Ok(I, O),
    None(I),
    Expected(I, String),
    Err(I, E),
}

impl<I, O, E> ParseResult<I, O, E> {
    pub fn flat_map<F, U>(self, f: F) -> ParseResult<I, U, E>
    where
        F: FnOnce(I, O) -> ParseResult<I, U, E>,
    {
        match self {
            ParseResult::Ok(i, o) => f(i, o),
            ParseResult::None(i) => ParseResult::None(i),
            ParseResult::Expected(i, s) => ParseResult::Expected(i, s),
            ParseResult::Err(i, e) => ParseResult::Err(i, e),
        }
    }

    pub fn map<F, U>(self, f: F) -> ParseResult<I, U, E>
    where
        F: FnOnce(O) -> U,
    {
        match self {
            ParseResult::Ok(i, o) => ParseResult::Ok(i, f(o)),
            ParseResult::None(i) => ParseResult::None(i),
            ParseResult::Expected(i, s) => ParseResult::Expected(i, s),
            ParseResult::Err(i, e) => ParseResult::Err(i, e),
        }
    }
}
