use crate::common::{QErrorNode, ToErrorEnvelopeNoPos};
use crate::linter::converter::dim_rules::dim_type_rules;
use crate::linter::converter::pos_context::PosContext;
use crate::parser::{BareName, ParamType};

pub fn on_param_type<'a>(
    dim_type: ParamType,
    bare_name: &BareName,
    ctx: &mut PosContext<'a>,
) -> Result<ParamType, QErrorNode> {
    match dim_type {
        ParamType::Bare => dim_type_rules::bare_to_dim_type(ctx, bare_name).with_err_no_pos(),
        ParamType::BuiltIn(q, built_in_style) => {
            dim_type_rules::built_in_to_dim_type(ctx, bare_name, q, built_in_style)
                .with_err_no_pos()
        }
        ParamType::UserDefined(u) => {
            dim_type_rules::user_defined_to_dim_type(ctx, bare_name, u).with_err_no_pos()
        }
        ParamType::Array(element_type) => param_array_to_param_type(ctx, bare_name, *element_type),
    }
}

fn param_array_to_param_type<'a>(
    ctx: &mut PosContext<'a>,
    bare_name: &BareName,
    element_type: ParamType,
) -> Result<ParamType, QErrorNode> {
    let resolved_element_dim_type = on_param_type(element_type, bare_name, ctx)?;
    Ok(ParamType::Array(Box::new(resolved_element_dim_type)))
}
