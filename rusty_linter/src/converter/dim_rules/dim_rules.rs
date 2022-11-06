use crate::converter::dim_rules::dim_list_state::DimListState;
use crate::converter::dim_rules::dim_name_state::DimNameState;
use crate::converter::dim_rules::dim_type_rules::on_dim_type;
use crate::converter::dim_rules::redim::on_redim_type;
use crate::converter::dim_rules::validation;
use crate::converter::traits::Convertible;
use crate::converter::types::DimContext;
use crate::error::LintErrorPos;
use crate::{LintError, LintPosResult};
use rusty_common::*;
use rusty_parser::*;

impl<'a> Convertible<DimListState<'a>> for DimList {
    fn convert(self, ctx: &mut DimListState<'a>) -> Result<Self, LintErrorPos> {
        let Self { variables, shared } = self;
        let mut new_variables = DimVars::new();
        for Positioned { element, pos } in variables {
            let mut new_state = DimNameState::new(ctx, shared, pos);
            let new_dim_name = element.convert(&mut new_state).patch_err_pos(&pos)?;
            new_variables.push(new_dim_name.at_pos(pos));
        }
        Ok(DimList {
            variables: new_variables,
            shared,
        })
    }
}

impl<'a, 'b> Convertible<DimNameState<'a, 'b>> for DimVar {
    fn convert(self, ctx: &mut DimNameState<'a, 'b>) -> Result<Self, LintErrorPos> {
        validation::validate(&self, ctx)?;
        shared_illegal_in_sub_function(ctx)?;
        let Self {
            bare_name,
            var_type,
        } = self;
        let (var_type, redim_info) = if ctx.dim_context() == DimContext::Redim {
            on_redim_type(var_type, &bare_name, ctx)?
        } else {
            let var_type = on_dim_type(var_type, &bare_name, ctx)?;
            (var_type, None)
        };
        let shared = ctx.is_shared();
        ctx.names
            .insert(bare_name.clone(), &var_type, shared, redim_info);
        Ok(DimVar::new(bare_name, var_type))
    }
}

fn shared_illegal_in_sub_function(ctx: &DimNameState) -> Result<(), LintError> {
    if ctx.is_shared() && ctx.is_in_subprogram() {
        Err(LintError::IllegalInSubFunction)
    } else {
        Ok(())
    }
}
