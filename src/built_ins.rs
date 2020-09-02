use crate::common::*;
use crate::parser::{HasQualifier, Name, TypeQualifier};
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInFunction {
    /// CHR$
    Chr,
    /// EOF
    Eof,
    /// ENVIRON$
    Environ,
    /// INSTR
    InStr,
    /// LEN
    Len,
    /// MID$
    Mid,
    /// STR$
    Str,
    /// VAL
    Val,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInSub {
    Environ,
    Input,
    Print,
    System,
    Close,
    Open,
    Kill,
    LineInput,
    Name,
}

impl HasQualifier for BuiltInFunction {
    fn qualifier(&self) -> TypeQualifier {
        match self {
            Self::Chr => TypeQualifier::DollarString,
            Self::Environ => TypeQualifier::DollarString,
            Self::Eof => TypeQualifier::PercentInteger,
            Self::InStr => TypeQualifier::PercentInteger,
            Self::Len => TypeQualifier::PercentInteger,
            Self::Mid => TypeQualifier::DollarString,
            Self::Str => TypeQualifier::DollarString,
            Self::Val => TypeQualifier::BangSingle,
        }
    }
}

impl From<&CaseInsensitiveString> for Option<BuiltInFunction> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInFunction> {
        if s == "EOF" {
            Some(BuiltInFunction::Eof)
        } else if s == "ENVIRON" {
            Some(BuiltInFunction::Environ)
        } else if s == "LEN" {
            Some(BuiltInFunction::Len)
        } else if s == "STR" {
            Some(BuiltInFunction::Str)
        } else if s == "VAL" {
            Some(BuiltInFunction::Val)
        } else if s == "CHR" {
            Some(BuiltInFunction::Chr)
        } else if s == "INSTR" {
            Some(BuiltInFunction::InStr)
        } else if s == "MID" {
            Some(BuiltInFunction::Mid)
        } else {
            None
        }
    }
}

fn demand_unqualified(
    built_in: BuiltInFunction,
    n: &Name,
) -> Result<Option<BuiltInFunction>, QError> {
    match n {
        Name::Bare(_) => Ok(Some(built_in)),
        _ => Err(QError::SyntaxError(format!(
            "Function {:?} must be unqualified",
            built_in
        ))),
    }
}

impl TryFrom<&Name> for Option<BuiltInFunction> {
    type Error = QError;
    fn try_from(n: &Name) -> Result<Option<BuiltInFunction>, Self::Error> {
        let opt_built_in: Option<BuiltInFunction> = n.as_ref().into();
        match opt_built_in {
            Some(b) => match b {
                BuiltInFunction::Eof
                | BuiltInFunction::InStr
                | BuiltInFunction::Len
                | BuiltInFunction::Val => demand_unqualified(b, n),
                BuiltInFunction::Environ | BuiltInFunction::Mid => {
                    // ENVIRON$ must be qualified
                    match n {
                        Name::Bare(_) => Err(QError::SyntaxError(format!(
                            "Function {:?} must be qualified",
                            n
                        ))),
                        Name::Qualified { qualifier, .. } => {
                            if *qualifier == TypeQualifier::DollarString {
                                Ok(Some(b))
                            } else {
                                Err(QError::TypeMismatch)
                            }
                        }
                    }
                }
                BuiltInFunction::Chr | BuiltInFunction::Str => {
                    // STR$ or otherwise it's undefined
                    match n {
                        // confirmed that even with DEFSTR A-Z it won't work as unqualified
                        Name::Bare(_) => Ok(None),
                        Name::Qualified { qualifier, .. } => {
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
}

impl From<&CaseInsensitiveString> for Option<BuiltInSub> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInSub> {
        if s == "ENVIRON" {
            Some(BuiltInSub::Environ)
        } else if s == "INPUT" {
            Some(BuiltInSub::Input)
        } else if s == "PRINT" {
            Some(BuiltInSub::Print)
        } else if s == "SYSTEM" {
            Some(BuiltInSub::System)
        } else if s == "CLOSE" {
            Some(BuiltInSub::Close)
        } else if s == "OPEN" {
            Some(BuiltInSub::Open)
        } else if s == "LINE INPUT" {
            Some(BuiltInSub::LineInput)
        } else if s == "NAME" {
            Some(BuiltInSub::Name)
        } else if s == "KILL" {
            Some(BuiltInSub::Kill)
        } else {
            None
        }
    }
}
