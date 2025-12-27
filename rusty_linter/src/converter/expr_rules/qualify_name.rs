use rusty_parser::{
    BuiltInFunction, {ExpressionType, Name, TypeQualifier},
};

use crate::LintError;

/// Validates and normalizes the given name
pub fn qualify_name(expression_type: &ExpressionType, name: Name) -> Result<Name, LintError> {
    match expression_type.opt_qualifier() {
        Some(expr_q) => {
            // try to modify the name to have the expected qualifier
            try_qualify(name, expr_q).map_err(|_| LintError::TypeMismatch)
        }
        None => {
            if name.is_bare() {
                Ok(name)
            } else {
                // trying to use a qualifier on an ExpressionType that doesn't accept it
                Err(LintError::TypeMismatch)
            }
        }
    }
}

/// Tries to convert this name into a qualified name.
/// Fails if the name is already qualified with a different qualifier.
pub fn try_qualify(name: Name, qualifier: TypeQualifier) -> Result<Name, LintError> {
    match name.qualifier() {
        Some(q) if q != qualifier => Err(LintError::DuplicateDefinition),
        Some(_) => Ok(name),
        None => Ok(Name::qualified(name.into(), qualifier)),
    }
}

// Name -> BuiltInFunction

pub fn try_built_in_function(n: &Name) -> Result<Option<BuiltInFunction>, LintError> {
    let opt_built_in: Option<BuiltInFunction> = BuiltInFunction::try_parse(n.bare_name());
    match opt_built_in {
        Some(b) => match b {
            BuiltInFunction::Cvd
            | BuiltInFunction::Eof
            | BuiltInFunction::Err
            | BuiltInFunction::InStr
            | BuiltInFunction::Len
            | BuiltInFunction::Peek
            | BuiltInFunction::LBound
            | BuiltInFunction::UBound
            | BuiltInFunction::Val
            | BuiltInFunction::VarPtr
            | BuiltInFunction::VarSeg => demand_unqualified(b, n),
            BuiltInFunction::Environ
            | BuiltInFunction::InKey
            | BuiltInFunction::LCase
            | BuiltInFunction::Left
            | BuiltInFunction::LTrim
            | BuiltInFunction::Mid
            | BuiltInFunction::Mkd
            | BuiltInFunction::Right
            | BuiltInFunction::RTrim
            | BuiltInFunction::Space
            | BuiltInFunction::UCase => {
                // ENVIRON$ must be qualified
                match n.qualifier() {
                    Some(TypeQualifier::DollarString) => Ok(Some(b)),
                    _ => Err(LintError::TypeMismatch),
                }
            }
            BuiltInFunction::Chr | BuiltInFunction::Str | BuiltInFunction::String => {
                // STR$ or otherwise it's undefined
                // confirmed that even with DEFSTR A-Z it won't work as unqualified
                match n.qualifier() {
                    Some(TypeQualifier::DollarString) => Ok(Some(b)),
                    _ => Ok(None),
                }
            }
        },
        None => Ok(None),
    }
}

fn demand_unqualified(
    built_in: BuiltInFunction,
    n: &Name,
) -> Result<Option<BuiltInFunction>, LintError> {
    if n.is_bare() {
        Ok(Some(built_in))
    } else {
        Err(LintError::TypeMismatch)
    }
}
