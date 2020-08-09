// LINE INPUT -> see INPUT
// LINE INPUT [;] ["prompt";] variable$
// LINE INPUT #file-number%, variable$

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::lexer::{BufLexer, Keyword};
use crate::linter::{Error, ExpressionNode};
use crate::parser::buf_lexer::*;
use crate::parser::sub_call;
use crate::parser::{
    HasQualifier, ParserErrorNode, QualifiedName, Statement, StatementNode, TypeQualifier,
};
use crate::variant::Variant;
use std::io::BufRead;

#[derive(Debug)]
pub struct LineInput {}

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, ParserErrorNode> {
    let Locatable { element: next, pos } = lexer.peek()?;
    if next.is_keyword(Keyword::Line) {
        lexer.read()?;
        read_demand_whitespace(lexer, "Expected space after LINE")?;
        read_demand_keyword(lexer, Keyword::Input)?;
        read_demand_whitespace(lexer, "Expected space after INPUT")?;
        let args = sub_call::read_arg_list(lexer)?;
        Ok(Some(Statement::SubCall("LINE INPUT".into(), args).at(pos)))
    } else {
        Ok(None)
    }
}

impl BuiltInLint for LineInput {
    fn lint(&self, _args: &Vec<ExpressionNode>) -> Result<(), Error> {
        // TODO lint
        Ok(())
    }
}

impl BuiltInRun for LineInput {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
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
                            return Err("Argument type mismatch".to_string()).with_err_no_pos();
                        }
                    }
                    Argument::ByRef(n) => {
                        line_input_one(interpreter, a, n, file_handle)?;
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
    arg: &Argument,
    n: &QualifiedName,
    file_handle: FileHandle,
) -> Result<(), InterpreterErrorNode> {
    if file_handle.is_valid() {
        line_input_one_file(interpreter, arg, n, file_handle)
    } else {
        line_input_one_stdin(interpreter, arg, n)
    }
}

fn line_input_one_file<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    arg: &Argument,
    n: &QualifiedName,
    file_handle: FileHandle,
) -> Result<(), InterpreterErrorNode> {
    let s = interpreter
        .file_manager
        .read_line(file_handle)
        .map_err(|e| e.to_string())
        .with_err_no_pos()?;
    let q: TypeQualifier = n.qualifier();
    match q {
        TypeQualifier::DollarString => interpreter
            .context_mut()
            .demand_sub()
            .set_value_to_popped_arg(arg, Variant::VString(s))
            .with_err_no_pos(),
        _ => unimplemented!(),
    }
}

fn line_input_one_stdin<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    arg: &Argument,
    _n: &QualifiedName,
) -> Result<(), InterpreterErrorNode> {
    let s = interpreter
        .stdlib
        .input()
        .map_err(|e| e.to_string())
        .with_err_no_pos()?;
    interpreter
        .context_mut()
        .demand_sub()
        .set_value_to_popped_arg(arg, Variant::VString(s))
        .with_err_no_pos()
}
