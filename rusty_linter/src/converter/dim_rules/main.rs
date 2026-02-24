use rusty_common::*;
use rusty_parser::*;

use crate::converter::common::{ConvertibleIn, DimContext, DimNameState};
use crate::converter::dim_rules::dim_type_rules::on_dim_type;
use crate::converter::dim_rules::redim::on_redim_type;
use crate::converter::dim_rules::validation;
use crate::core::{LintError, LintErrorPos, LinterContext};

impl ConvertibleIn<DimContext> for DimList {
    fn convert_in(
        self,
        ctx: &mut LinterContext,
        dim_context: DimContext,
    ) -> Result<Self, LintErrorPos> {
        let Self { variables, shared } = self;
        let mut new_variables = DimVars::new();
        for Positioned { element, pos } in variables {
            let new_state = DimNameState {
                dim_context,
                shared,
                pos,
            };
            let new_dim_name = element.convert_in(ctx, new_state)?;
            new_variables.push(new_dim_name.at_pos(pos));
        }
        Ok(Self {
            variables: new_variables,
            shared,
        })
    }
}

impl ConvertibleIn<DimNameState> for DimVar {
    fn convert_in(
        self,
        ctx: &mut LinterContext,
        extra: DimNameState,
    ) -> Result<Self, LintErrorPos> {
        validation::validate(&self, ctx, extra.pos)?;
        let shared = extra.shared;
        shared_illegal_in_sub_function(ctx, shared, extra.pos)?;
        let (bare_name, var_type) = self.into();
        let (var_type, redim_info) = if extra.dim_context == DimContext::Redim {
            on_redim_type(var_type, &bare_name, ctx, extra)?
        } else {
            let var_type = on_dim_type(var_type, &bare_name, ctx, extra)?;
            (var_type, None)
        };
        ctx.names
            .insert(bare_name.clone(), &var_type, shared, redim_info);
        Ok(Self::new(bare_name, var_type))
    }
}

fn shared_illegal_in_sub_function(
    ctx: &LinterContext,
    shared: bool,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if shared && ctx.is_in_subprogram() {
        Err(LintError::IllegalInSubFunction.at_pos(pos))
    } else {
        Ok(())
    }
}
