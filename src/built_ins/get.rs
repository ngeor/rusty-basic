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
        keyword_followed_by_whitespace_p(Keyword::Get)
            .and_demand(expression::file_handle_p().or_syntax_error("Expected: file-number"))
            .and_demand(
                item_p(',')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ,"),
            )
            .and_demand(expression::expression_node_p().or_syntax_error("Expected: record-number"))
            .map(|(((_, file_number), _), r)| {
                Statement::BuiltInSubCall(
                    BuiltInSub::Get,
                    vec![file_number.map(|x| Expression::IntegerLiteral(x.into())), r],
                )
            })
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
