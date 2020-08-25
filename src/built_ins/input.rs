use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::{Expression, ExpressionNode};
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::{HasQualifier, Keyword, QualifiedName, Statement, TypeQualifier};
use crate::variant::Variant;
use std::io::BufRead;

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

#[derive(Debug)]
pub struct Input {}

pub fn parse_input<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map(
        with_keyword_before(
            Keyword::Input,
            csv_one_or_more(expression::expression_node(), || {
                QError::SyntaxError("Expected at least one variable".to_string())
            }),
        ),
        |r| Statement::SubCall("INPUT".into(), r),
    )
}

impl BuiltInLint for Input {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.len() == 0 {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        } else {
            args.iter()
                .map(|a| match a.as_ref() {
                    Expression::Variable(_) => Ok(()),
                    _ => Err(QError::VariableRequired).with_err_at(a),
                })
                .collect()
        }
    }
}

impl BuiltInRun for Input {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
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
) -> Result<(), QErrorNode> {
    let raw_input: String = interpreter
        .stdlib
        .input()
        .map_err(|e| e.into())
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

fn parse_single_input(s: String) -> Result<f32, QError> {
    if s.is_empty() {
        Ok(0.0)
    } else {
        s.parse::<f32>()
            .map_err(|e| format!("Could not parse {} as float: {}", s, e).into())
    }
}

fn parse_int_input(s: String) -> Result<i32, QError> {
    if s.is_empty() {
        Ok(0)
    } else {
        s.parse::<i32>()
            .map_err(|e| format!("Could not parse {} as int: {}", s, e).into())
    }
}
