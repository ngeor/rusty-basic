use super::{dim, redim, validation};
use crate::common::*;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::Convertible;
use crate::linter::DimContext;
use crate::parser::*;
use crate::variant::{QBNumberCast, MAX_INTEGER};

pub fn on_dim(
    ctx: &mut Context,
    dim_list: DimList,
    dim_context: DimContext,
) -> Result<DimList, QErrorNode> {
    let DimList { variables, shared } = dim_list;
    let variables: DimNameNodes = variables
        .into_iter()
        .map(
            |Locatable {
                 element:
                     DimName {
                         bare_name,
                         var_type: dim_type,
                     },
                 pos,
             }| { convert(ctx, bare_name, dim_type, dim_context, shared, pos) },
        )
        .collect::<Result<DimNameNodes, QErrorNode>>()?;
    Ok(DimList { variables, shared })
}

impl Convertible for ArrayDimension {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(Self {
            lbound: self.lbound.convert_in_default(ctx)?,
            ubound: self.ubound.convert_in_default(ctx)?,
        })
    }
}

fn convert(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> Result<DimNameNode, QErrorNode> {
    validation::validate(ctx, &bare_name, &dim_type, dim_context, shared).patch_err_pos(pos)?;
    do_convert(ctx, bare_name, dim_type, dim_context, shared, pos)
}

fn do_convert(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> Result<DimNameNode, QErrorNode> {
    match dim_context {
        DimContext::Default | DimContext::Param => {
            dim::convert(ctx, bare_name, dim_type, dim_context, shared, pos)
        }
        DimContext::Redim => redim::convert(ctx, bare_name, dim_type, shared, pos),
    }
}

pub fn resolve_string_length(
    ctx: &Context,
    length_expression: &ExpressionNode,
) -> Result<u16, QErrorNode> {
    let v = ctx.names.resolve_const(length_expression)?;
    let i: i32 = v.try_cast().with_err_at(length_expression)?;
    if i >= 1 && i < MAX_INTEGER {
        Ok(i as u16)
    } else {
        Err(QError::OutOfStringSpace).with_err_at(length_expression)
    }
}

pub fn on_params(ctx: &mut Context, params: ParamNameNodes) -> Result<ParamNameNodes, QErrorNode> {
    params
        .into_iter()
        .map(|x| convert_param_name_node(ctx, x))
        .collect()
}

// TODO remove the dance between params and dim nodes
fn convert_param_name_node(
    ctx: &mut Context,
    param_name_node: ParamNameNode,
) -> Result<ParamNameNode, QErrorNode> {
    // destruct param_name_node
    let Locatable {
        element: ParamName {
            bare_name,
            var_type: param_type,
        },
        pos,
    } = param_name_node;
    // construct dim_list
    let dim_type = DimType::from(param_type);
    let dim_list: DimList = DimNameBuilder::new()
        .bare_name(bare_name)
        .dim_type(dim_type)
        .build_list(pos);
    // convert
    let mut converted_dim_list = on_dim(ctx, dim_list, DimContext::Param)?;
    let Locatable {
        element: DimName {
            bare_name,
            var_type: dim_type,
        },
        ..
    } = converted_dim_list
        .variables
        .pop()
        .expect("Should have one converted variable");
    let param_type = ParamType::from(dim_type);
    let param_name = ParamName::new(bare_name, param_type);
    Ok(param_name.at(pos))
}
