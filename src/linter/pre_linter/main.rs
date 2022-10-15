use crate::common::*;
use crate::linter::pre_linter::can_pre_lint::CanPreLint;
use crate::linter::pre_linter::context::{MainContext, MainContextInner, MainContextWithPos};
use crate::linter::pre_linter::sub_program_context::PostVisit;
use crate::linter::pre_linter::PreLinterResult;
use crate::parser::*;
use std::rc::Rc;

pub fn pre_lint_program(program: &ProgramNode) -> Result<PreLinterResult, QErrorNode> {
    // create a new context (inner mutable, multi-owned)
    let context = Rc::new(MainContext::new());

    // run the CanPreLint trait on the entire program
    program.pre_lint(&context)?;

    // consume the Rc
    let context = match Rc::try_unwrap(context) {
        Ok(c) => c,
        Err(_) => {
            panic!("Should be able to consume the Rc")
        }
    };

    // consume the RefCell
    let MainContextInner {
        functions,
        subs,
        user_defined_types,
        ..
    } = context.into_inner();

    // post visit checks
    functions.post_visit()?;
    subs.post_visit()?;

    Ok(PreLinterResult::new(
        functions.implementations(),
        subs.implementations(),
        user_defined_types,
    ))
}

impl CanPreLint for TopLevelToken {
    type Context = MainContextWithPos;

    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        match self {
            TopLevelToken::DefType(def_type) => def_type.pre_lint(context),
            TopLevelToken::FunctionDeclaration(name, params) => (name, params).pre_lint(context),
            TopLevelToken::FunctionImplementation(f) => f.pre_lint(context),
            TopLevelToken::Statement(s) => s.pre_lint(context),
            TopLevelToken::SubDeclaration(name, params) => (name, params).pre_lint(context),
            TopLevelToken::SubImplementation(s) => s.pre_lint(context),
            TopLevelToken::UserDefinedType(u) => u.pre_lint(context),
        }
    }
}

impl CanPreLint for DefType {
    type Context = MainContextWithPos;

    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        context.as_ref().resolver_mut().set(self);
        Ok(())
    }
}
