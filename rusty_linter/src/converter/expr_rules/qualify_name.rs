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
            match name {
                // trying to use a qualifier on an ExpressionType that doesn't accept it
                Name::Qualified(_, _) => Err(LintError::TypeMismatch),
                _ => Ok(name),
            }
        }
    }
}

/// Tries to convert this name into a qualified name.
/// Fails if the name is already qualified with a different qualifier.
pub fn try_qualify(name: Name, qualifier: TypeQualifier) -> Result<Name, LintError> {
    match name {
        Name::Bare(bare_name) => Ok(Name::Qualified(bare_name, qualifier)),
        Name::Qualified(_, q) if q != qualifier => Err(LintError::DuplicateDefinition),
        _ => Ok(name),
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
                match n {
                    Name::Bare(_) => Err(LintError::TypeMismatch),
                    Name::Qualified(_, qualifier) => {
                        if *qualifier == TypeQualifier::DollarString {
                            Ok(Some(b))
                        } else {
                            Err(LintError::TypeMismatch)
                        }
                    }
                }
            }
            BuiltInFunction::Chr | BuiltInFunction::Str | BuiltInFunction::String => {
                // STR$ or otherwise it's undefined
                match n {
                    // confirmed that even with DEFSTR A-Z it won't work as unqualified
                    Name::Bare(_) => Ok(None),
                    Name::Qualified(_, qualifier) => {
                        if *qualifier == TypeQualifier::DollarString {
                            Ok(Some(b))
                        } else {
                            Ok(None)
                        }
                    }
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
    match n {
        Name::Bare(_) => Ok(Some(built_in)),
        _ => Err(LintError::TypeMismatch),
    }
}
