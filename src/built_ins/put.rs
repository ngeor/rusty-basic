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
        keyword_followed_by_whitespace_p(Keyword::Put)
            .and_demand(expression::file_handle_p().or_syntax_error("Expected: file-number"))
            .and_demand(
                item_p(',')
                    .surrounded_by_opt_ws()
                    .or_syntax_error("Expected: ,"),
            )
            .and_demand(expression::expression_node_p().or_syntax_error("Expected: record-number"))
            .map(|(((_, file_number), _), r)| {
                Statement::BuiltInSubCall(
                    BuiltInSub::Put,
                    vec![file_number.map(|x| Expression::IntegerLiteral(x.into())), r],
                )
            })
    }
}

pub mod linter {
    use crate::common::QErrorNode;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        super::super::get::linter::lint(args)
    }
}

pub mod interpreter {
    use crate::common::{FileHandle, QError, StringUtils};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Field;
    use crate::interpreter::utils::{get_record_number, to_file_handle};
    use crate::parser::{BareName, TypeQualifier};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let handle: FileHandle = to_file_handle(&interpreter.context()[0])?;
        let record_number: usize = get_record_number(&interpreter.context()[1])?;
        let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
        let mut record_contents = String::new();
        // get the current field list
        let field_list = file_info
            .get_current_field_list()
            .ok_or(QError::BadFileMode)?
            .clone(); // TODO fighting the borrow checker
                      // convert the variables into a string
        for Field { width, name } in field_list {
            let bare_name: BareName = BareName::from(name.as_str());
            let v = interpreter
                .context()
                .caller_variables()
                .get_built_in(&bare_name, TypeQualifier::DollarString)
                .ok_or(QError::VariableRequired)?;
            let s = v.to_str_unchecked().fix_length_with_char(width, '\0');
            record_contents.push_str(s.as_str());
        }
        let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
        file_info.put_record(record_number, record_contents.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}