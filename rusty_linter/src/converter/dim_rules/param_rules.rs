use rusty_common::Position;
use rusty_parser::Parameter;

use crate::converter::common::ConvertibleIn;
use crate::converter::dim_rules::param_type_rules::on_param_type;
use crate::converter::dim_rules::validation;
use crate::core::{Context, LintErrorPos};

impl ConvertibleIn<Position> for Parameter {
    fn convert_in(self, ctx: &mut Context, pos: Position) -> Result<Self, LintErrorPos> {
        validation::validate(&self, ctx, pos)?;
        let (bare_name, var_type) = self.into();
        let var_type = on_param_type(var_type, &bare_name, ctx, pos)?;
        ctx.names.insert(bare_name.clone(), &var_type, false, None);
        Ok(Self::new(bare_name, var_type))
    }
}
