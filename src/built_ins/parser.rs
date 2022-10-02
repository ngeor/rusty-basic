use crate::built_ins;
use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, FileHandle, Locatable, Location};
use crate::parser::expression::expression_node_p;
use crate::parser::expression::file_handle::file_handle_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::{comma, keyword, trailing_comma_error, whitespace};
use crate::parser::{Expression, ExpressionNode, ExpressionNodes, Keyword, Statement};

/// Parses built-in subs which have a special syntax.
pub fn parse() -> impl Parser<Output = Statement> {
    Alt16::new(
        built_ins::close::parser::parse(),
        built_ins::color::parser::parse(),
        built_ins::data::parser::parse(),
        built_ins::def_seg::parser::parse(),
        built_ins::field::parser::parse(),
        built_ins::get::parser::parse(),
        built_ins::input::parser::parse(),
        built_ins::line_input::parser::parse(),
        built_ins::locate::parser::parse(),
        built_ins::lset::parser::parse(),
        built_ins::name::parser::parse(),
        built_ins::open::parser::parse(),
        built_ins::put::parser::parse(),
        built_ins::read::parser::parse(),
        built_ins::view_print::parser::parse(),
        built_ins::width::parser::parse(),
    )
}

// needed for built-in functions that are also keywords (e.g. LEN), so they
// cannot be parsed by the `word` module.
pub fn built_in_function_call_p() -> impl Parser<Output = Expression> {
    built_ins::len::parser::parse().or(built_ins::string_fn::parser::parse())
}

/// Parses built-in subs with optional arguments.
/// Used only by `COLOR` and `LOCATE`.
pub fn parse_built_in_sub_with_opt_args(
    k: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<Output = Statement> {
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
fn map_opt_args_to_flags(args: Vec<Option<ExpressionNode>>) -> ExpressionNodes {
    let mut result: ExpressionNodes = vec![];
    let mut mask = 1;
    let mut flags = 0;
    for arg in args {
        if let Some(arg) = arg {
            flags |= mask;
            result.push(arg);
        }
        mask <<= 1;
    }
    result.insert(0, Expression::IntegerLiteral(flags).at(Location::start()));
    result
}

/// Comma separated list of items, allowing items to be missing between commas.
pub fn csv_allow_missing() -> impl Parser<Output = Vec<Option<ExpressionNode>>> + NonOptParser {
    parse_delimited_to_items(
        opt_zip(expression_node_p(), comma()),
        true,
        trailing_comma_error(),
    )
    .allow_default()
}

/// Used in `INPUT` and `LINE INPUT`, parsing an optional file number.
pub fn opt_file_handle_comma_p(
) -> impl Parser<Output = Option<Locatable<FileHandle>>> + NonOptParser {
    seq2(file_handle_p(), comma().no_incomplete(), |l, _| l).allow_none()
}

/// Used in `INPUT` and `LINE INPUT`, converts an optional file-number into arguments.
pub fn encode_opt_file_handle_arg(
    opt_file_number_node: Option<Locatable<FileHandle>>,
) -> ExpressionNodes {
    match opt_file_number_node {
        Some(file_number_node) => {
            vec![
                // 1 == present
                Expression::IntegerLiteral(1).at(Location::start()),
                file_number_node.map(Expression::from),
            ]
        }
        None => {
            // 0 == absent
            vec![Expression::IntegerLiteral(0).at(Location::start())]
        }
    }
}
