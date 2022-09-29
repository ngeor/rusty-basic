pub mod parser {
    use crate::built_ins::BuiltInSub;
    use crate::parser::pc::*;
    use crate::parser::pc_specific::*;
    use crate::parser::*;

    pub fn parse() -> impl OptParser<Output = Statement> {
        seq4(
            keyword_followed_by_whitespace_p(Keyword::Put),
            expression::file_handle_p().or_syntax_error("Expected: file-number"),
            comma_surrounded_by_opt_ws(),
            expression::expression_node_p().or_syntax_error("Expected: record-number"),
            |_, file_number, _, r| {
                Statement::BuiltInSubCall(
                    BuiltInSub::Put,
                    vec![file_number.map(|x| Expression::IntegerLiteral(x.into())), r],
                )
            },
        )
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
    use crate::common::{FileHandle, QError, ToAsciiBytes};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Field;
    use crate::interpreter::utils::VariantCasts;
    use crate::parser::{BareName, TypeQualifier};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let handle: FileHandle = interpreter.context()[0].to_file_handle()?;
        let record_number: usize = interpreter.context()[1].to_record_number()?;
        let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
        let mut record_contents: Vec<u8> = vec![];
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
            let mut bytes: Vec<u8> = v.to_str_unchecked().to_ascii_bytes();
            fix_length(&mut bytes, width);
            record_contents.append(&mut bytes);
        }
        let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
        file_info.put_record(record_number, &record_contents)?;
        Ok(())
    }

    fn fix_length(bytes: &mut Vec<u8>, width: usize) {
        while bytes.len() < width {
            bytes.push(0);
        }
        while bytes.len() > width {
            bytes.pop();
        }
    }
}
