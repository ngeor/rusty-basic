use tokenizers::Token;
use crate::common::QError;
use crate::parser::base::delimited_pc::DelimitedTrait;
use crate::parser::base::parsers::{NonOptParser, Parser};
use crate::parser::specific::item_p;
use crate::parser::specific::whitespace::WhitespaceTrait;

pub fn csv_one_or_more_non_opt<P>(parser: P) -> impl NonOptParser<Output = Vec<P::Output>>
where
    P: NonOptParser,
{
    parser.one_or_more_delimited_by_non_opt(comma_surrounded_by_opt_ws())
}

pub fn csv_one_or_more<P>(parser: P) -> impl Parser<Output = Vec<P::Output>>
where
    P: Parser,
{
    parser.one_or_more_delimited_by(
        comma_surrounded_by_opt_ws(),
        QError::syntax_error("Trailing comma"),
    )
}

pub fn csv_zero_or_more<P>(parser: P) -> impl NonOptParser<Output = Vec<P::Output>>
where
    P: Parser,
{
    parser.one_or_more_delimited_by(
        comma_surrounded_by_opt_ws(),
        QError::syntax_error("Trailing comma"),
    )
}

pub fn csv_zero_or_more_allow_missing<P>(
    parser: P,
) -> impl NonOptParser<Output = Vec<Option<P::Output>>>
where
    P: Parser,
{
    parser.one_or_more_delimited_by_allow_missing(comma_surrounded_by_opt_ws())
}

pub fn comma_surrounded_by_opt_ws() -> impl Parser<Output = (Option<Token>, Token, Option<Token>)> {
    item_p(',').surrounded_by_ws_preserving()
}
