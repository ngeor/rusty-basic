mod built_in_function;
mod built_in_sub;
mod close;
mod cls;
mod color;
mod data;
mod def_seg;
mod field;
mod get;
mod input;
mod len;
mod line_input;
mod locate;
mod lset;
mod name;
mod open;
mod put;
mod read;
mod string_fn;
mod view_print;
mod width;

pub use self::built_in_function::BuiltInFunction;
pub use self::built_in_sub::BuiltInSub;

use crate::expression::expression_pos_p;
use crate::expression::file_handle::file_handle_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::{lazy_parser, FileHandle};
use crate::{Expression, ExpressionPos, Expressions, Keyword, Statement};
use rusty_common::{AtPos, Position, Positioned};

// Parses built-in subs which have a special syntax.
lazy_parser!(pub fn parse<Output=Statement> ; struct LazyParser ; OrParser::new(vec![
    Box::new(close::parse()),
    Box::new(color::parse()),
    Box::new(data::parse()),
    Box::new(def_seg::parse()),
    Box::new(field::parse()),
    Box::new(get::parse()),
    Box::new(input::parse()),
    Box::new(line_input::parse()),
    Box::new(locate::parse()),
    Box::new(lset::parse()),
    Box::new(name::parse()),
    Box::new(open::parse()),
    Box::new(put::parse()),
    Box::new(read::parse()),
    Box::new(view_print::parse()),
    Box::new(width::parse()),
]));

// needed for built-in functions that are also keywords (e.g. LEN), so they
// cannot be parsed by the `word` module.
pub fn built_in_function_call_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Expression> {
    len::parse().or(string_fn::parse())
}

/// Parses built-in subs with optional arguments.
/// Used only by `COLOR` and `LOCATE`.
fn parse_built_in_sub_with_opt_args<I: Tokenizer + 'static>(
    k: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<I, Output = Statement> {
    seq3(
        keyword(k),
        whitespace().no_incomplete(),
        csv_allow_missing(),
        move |_, _, opt_args| {
            Statement::BuiltInSubCall(built_in_sub, map_opt_args_to_flags(opt_args))
        },
    )
}

/// Maps optional arguments to arguments, inserting a dummy first argument indicating
/// which arguments were present in the form of a bit mask.
fn map_opt_args_to_flags(args: Vec<Option<ExpressionPos>>) -> Expressions {
    let mut result: Expressions = vec![];
    let mut mask = 1;
    let mut flags = 0;
    for arg in args {
        if let Some(arg) = arg {
            flags |= mask;
            result.push(arg);
        }
        mask <<= 1;
    }
    result.insert(
        0,
        Expression::IntegerLiteral(flags).at_pos(Position::start()),
    );
    result
}

/// Comma separated list of items, allowing items to be missing between commas.
fn csv_allow_missing<I: Tokenizer + 'static>(
) -> impl Parser<I, Output = Vec<Option<ExpressionPos>>> + NonOptParser<I> {
    parse_delimited_to_items(opt_zip(expression_pos_p(), comma()), trailing_comma_error())
        .allow_default()
}

/// Used in `INPUT` and `LINE INPUT`, parsing an optional file number.
fn opt_file_handle_comma_p<I: Tokenizer + 'static>(
) -> impl Parser<I, Output = Option<Positioned<FileHandle>>> + NonOptParser<I> {
    seq2(file_handle_p(), comma().no_incomplete(), |l, _| l).allow_none()
}

/// Used in `INPUT` and `LINE INPUT`, converts an optional file-number into arguments.
fn encode_opt_file_handle_arg(opt_file_number_pos: Option<Positioned<FileHandle>>) -> Expressions {
    match opt_file_number_pos {
        Some(file_number_pos) => {
            vec![
                // 1 == present
                Expression::IntegerLiteral(1).at_pos(Position::start()),
                file_number_pos.map(Expression::from),
            ]
        }
        None => {
            // 0 == absent
            vec![Expression::IntegerLiteral(0).at_pos(Position::start())]
        }
    }
}
