use crate::built_ins::built_in_sub::BuiltInSub;
use crate::expression::expr_pos_ws_p;
use crate::expression::file_handle::file_handle_p;
use crate::name::name_with_dots;
use crate::pc::*;
use crate::pc_specific::*;
use crate::specific::*;
use rusty_common::*;

/// Example: FIELD #1, 10 AS FirstName$, 20 AS LastName$
pub fn parse() -> impl Parser<RcStringView, Output = Statement> {
    seq5(
        keyword(Keyword::Field),
        // TODO: create a shortcut for whitespace().no_incomplete() and simple token parsers in general
        // they should have a different FilterParser implementation that does not require Undo capability
        whitespace(),
        file_handle_p().or_syntax_error("Expected: file-number"),
        comma(),
        csv_non_opt(field_item_p(), "Expected: field width"),
        |_, _, file_number, _, fields| {
            Statement::BuiltInSubCall(BuiltInSub::Field, build_args(file_number, fields))
        },
    )
}

fn field_item_p() -> impl Parser<RcStringView, Output = (ExpressionPos, NamePos)> {
    seq4(
        expr_pos_ws_p(),
        keyword(Keyword::As),
        whitespace(),
        name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: variable name"),
        |width, _, _, name| (width, name),
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
        let variable_name: String = name.bare_name().to_string();
        args.push(Expression::StringLiteral(variable_name).at_pos(pos));
        // to lint the variable, not used at runtime
        args.push(Expression::Variable(name, VariableInfo::unresolved()).at_pos(pos));
    }
    args
}
