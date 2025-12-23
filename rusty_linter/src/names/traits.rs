use rusty_parser::specific::{BareName, BuiltInStyle, TypeQualifier, VariableInfo};
use rusty_variant::Variant;

pub trait SingleNameTrait {
    fn get_compact(&self, qualifier: TypeQualifier) -> Option<&VariableInfo>;

    fn get_extended(&self) -> Option<&VariableInfo>;

    fn get_const_value(&self) -> Option<&Variant>;

    fn collect_var_info(&self, only_shared: bool) -> Vec<(BuiltInStyle, &VariableInfo)>;

    fn insert_compact(&mut self, variable_info: VariableInfo);
}

pub trait ManyNamesTrait {
    fn get_compact(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&VariableInfo>;

    fn get_extended(&self, bare_name: &BareName) -> Option<&VariableInfo>;

    fn get_const_value(&self, bare_name: &BareName) -> Option<&Variant>;

    fn collect_var_info(
        &self,
        bare_name: &BareName,
        only_shared: bool,
    ) -> Vec<(BuiltInStyle, &VariableInfo)>;

    fn insert_compact(&mut self, bare_name: BareName, variable_info: VariableInfo);
}
