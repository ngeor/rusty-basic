use super::{dim, redim, validation};
use crate::common::*;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::conversion_traits::{
    SameTypeConverterWithImplicits, SameTypeConverterWithImplicitsInContext,
};
use crate::linter::converter::{Context, ExprContext, Implicits, R};
use crate::linter::DimContext;
use crate::parser::*;
use crate::variant::{QBNumberCast, MAX_INTEGER};

pub fn on_dim(ctx: &mut Context, dim_list: DimList, dim_context: DimContext) -> R<DimList> {
    ctx.convert_same_type_with_implicits_in_context(dim_list, dim_context)
}

impl SameTypeConverterWithImplicits<ArrayDimension> for Context {
    fn convert_same_type_with_implicits(&mut self, a: ArrayDimension) -> R<ArrayDimension> {
        let (lbound, mut implicits) = self.on_opt_expression(a.lbound, ExprContext::Default)?;
        let (ubound, mut ubound_implicits) = self.on_expression(a.ubound, ExprContext::Default)?;
        implicits.append(&mut ubound_implicits);
        Ok((ArrayDimension { lbound, ubound }, implicits))
    }
}

impl SameTypeConverterWithImplicitsInContext<DimList, DimContext> for Context {
    fn convert_same_type_with_implicits_in_context(
        &mut self,
        item: DimList,
        dim_context: DimContext,
    ) -> R<DimList> {
        let DimList { variables, shared } = item;
        let new_extra = (dim_context, shared);
        let (converted_variables, implicits) =
            self.convert_same_type_with_implicits_in_context(variables, new_extra)?;
        let converted_dim_list = DimList {
            variables: converted_variables,
            shared,
        };
        Ok((converted_dim_list, implicits))
    }
}

impl SameTypeConverterWithImplicitsInContext<DimNameNode, (DimContext, bool)> for Context {
    fn convert_same_type_with_implicits_in_context(
        &mut self,
        item: DimNameNode,
        context: (DimContext, bool),
    ) -> R<DimNameNode> {
        let (dim_context, shared) = context;
        let Locatable {
            element:
                DimName {
                    bare_name,
                    var_type: dim_type,
                },
            pos,
        } = item;
        convert(self, bare_name, dim_type, dim_context, shared, pos)
    }
}

fn convert(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> R<DimNameNode> {
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
) -> R<DimNameNode> {
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

pub fn convert_array_dimensions(
    ctx: &mut Context,
    array_dimensions: ArrayDimensions,
) -> R<ArrayDimensions> {
    ctx.convert_same_type_with_implicits(array_dimensions)
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
    let (mut converted_dim_list, implicits) = on_dim(ctx, dim_list, DimContext::Param)?;
    debug_assert!(
        implicits.is_empty(),
        "Should not have introduced implicit variables via parameter"
    );
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

pub fn no_implicits<T>(value: T) -> (T, Implicits) {
    (value, vec![])
}
