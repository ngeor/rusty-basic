use crate::parser::expression::expression_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;
use rusty_common::*;

pub fn parse() -> impl Parser<Output = Statement> {
    seq5(
        keyword(Keyword::LSet),
        whitespace().no_incomplete(),
        name::name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: variable after LSET"),
        equal_sign().no_incomplete(),
        expression_node_p().or_syntax_error("Expected: expression"),
        |_, _, name_node, _, value_expr_node| {
            Statement::BuiltInSubCall(BuiltInSub::LSet, build_args(name_node, value_expr_node))
        },
    )
}

fn build_args(name_node: NameNode, value_expr_node: ExpressionNode) -> ExpressionNodes {
    let Locatable { element: name, pos } = name_node;
    let variable_name: String = name.bare_name().to_string();
    let name_expr_node = Expression::Variable(name, VariableInfo::unresolved()).at(pos);
    vec![
        // pass the name of the variable as a special argument
        Expression::StringLiteral(variable_name).at(pos),
        // pass the variable itself (ByRef) to be able to write to it
        name_expr_node,
        // pass the value to assign to the variable
        value_expr_node,
    ]
}
