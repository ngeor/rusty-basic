use rusty_common::*;
use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::comma_ws;
use crate::{BuiltInSub, ParseError, *};

/// Example: FIELD #1, 10 AS FirstName$, 20 AS LastName$
pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
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

fn field_item_p() -> impl Parser<RcStringView, Output = (ExpressionPos, NamePos), Error = ParseError>
{
    seq3(
        expr_pos_ws_p(),
        keyword_ws_p(Keyword::As),
        name_p().with_pos().or_expected("variable name"),
        |width, _, name| (width, name),
    )
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
        args.push(Expression::Variable(name, VariableInfo::unresolved()).at_pos(pos));
    }
    args
}
