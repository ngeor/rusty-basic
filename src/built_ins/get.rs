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
        if args.len() != 2 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }
        args.require_integer_argument(0)?;
        args.require_long_argument(1)
    }
}

pub mod interpreter {
    use crate::common::{FileHandle, QError};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::io::Field;
    use crate::interpreter::utils::{get_record_number, to_file_handle};
    use crate::parser::{BareName, TypeQualifier};
    use crate::variant::Variant;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let handle: FileHandle = to_file_handle(&interpreter.context()[0])?;
        let record_number: usize = get_record_number(&interpreter.context()[1])?;
        let file_info = interpreter.file_manager().try_get_file_info_mut(&handle)?;
        let field_lists: Vec<Vec<Field>> = file_info.get_field_lists().clone(); // TODO fighting the borrow checker
        let bytes = file_info.get_record(record_number)?;
        for fields in field_lists {
            let mut start: usize = 0;
            for Field { width, name } in fields {
                // collect ASCII chars stop at non printable char
                let s = from_ascii(&bytes[start..(start + width)]);
                let v = Variant::VString(s);
                // set variable in parent context, because we're inside the context of the built-in sub
                let bare_name: BareName = BareName::from(name);
                interpreter
                    .context_mut()
                    .caller_variables_mut()
                    .insert_built_in(bare_name, TypeQualifier::DollarString, v);
                // shift to next offset
                start += width;
            }
        }
        Ok(())
    }

    fn from_ascii(bytes: &[u8]) -> String {
        let mut s = String::new();
        for byte in bytes {
            let ch = *byte as char;
            if ch >= ' ' {
                s.push(ch);
            } else {
                break;
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {}
