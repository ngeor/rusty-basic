pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::base::parsers::Parser;
    use crate::parser::specific::{item_p, keyword_p, whitespace_p};
    use crate::parser::*;

    pub fn parse() -> impl Parser<Output = Statement> {
        keyword_p(Keyword::Field)
            .and_demand(field_node_p().or_syntax_error("Expected: file number after FIELD"))
            .keep_right()
    }

    fn field_node_p() -> impl Parser<Output = Statement> {
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

    fn field_item_p() -> impl Parser<Output = (ExpressionNode, NameNode)> {
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
        // needs to be 1 + N*3 args, N >= 1
        // first is the file number
        // then the fields: width, variable name, variable
        if args.len() < 4 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        if (args.len() - 1) % 3 != 0 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        args.require_integer_argument(0)?;
        let mut i: usize = 1;
        while i < args.len() {
            args.require_integer_argument(i)?;
            args.require_string_argument(i + 1)?;
            args.require_string_variable(i + 2)?;
            i += 3;
        }
        Ok(())
    }
}

pub mod interpreter {
    use crate::common::{FileHandle, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Field;
    use crate::interpreter::utils::VariantCasts;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let len = interpreter.context().variables().len();
        let file_handle: FileHandle = interpreter.context()[0].to_file_handle()?;
        let mut i: usize = 1;
        let mut fields: Vec<Field> = vec![];
        while i < len {
            let width: usize =
                interpreter.context()[i].to_positive_int_or(QError::FieldOverflow)?;
            i += 1;
            // TODO would be great to have a pointer to a variable here, maybe revisit when implementing DEF SEG
            let name: &str = interpreter.context()[i].to_str_unchecked();
            i += 2; // skip over the actual variable
            fields.push(Field {
                width,
                name: name.to_owned(),
            });
        }
        interpreter
            .file_manager()
            .add_field_list(file_handle, fields)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
