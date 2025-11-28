use crate::pc::*;
use crate::pc_specific::*;
use crate::ParseError;

/// Comma separated list of items.
pub fn csv<I: Tokenizer + 'static, L: Parser<I>>(
    parser: L,
) -> impl Parser<I, Output = Vec<L::Output>> {
    delimited_by(parser, comma(), trailing_comma_error())
}

pub fn csv_non_opt<I: Tokenizer + 'static, P: Parser<I>>(
    parser: P,
    err: &str,
) -> impl Parser<I, Output = Vec<P::Output>> + NonOptParser<I> {
    csv(parser).or_syntax_error(err)
}

pub fn trailing_comma_error() -> ParseError {
    ParseError::syntax_error("Error: trailing comma")
}
