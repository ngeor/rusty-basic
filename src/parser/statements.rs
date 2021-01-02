use crate::common::*;
use crate::parser::pc::combine::combine_if_first_some;
use crate::parser::pc::common::*;
use crate::parser::pc::str::{map_to_str, zero_or_more_if_leading_remaining};
use crate::parser::pc::ws::{is_eol, is_eol_or_whitespace, is_whitespace};
use crate::parser::pc::*;
use crate::parser::pc2::binary::BinaryParser;
use crate::parser::pc2::many::ManyParser;
use crate::parser::pc2::text::{string_while_p, TextParser};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::{any_p, item_p, Parser};
use crate::parser::pc_specific::with_pos;
use crate::parser::statement;
use crate::parser::types::*;

pub fn single_line_non_comment_statements<R>(
) -> Box<dyn Fn(R) -> ReaderResult<R, StatementNodes, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    crate::parser::pc::ws::one_or_more_leading(map_default_to_not_found(
        many_with_terminating_indicator(opt_seq2(
            with_pos(statement::single_line_non_comment_statement()),
            crate::parser::pc::ws::zero_or_more_around(read(':')),
        )),
    ))
}

pub fn single_line_statements<R>() -> Box<dyn Fn(R) -> ReaderResult<R, StatementNodes, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    crate::parser::pc::ws::one_or_more_leading(map_default_to_not_found(
        many_with_terminating_indicator(opt_seq2(
            with_pos(statement::single_line_statement()),
            crate::parser::pc::ws::zero_or_more_around(read(':')),
        )),
    ))
}

fn parse_first_statement_separator<R, FE>(
    err_fn: FE,
) -> Box<dyn Fn(R) -> ReaderResult<R, String, QError>>
where
    R: Reader<Item = char, Err = QError>,
    FE: Fn() -> QError + 'static,
{
    // Allowed to read:
    // <ws*> ' -- and if it is comment, put it back
    // <ws*> :
    // <ws*> EOL
    Box::new(move |reader| {
        let mut r = reader;
        let mut buf: String = String::new();
        let mut found = false;
        loop {
            match r.read() {
                Ok((tmp, Some(ch))) => {
                    r = tmp;
                    if crate::parser::pc::ws::is_whitespace(ch) {
                        if found {
                            buf.push(ch);
                        } else {
                            // skip over it, so that the error will hit the
                            // first non-whitespace character
                        }
                    } else if ch == ':' || crate::parser::pc::ws::is_eol(ch) {
                        // exit
                        buf.push(ch);
                        found = true;
                    } else if ch == '\'' {
                        return Ok((r.undo(ch), Some(buf)));
                    } else {
                        r = r.undo(ch);
                        break;
                    }
                }
                Ok((tmp, None)) => {
                    r = tmp;
                    break;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        if found {
            Ok((r, Some(buf)))
        } else {
            Err((r, err_fn()))
        }
    })
}

// When `statements` is called, it must always read first the first statement separator.
// `top_level_token` handles the case where the first statement does not start with
// a separator.
#[deprecated]
pub fn statements<R, S, X, FE>(
    exit_source: S,
    err_fn: FE,
) -> Box<dyn Fn(R) -> ReaderResult<R, StatementNodes, QError>>
where
    S: Fn(R) -> ReaderResult<R, X, QError> + 'static,
    R: Reader<Item = char, Err = QError> + Undo<X> + HasLocation + 'static,
    FE: Fn() -> QError + 'static,
{
    drop_left(and(
        parse_first_statement_separator(err_fn),
        many_with_terminating_indicator(move |reader| {
            match exit_source(reader) {
                Ok((reader, Some(x))) => {
                    // found the exit
                    Ok((reader.undo(x), None))
                }
                Ok((reader, None)) => {
                    // did not find the exit, we can parse a statement
                    statement_node_and_separator()(reader)
                }
                Err(err) => {
                    // something else happened, abort
                    Err(err)
                }
            }
        }),
    ))
}

pub fn statements_p<R, S>(exit_source: S) -> impl Parser<R, Output = StatementNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + Undo<S::Output> + 'static,
    S: Parser<R>,
{
    first_statement_separator_p()
        .and(
            exit_source
                .negate()
                .and(statement_node_and_separator_p().keep_left())
                .keep_right()
                .zero_or_more(),
        )
        .keep_right()
    // first separator
    // loop
    //      negate exit source
    //      statement node and separator
}

// <ws>* '\'' (undoing it)
// <ws>* ':' <ws*>
// <ws>* EOL <ws | eol>*
fn first_statement_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    comment_separator_p()
        .or(colon_separator_p())
        .or(eol_separator_p())
        .preceded_by_opt_ws()
        .stringify()
}

// '\'' (undoing it)
fn comment_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    item_p('\'').peek_reader_item().stringify()
}

// ':' <ws>*
fn colon_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    item_p(':').followed_by_opt_ws().stringify()
}

// <eol> < ws | eol >*
fn eol_separator_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    any_p()
        .filter_reader_item(is_eol)
        .and_opt(string_while_p(is_eol_or_whitespace))
        .stringify()
}

fn statement_node_and_separator<R>(
) -> Box<dyn Fn(R) -> ReaderResult<R, (StatementNode, Option<String>), QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    combine_if_first_some(
        // first part is the statement node
        statement::statement_node(),
        // second part the separator, which is used by the `many_with_terminating_indicator` to understand if it's the terminal statement
        |s_node| {
            // if the statement is a comment, the only valid separator is EOL (or EOF)
            let is_comment = match s_node.as_ref() {
                Statement::Comment(_) => true,
                _ => false,
            };
            if is_comment {
                comment_separator()
            } else {
                crate::parser::pc::ws::zero_or_more_leading(non_comment_separator())
            }
        },
    )
}

fn statement_node_and_separator_p<R>() -> impl Parser<R, Output = (StatementNode, Option<String>)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    statement::statement_p()
        .with_pos()
        .and_opt_factory(|Locatable { element, .. }| {
            if let Statement::Comment(_) = element {
                eol_separator_p().box_dyn()
            } else {
                first_statement_separator_p().box_dyn()
            }
        })
}

fn comment_separator<R>() -> Box<dyn Fn(R) -> ReaderResult<R, String, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map_default_to_not_found(zero_or_more_if_leading_remaining(
        is_eol,
        is_eol_or_whitespace,
    ))
}

fn non_comment_separator<R>() -> Box<dyn Fn(R) -> ReaderResult<R, String, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    // ws* : ws*
    // ws* eol (ws | eol)*
    // ws*' comment
    or_vec(vec![
        map_default_to_not_found(zero_or_more_if_leading_remaining(
            |ch| ch == ':',
            is_whitespace,
        )),
        comment_separator(),
        map_to_str(peek('\'')),
        Box::new(expected_end_of_statement),
    ])
}

fn expected_end_of_statement<R, T>(reader: R) -> ReaderResult<R, T, QError>
where
    R: Reader<Err = QError> + 'static,
    T: 'static,
{
    reader.read().and_then(|(reader, opt_res)| match opt_res {
        Some(ch) => {
            // undo so that the error will be positioned at the offending character
            Err((
                reader.undo_item(ch),
                QError::syntax_error("Expected: end-of-statement"),
            ))
        }
        None => Ok((reader, None)),
    })
}
