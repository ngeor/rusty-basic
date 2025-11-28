use crate::expression::expr_pos_ws_p;
use crate::expression::file_handle::file_handle_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::*;
use rusty_common::*;

/// Example: FIELD #1, 10 AS FirstName$, 20 AS LastName$
pub fn parse<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    seq5(
        keyword(Keyword::Field),
        whitespace().no_incomplete(),
        file_handle_p().or_syntax_error("Expected: file-number"),
        comma().no_incomplete(),
        csv_non_opt(field_item_p(), "Expected: field width"),
        |_, _, file_number, _, fields| {
            Statement::BuiltInSubCall(BuiltInSub::Field, build_args(file_number, fields))
        },
    )
}

fn field_item_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = (ExpressionPos, NamePos)> {
    seq4(
        expr_pos_ws_p(),
        keyword(Keyword::As).no_incomplete(),
        whitespace().no_incomplete(),
        name::name_with_dots()
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
