use crate::common::*;
use crate::linter::{DimName, DimType};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use std::convert::TryFrom;

// ========================================================
// BuiltInFunction
// ========================================================

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
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

impl std::fmt::Debug for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl std::fmt::Display for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// BuiltInFunction -> CaseInsensitiveString

impl From<BuiltInFunction> for CaseInsensitiveString {
    fn from(x: BuiltInFunction) -> Self {
        Self::from(x.as_ref())
    }
}

// BuiltInFunction -> TypeQualifier

impl From<&BuiltInFunction> for TypeQualifier {
    fn from(x: &BuiltInFunction) -> TypeQualifier {
        match x {
            BuiltInFunction::Chr => TypeQualifier::DollarString,
            BuiltInFunction::Environ => TypeQualifier::DollarString,
            BuiltInFunction::Eof => TypeQualifier::PercentInteger,
            BuiltInFunction::InStr => TypeQualifier::PercentInteger,
            BuiltInFunction::Len => TypeQualifier::PercentInteger,
            BuiltInFunction::Mid => TypeQualifier::DollarString,
            BuiltInFunction::Str => TypeQualifier::DollarString,
            BuiltInFunction::Val => TypeQualifier::BangSingle,
        }
    }
}

// BuiltInFunction -> QualifiedName

impl From<BuiltInFunction> for QualifiedName {
    fn from(built_in_function: BuiltInFunction) -> Self {
        let qualifier: TypeQualifier = (&built_in_function).into();
        Self::new(built_in_function.into(), qualifier)
    }
}

// BuiltInFunction -> DimName

impl From<BuiltInFunction> for DimName {
    fn from(built_in_function: BuiltInFunction) -> Self {
        let qualifier: TypeQualifier = (&built_in_function).into();
        let bare_name: BareName = built_in_function.into();
        Self::new(bare_name, DimType::BuiltIn(qualifier))
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
                        Name::Qualified(QualifiedName { qualifier, .. }) => {
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
                        Name::Qualified(QualifiedName { qualifier, .. }) => {
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

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum BuiltInSub {
    Close,
    Environ,
    Input,
    Kill,
    LineInput,
    LPrint,
    Name,
    Open,
    Print,
    System,
}

const SORTED_BUILT_IN_SUBS: [BuiltInSub; 10] = [
    BuiltInSub::Close,
    BuiltInSub::Environ,
    BuiltInSub::Input,
    BuiltInSub::Kill,
    BuiltInSub::LineInput,
    BuiltInSub::LPrint,
    BuiltInSub::Name,
    BuiltInSub::Open,
    BuiltInSub::Print,
    BuiltInSub::System,
];

const SORTED_BUILT_IN_SUB_NAMES: [&str; 10] = [
    "CLOSE",
    "ENVIRON",
    "INPUT",
    "KILL",
    "LINE INPUT",
    "LPRINT",
    "NAME",
    "OPEN",
    "PRINT",
    "SYSTEM",
];

// BuiltInSub -> &str

impl AsRef<str> for BuiltInSub {
    fn as_ref(&self) -> &str {
        let idx = SORTED_BUILT_IN_SUBS
            .binary_search(self)
            .expect("Missing built-in sub!");
        SORTED_BUILT_IN_SUB_NAMES[idx]
    }
}

impl std::fmt::Debug for BuiltInSub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl std::fmt::Display for BuiltInSub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// BuiltInSub -> CaseInsensitiveString

impl From<BuiltInSub> for CaseInsensitiveString {
    fn from(x: BuiltInSub) -> Self {
        Self::from(x.as_ref())
    }
}

// CaseInsensitiveString -> BuiltInSub

impl From<&CaseInsensitiveString> for Option<BuiltInSub> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInSub> {
        match SORTED_BUILT_IN_SUB_NAMES
            .binary_search_by(|p| CmpIgnoreAsciiCase::compare_ignore_ascii_case(*p, s.as_ref()))
        {
            Ok(idx) => Some(SORTED_BUILT_IN_SUBS[idx]),
            Err(_) => None,
        }
    }
}
