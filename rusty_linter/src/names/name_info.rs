use std::collections::HashMap;

use rusty_parser::specific::{TypeQualifier, VariableInfo};
use rusty_variant::Variant;


pub enum NameInfo {
    Constant(Variant),
    Compacts(HashMap<TypeQualifier, VariableInfo>),
    Extended(VariableInfo),
}
