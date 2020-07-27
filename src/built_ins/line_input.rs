// LINE INPUT -> see INPUT
// LINE INPUT [;] ["prompt";] variable$
// LINE INPUT #file-number%, variable$

use super::{BuiltInLint, BuiltInParse, BuiltInRun};
use crate::common::{FileHandle, Location};
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{err, Interpreter, InterpreterError, Stdlib};
use crate::lexer::Keyword;
use crate::linter::{Error, ExpressionNode};
use crate::parser::{
    BareNameNode, HasQualifier, Parser, ParserError, QualifiedName, StatementContext,
    StatementNode, TypeQualifier,
};
use crate::variant::Variant;
use std::io::BufRead;

pub struct LineInput {}

impl BuiltInParse for LineInput {
    fn demand<T: BufRead>(
        &self,
        parser: &mut Parser<T>,
        pos: Location,
        context: StatementContext,
    ) -> Result<StatementNode, ParserError> {
        parser.read_demand_whitespace("Expected space after LINE")?;
        parser.read_demand_keyword(Keyword::Input)?;
        parser.read_demand_whitespace("Expected space after INPUT")?;
        let next = parser.buf_lexer.read()?;
        parser.demand_sub_call(BareNameNode::new("LINE INPUT".into(), pos), next, context)
    }
}

impl BuiltInLint for LineInput {
    fn lint(&self, _args: &Vec<ExpressionNode>) -> Result<(), Error> {
        // TODO lint
        Ok(())
    }
}

impl BuiltInRun for LineInput {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        let mut is_first = true;
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_more = true;
        while has_more {
            let arg = &interpreter.pop_unnamed_arg();
            match arg {
                Some(a) => match a {
                    Argument::ByVal(v) => {
                        if is_first && v.qualifier() == TypeQualifier::FileHandle {
                            file_handle = v.clone().demand_file_handle();
                        } else {
                            return err("Argument type mismatch", pos);
                        }
                    }
                    Argument::ByRef(n) => {
                        line_input_one(interpreter, pos, a, n, file_handle)?;
                    }
                },
                None => {
                    has_more = false;
                }
            }

            is_first = false;
        }
        Ok(())
    }
}
fn line_input_one<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    pos: Location,
    arg: &Argument,
    n: &QualifiedName,
    file_handle: FileHandle,
) -> Result<(), InterpreterError> {
    if file_handle.is_valid() {
        line_input_one_file(interpreter, pos, arg, n, file_handle)
    } else {
        line_input_one_stdin(interpreter, pos, arg, n)
    }
}

fn line_input_one_file<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    pos: Location,
    arg: &Argument,
    n: &QualifiedName,
    file_handle: FileHandle,
) -> Result<(), InterpreterError> {
    let s = interpreter
        .file_manager
        .read_line(file_handle)
        .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))?;
    let q: TypeQualifier = n.qualifier();
    match q {
        TypeQualifier::DollarString => interpreter
            .context_mut()
            .demand_sub()
            .set_value_to_popped_arg(arg, Variant::VString(s))
            .map_err(|e| InterpreterError::new_with_pos(e, pos)),
        _ => unimplemented!(),
    }
}

fn line_input_one_stdin<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    pos: Location,
    arg: &Argument,
    _n: &QualifiedName,
) -> Result<(), InterpreterError> {
    let s = interpreter
        .stdlib
        .input()
        .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))?;
    interpreter
        .context_mut()
        .demand_sub()
        .set_value_to_popped_arg(arg, Variant::VString(s))
        .map_err(|e| InterpreterError::new_with_pos(e, pos))
}

// TODO: remove Result aliases, always use Result<T, E>
// TODO: unify errors
