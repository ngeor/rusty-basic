use crate::common::{HasLocation, Locatable, QError};
use crate::parser::expression::expression_node_p;
use crate::parser::name::bare_name_p;
use crate::parser::pc::{whitespace_p, BinaryParser, Parser, Reader, UnaryFnParser, UnaryParser};
use crate::parser::pc_specific::{keyword_followed_by_whitespace_p, keyword_pair_p, PcSpecific};
use crate::parser::{Expression, Keyword, OnErrorOption, Statement};

pub fn statement_on_error_go_to_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_pair_p(Keyword::On, Keyword::Error)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace"))
        .and_demand(
            next()
                .or(goto())
                .or_syntax_error("Expected: GOTO or RESUME"),
        )
        .map(|(_, r)| Statement::OnError(r))
}

fn next<R>() -> impl Parser<R, Output = OnErrorOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_pair_p(Keyword::Resume, Keyword::Next).map(|_| OnErrorOption::Next)
}

fn goto<R>() -> impl Parser<R, Output = OnErrorOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .and_demand(
            goto_label()
                .or(goto_zero())
                .or_syntax_error("Expected: label or 0"),
        )
        .keep_right()
}

fn goto_label<R>() -> impl Parser<R, Output = OnErrorOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    bare_name_p().map(OnErrorOption::Label)
}

fn goto_zero<R>() -> impl Parser<R, Output = OnErrorOption>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression_node_p().and_then(|Locatable { element, .. }| match element {
        Expression::IntegerLiteral(0) => Ok(OnErrorOption::Zero),
        _ => Err(QError::syntax_error("Expected: label or 0")),
    })
}
