
use crate::common::*;
use crate::parser::expression::expr_node_ws;
use crate::parser::expression::file_handle::file_handle_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::*;

/// Example: FIELD #1, 10 AS FirstName$, 20 AS LastName$
pub fn parse() -> impl Parser<Output = Statement> {
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

fn field_item_p() -> impl Parser<Output = (ExpressionNode, NameNode)> {
    seq4(
        expr_node_ws(),
        keyword(Keyword::As).no_incomplete(),
        whitespace().no_incomplete(),
        name::name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: variable name"),
        |width, _, _, name| (width, name),
    )
}

fn build_args(
    file_number_node: Locatable<FileHandle>,
    fields: Vec<(ExpressionNode, NameNode)>,
) -> ExpressionNodes {
    let mut args: ExpressionNodes = vec![];
    args.push(file_number_node.map(Expression::from));
    for (width, Locatable { element: name, pos }) in fields {
        args.push(width);
        let variable_name: String = name.bare_name().to_string();
        args.push(Expression::StringLiteral(variable_name).at(pos));
        // to lint the variable, not used at runtime
        args.push(Expression::Variable(name, VariableInfo::unresolved()).at(pos));
    }
    args
}
