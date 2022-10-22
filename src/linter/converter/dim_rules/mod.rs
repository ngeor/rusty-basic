mod array_dimension;
mod dim_list_state;
mod dim_name_state;
mod dim_rules;
mod dim_type_rules;
mod param_rules;
mod param_type_rules;
mod redim;
mod string_length;
mod validation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DimContext {
    /// Normal DIM statement
    Default,

    /// REDIM statement
    Redim,
}

impl Default for DimContext {
    fn default() -> Self {
        Self::Default
    }
}
