use crate::converter::context::Context;
use crate::converter::dim_rules::param_type_rules::on_param_type;
use crate::converter::dim_rules::validation;
use crate::converter::pos_context::PosContext;
use crate::converter::traits::Convertible;
use rusty_common::{AtLocation, PatchErrPos, QErrorNode};
use rusty_parser::{ParamName, ParamNameNode};

impl Convertible for ParamNameNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let Self { element, pos } = self;
        element
            .convert_in(ctx, pos)
            .map(|p| p.at(pos))
            .patch_err_pos(pos)
    }
}

impl<'a> Convertible<PosContext<'a>> for ParamName {
    fn convert(self, ctx: &mut PosContext<'a>) -> Result<Self, QErrorNode> {
        validation::validate(&self, ctx)?;
        let Self {
            bare_name,
            var_type,
        } = self;
        let var_type = on_param_type(var_type, &bare_name, ctx)?;
        ctx.names.insert(bare_name.clone(), &var_type, false, None);
        Ok(ParamName::new(bare_name, var_type))
    }
}