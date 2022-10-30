use crate::converter::context::Context;
use crate::converter::pos_context::PosContext;
use crate::converter::traits::Convertible;
use crate::converter::types::Implicits;
use rusty_common::{AtLocation, HasLocation, Locatable, QErrorNode};
use rusty_parser::{
    DimName, FunctionImplementation, ProgramNode, Statement, StatementNodes, SubImplementation,
    TopLevelToken, TopLevelTokenNode,
};

impl Convertible for ProgramNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let mut result: ProgramNode = vec![];
        for Locatable { element, pos } in self {
            let top_level_vec = element.convert_in(ctx, pos)?;
            result.extend(top_level_vec.into_iter());
        }

        // insert implicits at the top
        let mut implicits = Implicits::new();
        implicits.append(ctx.names.get_implicits());
        let mut implicit_statements: ProgramNode = implicits
            .into_iter()
            .map(|Locatable { element, pos }| {
                TopLevelToken::Statement(Statement::Dim(DimName::from(element).into_list(pos)))
                    .at(pos)
            })
            .collect();
        implicit_statements.append(&mut result);
        Ok(implicit_statements)
    }
}

impl<'a> Convertible<PosContext<'a>, Vec<TopLevelTokenNode>> for TopLevelToken {
    fn convert(self, ctx: &mut PosContext<'a>) -> Result<Vec<TopLevelTokenNode>, QErrorNode> {
        match self {
            Self::DefType(def_type) => {
                ctx.resolver.set(&def_type);
                Ok(vec![])
            }
            Self::FunctionDeclaration(_, _)
            | Self::SubDeclaration(_, _)
            | Self::UserDefinedType(_) => Ok(vec![]),
            Self::FunctionImplementation(f) => on_function_implementation(f, ctx)
                .map(|f| vec![Self::FunctionImplementation(f).at(ctx.pos())]),
            Self::SubImplementation(s) => on_sub_implementation(s, ctx)
                .map(|s| vec![Self::SubImplementation(s).at(ctx.pos())]),
            Self::Statement(s) => on_statement(s, ctx),
        }
    }
}

fn on_function_implementation(
    function_implementation: FunctionImplementation,
    ctx: &mut PosContext,
) -> Result<FunctionImplementation, QErrorNode> {
    let FunctionImplementation {
        name: Locatable {
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
        name: resolved_function_name.at(pos),
        params: resolved_params,
        body: convert_block_hoisting_implicits(body, ctx)?,
        is_static,
    };
    Ok(mapped)
}

fn on_sub_implementation(
    sub_implementation: SubImplementation,
    ctx: &mut PosContext,
) -> Result<SubImplementation, QErrorNode> {
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
    statements: StatementNodes,
    ctx: &mut Context,
) -> Result<StatementNodes, QErrorNode> {
    let mut result = statements.convert(ctx)?;
    let implicits = ctx.pop_context();
    let mut implicit_dim: StatementNodes = implicits
        .into_iter()
        .map(
            |Locatable {
                 element: q_name,
                 pos,
             }| Statement::Dim(DimName::from(q_name).into_list(pos)).at(pos),
        )
        .collect();

    implicit_dim.append(&mut result);
    Ok(implicit_dim)
}

fn on_statement(
    statement: Statement,
    ctx: &mut PosContext,
) -> Result<Vec<TopLevelTokenNode>, QErrorNode> {
    // a statement might be converted into multiple statements due to implicits
    let statements = vec![statement.at(ctx.pos())];
    let statements = statements.convert(ctx)?;
    Ok(statements
        .into_iter()
        .map(|statement_node| statement_node.map(TopLevelToken::Statement))
        .collect())
}
