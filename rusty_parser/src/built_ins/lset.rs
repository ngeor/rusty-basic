use rusty_common::*;
use rusty_pc::*;

use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::{equal_sign_ws, whitespace};
use crate::{BuiltInSub, ParseError, *};
pub fn parse() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq5(
        keyword(Keyword::LSet),
        whitespace(),
        name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: variable after LSET"),
        equal_sign_ws(),
        expression_pos_p().or_syntax_error("Expected: expression"),
        |_, _, name_pos, _, value_expr_pos| {
            Statement::built_in_sub_call(BuiltInSub::LSet, build_args(name_pos, value_expr_pos))
        },
    )
}

fn build_args(name_pos: NamePos, value_expr_pos: ExpressionPos) -> Expressions {
    let Positioned { element: name, pos } = name_pos;
    let variable_name: String = name.as_bare_name().to_string();
    let name_expr_pos = Expression::Variable(name, VariableInfo::unresolved()).at_pos(pos);
    vec![
        // pass the name of the variable as a special argument
        Expression::StringLiteral(variable_name).at_pos(pos),
        // pass the variable itself (ByRef) to be able to write to it
        name_expr_pos,
        // pass the value to assign to the variable
        value_expr_pos,
    ]
}
