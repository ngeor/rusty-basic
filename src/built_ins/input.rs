// INPUT [;] ["prompt"{; | ,}] variable-list
// INPUT #file-number%, variable-list
//
// prompt - An optional string that is displayed before the user
// enters data. A semicolon after promp appends a question mark to the
// prompt string.
//
// variable names can consist of up to 40 characters and must begin
// with a letter. Valid characters are a-z, 0-9 and period (.).
// TODO support periods in variable names
// TODO enforce 40 character limit (with error: Identifier too long)
//
// A semicolon immediately after INPUT keeps the cursor on the same line
// after the user presses the Enter key.

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::lexer::*;
use crate::linter::{Error, Expression, ExpressionNode, LinterError};
use crate::parser::buf_lexer::*;
use crate::parser::sub_call;
use crate::parser::{
    HasQualifier, ParserErrorNode, QualifiedName, Statement, StatementNode, TypeQualifier,
};
use crate::variant::Variant;
use std::io::BufRead;

#[derive(Debug)]
pub struct Input {}

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, ParserErrorNode> {
    let Locatable { element: next, pos } = lexer.peek()?;
    if next.is_keyword(Keyword::Input) {
        lexer.read()?;
        read_demand_whitespace(lexer, "Expected space after INPUT")?;
        let args = sub_call::read_arg_list(lexer)?;
        Ok(Some(Statement::SubCall("INPUT".into(), args).at(pos)))
    } else {
        Ok(None)
    }
}

impl BuiltInLint for Input {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() == 0 {
            Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            args.iter()
                .map(|a| match a.as_ref() {
                    Expression::Variable(_) => Ok(()),
                    _ => Err(LinterError::VariableRequired).with_err_at(a),
                })
                .collect()
        }
    }
}

impl BuiltInRun for Input {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        loop {
            match &interpreter.pop_unnamed_arg() {
                Some(a) => match a {
                    Argument::ByRef(n) => {
                        do_input_one_var(interpreter, a, n)?;
                    }
                    _ => {
                        panic!("Expected variable (linter should have caught this)");
                    }
                },
                None => {
                    break;
                }
            }
        }
        Ok(())
    }
}

fn do_input_one_var<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    a: &Argument,
    n: &QualifiedName,
) -> Result<(), InterpreterErrorNode> {
    let raw_input: String = interpreter
        .stdlib
        .input()
        .map_err(|e| e.to_string())
        .with_err_no_pos()?;
    let q: TypeQualifier = n.qualifier();
    let variable_value = match q {
        TypeQualifier::BangSingle => {
            Variant::from(parse_single_input(raw_input).with_err_no_pos()?)
        }
        TypeQualifier::DollarString => Variant::from(raw_input),
        TypeQualifier::PercentInteger => {
            Variant::from(parse_int_input(raw_input).with_err_no_pos()?)
        }
        _ => unimplemented!(),
    };
    interpreter
        .context_mut()
        .demand_sub()
        .set_value_to_popped_arg(a, variable_value)
        .with_err_no_pos()
}

fn parse_single_input(s: String) -> Result<f32, String> {
    if s.is_empty() {
        Ok(0.0)
    } else {
        s.parse::<f32>()
            .map_err(|e| format!("Could not parse {} as float: {}", s, e))
    }
}

fn parse_int_input(s: String) -> Result<i32, String> {
    if s.is_empty() {
        Ok(0)
    } else {
        s.parse::<i32>()
            .map_err(|e| format!("Could not parse {} as int: {}", s, e))
    }
}
