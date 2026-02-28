use rusty_common::{AtPos, Position, Positioned};
use rusty_pc::*;

use crate::expr::expression_pos_p;
use crate::expr::file_handle::file_handle_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{
    BuiltInSub, Expression, ExpressionPos, Expressions, FileHandle, Keyword, ParserError, Statement,
};

/// Parses built-in subs with optional arguments.
/// Used only by `COLOR` and `LOCATE`.
pub fn parse_built_in_sub_with_opt_args(
    k: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    keyword_ws_p(k)
        .and_keep_right(csv_allow_missing())
        .map(move |opt_args| {
            Statement::built_in_sub_call(built_in_sub, map_opt_args_to_flags(opt_args))
        })
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
pub fn csv_allow_missing()
-> impl Parser<StringView, Output = Vec<Option<ExpressionPos>>, Error = ParserError> {
    expression_pos_p()
        .delimited_by_allow_missing(comma_ws(), trailing_comma_error())
        .or_default()
}

/// Used in `INPUT` and `LINE INPUT`, parsing an optional file number.
pub fn opt_file_handle_comma_p()
-> impl Parser<StringView, Output = Option<Positioned<FileHandle>>, Error = ParserError> {
    seq2(file_handle_p(), comma_ws(), |l, _| l).to_option()
}

/// Used in `INPUT` and `LINE INPUT`, converts an optional file-number into arguments.
pub fn encode_opt_file_handle_arg(
    opt_file_number_pos: Option<Positioned<FileHandle>>,
) -> Expressions {
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
