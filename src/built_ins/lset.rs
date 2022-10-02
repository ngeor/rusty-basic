pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::expression::expression_node_p;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        seq5(
            keyword(Keyword::LSet),
            whitespace().no_incomplete(),
            name::name_with_dot_p()
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
        // the parser should produce 3 arguments:
        // the variable name, as a string literal
        // the variable itself, a ByRef string variable
        // a string expression to assign to
        if args.len() != 3 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        args.require_string_argument(0)?;
        // TODO ensure LSET is operating on variables previously used by FIELD in this scope
        args.require_string_variable(1)?;
        args.require_string_argument(2)
    }
}

pub mod interpreter {
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::Variant;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let name: String = interpreter.context()[0].to_str_unchecked().to_owned(); // TODO fighting borrow checker
        let value: Variant = interpreter.context()[2].clone();
        // find which file number is associated with this name and find the width
        // also marks that field index as current for the next PUT operation
        interpreter.file_manager().mark_current_field_list(&name)?;
        interpreter.context_mut()[1] = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
