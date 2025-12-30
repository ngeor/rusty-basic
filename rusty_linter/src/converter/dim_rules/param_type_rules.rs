use rusty_common::Position;
use rusty_parser::{BareName, ParamType};

use crate::converter::common::Context;
use crate::converter::dim_rules::dim_type_rules;
use crate::core::LintErrorPos;

pub fn on_param_type(
    dim_type: ParamType,
    bare_name: &BareName,
    ctx: &mut Context,
    pos: Position,
) -> Result<ParamType, LintErrorPos> {
    match dim_type {
        ParamType::Bare => dim_type_rules::bare_to_dim_type(ctx, bare_name, pos),
        ParamType::BuiltIn(q, built_in_style) => {
            dim_type_rules::built_in_to_dim_type(ctx, bare_name, q, built_in_style, pos)
        }
        ParamType::UserDefined(u) => {
            dim_type_rules::user_defined_to_dim_type(ctx, bare_name, u, pos)
        }
        ParamType::Array(element_type) => {
            param_array_to_param_type(ctx, pos, bare_name, *element_type)
        }
    }
}

fn param_array_to_param_type(
    ctx: &mut Context,
    pos: Position,
    bare_name: &BareName,
    element_type: ParamType,
) -> Result<ParamType, LintErrorPos> {
    let resolved_element_dim_type = on_param_type(element_type, bare_name, ctx, pos)?;
    Ok(ParamType::Array(Box::new(resolved_element_dim_type)))
}
