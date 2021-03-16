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
        keyword_p(Keyword::Name)
            .and_demand(
                expression::back_guarded_expression_node_p()
                    .or_syntax_error("Expected: old file name"),
            )
            .and_demand(keyword_p(Keyword::As).or_syntax_error("Expected: AS"))
            .keep_middle()
            .and_demand(
                expression::guarded_expression_node_p().or_syntax_error("Expected: new file name"),
            )
            .map(|(l, r)| Statement::BuiltInSubCall(BuiltInSub::Name, vec![l, r]))
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
