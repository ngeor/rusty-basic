use crate::converter::context::Context;
use crate::converter::pos_context::PosContext;
use crate::converter::traits::Convertible;
use crate::converter::types::Implicits;
use crate::error::LintErrorPos;
use rusty_common::{AtPos, HasPos, Positioned};
use rusty_parser::{
    DimVar, FunctionImplementation, GlobalStatement, GlobalStatementPos, Program, Statement,
    Statements, SubImplementation,
};

impl Convertible for Program {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let mut result: Program = vec![];
        for Positioned { element, pos } in self {
            let global_statements = element.convert_in(ctx, pos)?;
            result.extend(global_statements.into_iter());
        }

        // insert implicits at the top
        let mut implicits = Implicits::new();
        implicits.append(ctx.names.get_implicits());
        let mut implicit_statements: Program = implicits
            .into_iter()
            .map(|Positioned { element, pos }| {
                GlobalStatement::Statement(Statement::Dim(DimVar::from(element).into_list(pos)))
                    .at_pos(pos)
            })
            .collect();
        implicit_statements.append(&mut result);
        Ok(implicit_statements)
    }
}

impl<'a> Convertible<PosContext<'a>, Vec<GlobalStatementPos>> for GlobalStatement {
    fn convert(self, ctx: &mut PosContext<'a>) -> Result<Vec<GlobalStatementPos>, LintErrorPos> {
        match self {
            Self::DefType(def_type) => {
                ctx.resolver.set(&def_type);
                Ok(vec![])
            }
            Self::FunctionDeclaration(_, _)
            | Self::SubDeclaration(_, _)
            | Self::UserDefinedType(_) => Ok(vec![]),
            Self::FunctionImplementation(f) => on_function_implementation(f, ctx)
                .map(|f| vec![Self::FunctionImplementation(f).at_pos(ctx.pos())]),
            Self::SubImplementation(s) => on_sub_implementation(s, ctx)
                .map(|s| vec![Self::SubImplementation(s).at_pos(ctx.pos())]),
            Self::Statement(s) => on_statement(s, ctx),
        }
    }
}

fn on_function_implementation(
    function_implementation: FunctionImplementation,
    ctx: &mut PosContext,
) -> Result<FunctionImplementation, LintErrorPos> {
    let FunctionImplementation {
        name: Positioned {
            element: unresolved_function_name,
            pos,
        },
        params,
        body,
        is_static,
    } = function_implementation;
    let (resolved_function_name, resolved_params) =
        ctx.push_function_context(unresolved_function_name, params)?;
    let mapped = FunctionImplementation {
        name: resolved_function_name.at_pos(pos),
        params: resolved_params,
        body: convert_block_hoisting_implicits(body, ctx)?,
        is_static,
    };
    Ok(mapped)
}

fn on_sub_implementation(
    sub_implementation: SubImplementation,
    ctx: &mut PosContext,
) -> Result<SubImplementation, LintErrorPos> {
    let SubImplementation {
        name,
        params,
        body,
        is_static,
    } = sub_implementation;
    let mapped_params = ctx.push_sub_context(params)?;
    let mapped = SubImplementation {
        name,
        params: mapped_params,
        body: convert_block_hoisting_implicits(body, ctx)?,
        is_static,
    };
    Ok(mapped)
}

// A statement can be expanded into multiple statements to convert implicitly
// declared variables into explicit.
// Example:
//      A = B + C
// becomes
//      DIM B
//      DIM C
//      DIM A
//      A = B + C
fn convert_block_hoisting_implicits(
    statements: Statements,
    ctx: &mut Context,
) -> Result<Statements, LintErrorPos> {
    let mut result = statements.convert(ctx)?;
    let implicits = ctx.pop_context();
    let mut implicit_dim: Statements = implicits
        .into_iter()
        .map(
            |Positioned {
                 element: q_name,
                 pos,
             }| Statement::Dim(DimVar::from(q_name).into_list(pos)).at_pos(pos),
        )
        .collect();

    implicit_dim.append(&mut result);
    Ok(implicit_dim)
}

fn on_statement(
    statement: Statement,
    ctx: &mut PosContext,
) -> Result<Vec<GlobalStatementPos>, LintErrorPos> {
    // a statement might be converted into multiple statements due to implicits
    let statements = vec![statement.at_pos(ctx.pos())];
    let statements = statements.convert(ctx)?;
    Ok(statements
        .into_iter()
        .map(|statement_pos| statement_pos.map(GlobalStatement::Statement))
        .collect())
}
