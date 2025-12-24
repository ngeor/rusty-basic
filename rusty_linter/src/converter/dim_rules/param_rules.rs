use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::PosContext;
use crate::converter::dim_rules::param_type_rules::on_param_type;
use crate::converter::dim_rules::validation;
use crate::core::LintErrorPos;
use crate::core::LintPosResult;
use rusty_common::AtPos;
use rusty_parser::{Parameter, ParameterPos};

impl Convertible for ParameterPos {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let Self { element, pos } = self;
        element
            .convert_in(ctx, pos)
            .map(|p| p.at_pos(pos))
            .patch_err_pos(&pos)
    }
}

impl<'a> Convertible<PosContext<'a>> for Parameter {
    fn convert(self, ctx: &mut PosContext<'a>) -> Result<Self, LintErrorPos> {
        validation::validate(&self, ctx)?;
        let Self {
            bare_name,
            var_type,
        } = self;
        let var_type = on_param_type(var_type, &bare_name, ctx)?;
        ctx.names.insert(bare_name.clone(), &var_type, false, None);
        Ok(Self::new(bare_name, var_type))
    }
}
