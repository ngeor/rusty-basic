use rusty_common::{AtPos, Position, Positioned};
use rusty_parser::{
    DimVar, FunctionImplementation, GlobalStatement, GlobalStatementPos, Program, Statement,
    Statements, SubImplementation,
};

use crate::converter::common::{Convertible, ConvertibleIn};
use crate::core::{IntoQualified, LintErrorPos, LinterContext, ScopeName};
use crate::names::ImplicitVars;

impl Convertible for Program {
    fn convert(self, ctx: &mut LinterContext) -> Result<Self, LintErrorPos> {
        // collect the global statements
        let mut global_statements: Self = vec![];
        for Positioned { element, pos } in self {
            let expanded_statements_for_element = element.convert_in(ctx, pos)?;
            global_statements.extend(expanded_statements_for_element);
        }

        // collect implicitly defined variables
        let mut implicit_vars = ImplicitVars::new();
        implicit_vars.append(ctx.names.get_implicit_vars_mut());
        let mut implicit_statements: Self = implicit_vars
            .into_iter()
            .map(|Positioned { element, pos }| {
                GlobalStatement::Statement(Statement::Dim(DimVar::from(element).into_list(pos)))
                    .at_pos(pos)
            })
            .collect();

        // insert them at the top of the program
        implicit_statements.append(&mut global_statements);
        Ok(implicit_statements)
    }
}

impl ConvertibleIn<Position, Vec<GlobalStatementPos>> for GlobalStatement {
    fn convert_in(
        self,
        ctx: &mut LinterContext,
        pos: Position,
    ) -> Result<Vec<GlobalStatementPos>, LintErrorPos> {
        match self {
            Self::DefType(def_type) => {
                ctx.resolver.set(&def_type);
                Ok(vec![])
            }
            Self::FunctionDeclaration(_) | Self::SubDeclaration(_) | Self::UserDefinedType(_) => {
                Ok(vec![])
            }
            Self::FunctionImplementation(f) => on_function_implementation(f, ctx)
                .map(|f| vec![Self::FunctionImplementation(f).at_pos(pos)]),
            Self::SubImplementation(s) => {
                on_sub_implementation(s, ctx).map(|s| vec![Self::SubImplementation(s).at_pos(pos)])
            }
            Self::Statement(s) => on_statement(s, ctx, pos),
        }
    }
}

fn on_function_implementation(
    function_implementation: FunctionImplementation,
    ctx: &mut LinterContext,
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

    // resolve the function's qualified name
    let resolved_function_name = unresolved_function_name.to_qualified(&ctx.resolver);

    // push a new naming scope for the FUNCTION
    ctx.names.push(ScopeName::Function(
        resolved_function_name.clone().demand_qualified(),
    ));

    // convert the function parameters
    let params = params.convert(ctx)?;

    let mapped = FunctionImplementation {
        name: resolved_function_name.at_pos(pos),
        params,
        body: convert_block_hoisting_implicit_vars_and_pop_scope(body, ctx)?,
        is_static,
    };
    Ok(mapped)
}

fn on_sub_implementation(
    sub_implementation: SubImplementation,
    ctx: &mut LinterContext,
) -> Result<SubImplementation, LintErrorPos> {
    let SubImplementation {
        name,
        params,
        body,
        is_static,
    } = sub_implementation;

    // push a new naming scope for the SUB
    ctx.names.push(ScopeName::Sub(name.element.clone()));

    // convert the parameters
    let params = params.convert(ctx)?;
    let mapped = SubImplementation {
        name,
        params,
        body: convert_block_hoisting_implicit_vars_and_pop_scope(body, ctx)?,
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
fn convert_block_hoisting_implicit_vars_and_pop_scope(
    statements: Statements,
    ctx: &mut LinterContext,
) -> Result<Statements, LintErrorPos> {
    let mut result = statements.convert(ctx)?;
    let implicit_vars = collect_implicit_vars_and_pop_scope(ctx);
    let mut implicit_dim: Statements = implicit_vars
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
    ctx: &mut LinterContext,
    pos: Position,
) -> Result<Vec<GlobalStatementPos>, LintErrorPos> {
    // a statement might be converted into multiple statements due to implicit vars
    let statements = vec![statement.at_pos(pos)];
    let statements: Statements = statements.convert(ctx)?;
    Ok(statements
        .into_iter()
        .map(|statement_pos| statement_pos.map(GlobalStatement::Statement))
        .collect())
}

fn collect_implicit_vars_and_pop_scope(ctx: &mut LinterContext) -> ImplicitVars {
    // collect implicit vars
    let mut implicit_vars = ImplicitVars::new();
    implicit_vars.append(ctx.names.get_implicit_vars_mut());
    // restore the global scope name
    ctx.names.pop();
    implicit_vars
}
