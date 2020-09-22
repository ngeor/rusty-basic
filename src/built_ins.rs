use crate::common::*;
use crate::linter::DimName;
use crate::parser::{HasQualifier, Name, QualifiedName, TypeQualifier};
use std::convert::TryFrom;

// ========================================================
// BuiltInFunction
// ========================================================

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BuiltInFunction {
    /// CHR$
    Chr,
    /// ENVIRON$
    Environ,
    /// EOF
    Eof,
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

const SORTED_BUILT_IN_FUNCTIONS: [BuiltInFunction; 8] = [
    BuiltInFunction::Chr,
    BuiltInFunction::Environ,
    BuiltInFunction::Eof,
    BuiltInFunction::InStr,
    BuiltInFunction::Len,
    BuiltInFunction::Mid,
    BuiltInFunction::Str,
    BuiltInFunction::Val,
];

const SORTED_BUILT_IN_FUNCTION_NAMES: [&str; 8] =
    ["Chr", "Environ", "Eof", "InStr", "Len", "Mid", "Str", "Val"];

// BuiltInFunction -> &str

impl AsRef<str> for BuiltInFunction {
    fn as_ref(&self) -> &str {
        let idx = SORTED_BUILT_IN_FUNCTIONS
            .binary_search(self)
            .expect("Missing built-in function!");
        SORTED_BUILT_IN_FUNCTION_NAMES[idx]
    }
}

// BuiltInFunction -> CaseInsensitiveString

impl From<BuiltInFunction> for CaseInsensitiveString {
    fn from(x: BuiltInFunction) -> Self {
        Self::from(x.as_ref())
    }
}

// BuiltInFunction -> TypeQualifier

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

// BuiltInFunction -> QualifiedName

impl From<BuiltInFunction> for QualifiedName {
    fn from(x: BuiltInFunction) -> Self {
        Self::new(x.into(), x.qualifier())
    }
}

// BuiltInFunction -> ResolvedDeclaredName

impl From<BuiltInFunction> for DimName {
    fn from(x: BuiltInFunction) -> Self {
        Self::BuiltIn(x.into(), x.qualifier())
    }
}

// CaseInsensitiveString -> BuiltInFunction

impl From<&CaseInsensitiveString> for Option<BuiltInFunction> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInFunction> {
        match SORTED_BUILT_IN_FUNCTION_NAMES
            .binary_search_by(|p| CmpIgnoreAsciiCase::compare_ignore_ascii_case(*p, s.as_ref()))
        {
            Ok(idx) => Some(SORTED_BUILT_IN_FUNCTIONS[idx]),
            Err(_) => None,
        }
    }
}

// Name -> BuiltInFunction

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

// ========================================================
// BuiltInSub
// ========================================================

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
