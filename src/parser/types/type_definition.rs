use crate::common::CaseInsensitiveString;
use crate::parser::types::TypeQualifier;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeDefinition {
    Bare,
    CompactBuiltIn(TypeQualifier),
    ExtendedBuiltIn(TypeQualifier),
    // TODO add test proving it is not possible to have a parameter like X AS STRING * 8
    // TODO add test passing a STRING(len) into a STRING parameter and show it's getting truncated AFTER the sub exits
    UserDefined(CaseInsensitiveString),
}
