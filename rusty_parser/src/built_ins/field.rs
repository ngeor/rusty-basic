use rusty_common::*;
use rusty_pc::*;

use crate::expr::expr_ws_keyword_p;
use crate::expr::file_handle::file_handle_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{BuiltInSub, ParserError, *};

/// Example: FIELD #1, 10 AS FirstName$, 20 AS LastName$
pub fn parse() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq4(
        keyword_ws_p(Keyword::Field),
        file_handle_p().or_expected("file-number"),
        comma_ws(),
        csv_non_opt(field_item_p(), "field width"),
        |_, file_number, _, fields| {
            Statement::built_in_sub_call(BuiltInSub::Field, build_args(file_number, fields))
        },
    )
}

fn field_item_p() -> impl Parser<StringView, Output = (ExpressionPos, NamePos), Error = ParserError>
{
    expr_ws_keyword_p(Keyword::As).and_tuple(demand_lead_ws(
        name_p().with_pos().or_expected("variable name"),
    ))
}

fn build_args(
    file_number_pos: Positioned<FileHandle>,
    fields: Vec<(ExpressionPos, NamePos)>,
) -> Expressions {
    let mut args: Expressions = vec![];
    args.push(file_number_pos.map(Expression::from));
    for (width, Positioned { element: name, pos }) in fields {
        args.push(width);
        let variable_name: String = name.as_bare_name().to_string();
        args.push(Expression::StringLiteral(variable_name).at_pos(pos));
        // to lint the variable, not used at runtime
        args.push(Expression::Variable(name, ExpressionType::Unresolved).at_pos(pos));
    }
    args
}
