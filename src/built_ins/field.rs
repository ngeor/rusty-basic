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
        keyword_p(Keyword::Field)
            .and_demand(field_node_p().or_syntax_error("Expected: file number after FIELD"))
            .keep_right()
    }

    fn field_node_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        whitespace_p()
            .and_demand(expression::file_handle_p().or_syntax_error("Expected: file-number"))
            .and_demand(
                item_p(',')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ,"),
            )
            .and_demand(
                field_item_p()
                    .csv()
                    .or_syntax_error("Expected: field width"),
            )
            .map(|(((_, file_number), _), fields)| {
                Statement::BuiltInSubCall(BuiltInSub::Field, build_args(file_number, fields))
            })
    }

    fn field_item_p<R>() -> impl Parser<R, Output = (ExpressionNode, NameNode)>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        expression::expression_node_p()
            // TODO 'AS' does not need leading whitespace if expression has parenthesis
            // TODO solve this not by peeking the previous but with a new expression:: function
            .and_demand(
                keyword_p(Keyword::As)
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: AS"),
            )
            .and_demand(
                name::name_with_dot_p()
                    .with_pos()
                    .or_syntax_error("Expected: variable name"),
            )
            .map(|((width, _), name)| (width, name))
    }

    fn build_args(
        file_number: Locatable<FileHandle>,
        fields: Vec<(ExpressionNode, NameNode)>,
    ) -> ExpressionNodes {
        let mut args: ExpressionNodes = vec![];
        args.push(file_number.map(|x| Expression::IntegerLiteral(x.into())));
        for (width, Locatable { element: name, pos }) in fields {
            args.push(width);
            let variable_name: String = name.bare_name().as_ref().to_string();
            args.push(Expression::StringLiteral(variable_name).at(pos));
            // to lint the variable, not used at runtime
            args.push(Expression::Variable(name, VariableInfo::unresolved()).at(pos));
        }
        args
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
