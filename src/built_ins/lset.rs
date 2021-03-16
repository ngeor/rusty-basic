pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_followed_by_whitespace_p(Keyword::LSet)
            .and_demand(
                name::name_with_dot_p()
                    .with_pos()
                    .or_syntax_error("Expected: variable after LSET"),
            )
            .and_demand(
                item_p('=')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ="),
            )
            .and_demand(expression::expression_node_p().or_syntax_error("Expected: expression"))
            .map(|(((_, name_node), _), value_expr_node)| {
                Statement::BuiltInSubCall(BuiltInSub::LSet, build_args(name_node, value_expr_node))
            })
    }

    fn build_args(name_node: NameNode, value_expr_node: ExpressionNode) -> ExpressionNodes {
        let Locatable { element: name, pos } = name_node;
        let variable_name: String = name.bare_name().as_ref().to_owned();
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
}

pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        Ok(())
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
