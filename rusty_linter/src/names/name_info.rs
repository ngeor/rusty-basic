use rusty_parser::specific::VariableInfo;
use rusty_variant::Variant;

use crate::names::compacts_info::CompactsInfo;

pub enum NameInfo {
    Constant(Variant),
    Compacts(CompactsInfo),
    Extended(VariableInfo),
}
