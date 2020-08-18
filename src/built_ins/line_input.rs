// LINE INPUT -> see INPUT
// LINE INPUT [;] ["prompt";] variable$
// LINE INPUT #file-number%, variable$

use super::{BuiltInLint, BuiltInRun};
use crate::common::pc::*;
use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, Stdlib};
use crate::lexer::*;
use crate::linter::ExpressionNode;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::expression;
use crate::parser::{HasQualifier, QualifiedName, Statement, StatementNode, TypeQualifier};
use crate::variant::Variant;
use std::io::BufRead;

#[derive(Debug)]
pub struct LineInput {}

pub fn take_if_line_input<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    apply(
        |(l, (_, r))| Statement::SubCall("LINE INPUT".into(), r).at(l.pos()),
        with_whitespace_between(
            take_if_keyword(Keyword::Line),
            with_whitespace_between(
                take_if_keyword(Keyword::Input),
                csv(expression::take_if_expression_node()),
            ),
        ),
    )
}

impl BuiltInLint for LineInput {
    fn lint(&self, _args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        // TODO lint
        Ok(())
    }
}

impl BuiltInRun for LineInput {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
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
                            panic!("LINE INPUT linter should have caught this");
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
) -> Result<(), QErrorNode> {
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
) -> Result<(), QErrorNode> {
    let s = interpreter
        .file_manager
        .read_line(file_handle)
        .map_err(|e| e.into())
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
) -> Result<(), QErrorNode> {
    let s = interpreter
        .stdlib
        .input()
        .map_err(|e| e.into())
        .with_err_no_pos()?;
    interpreter
        .context_mut()
        .demand_sub()
        .set_value_to_popped_arg(arg, Variant::VString(s))
        .with_err_no_pos()
}
