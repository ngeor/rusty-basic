use crate::common::*;
use crate::parser::{Name, QualifiedName, TypeQualifier};
use std::convert::TryFrom;

// ========================================================
// BuiltInFunction
// ========================================================

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BuiltInFunction {
    /// `CHR$(ascii-code%)` returns the text representation of the given ascii code
    Chr,

    /// `CVD(8 byte string-expression)`
    ///
    /// Converts a string previously created by `MKD$` into a double.
    Cvd,

    /// `ENVIRON$ (env-variable$)` -> returns the variable
    ///
    /// `ENVIRON$ (n%)` -> returns the nth variable (TODO support this)
    Environ,

    /// `EOF(file-number%)` -> checks if the end of file has been reached
    Eof,

    /// `ERR`
    Err,

    /// `INKEY$`
    ///
    /// Reads a character from the keyboard.
    InKey,

    /// `INSTR([start%,] hay$, needle$)`
    /// if start% is omitted, INSTR starts at position 1
    /// returns the first occurrence of needle$ inside hay$
    InStr,

    /// LBOUND
    LBound,

    /// LCASE$
    LCase,

    /// `LEFT$(str_expr$, count%)`
    Left,

    /// `LEN(str_expr$)` -> number of characters in string
    ///
    /// `LEN(variable)` -> number of bytes required to store a variable
    Len,

    /// `LTRIM$`
    LTrim,

    /// MID$ function returns part of a string
    ///
    /// MID$ statement replaces part of a string (TODO support this)
    ///
    /// MID$(str_expr$, start%[, length%])
    ///
    /// MID$(str_var$, start%[, length%]) = str_expr$
    ///
    /// if the length is omitted, returns or replaces all remaining characters
    Mid,

    /// `MKD$(double-expression#)`
    ///
    /// Converts a double precision number into an 8 byte string that can be
    /// used in `FIELD` statements.
    Mkd,

    /// `PEEK`
    Peek,

    /// `RIGHT$(str_expr$, count%)`
    Right,

    /// `RTRIM$`
    RTrim,

    /// `SPACE$(number-of-spaces)`
    Space,

    /// `STR$(numeric-expression)` returns a string representation of a number
    Str,

    /// Returns a string of a specified length made up of a repeating character.
    ///
    /// `STRING$(length%, { ascii-code% | string-expression$ })`
    ///
    /// - `length%` The length of the string
    /// - `ascii-code%` The ASCII code of the repeating character
    /// - `string-expression$` Any string expression. Only the first character will be used.
    String_,

    /// UBOUND
    UBound,

    /// UCASE$
    UCase,

    /// `VAL(str-expr$)` converts a string representation of a number to a number.
    Val,

    /// `VARPTR`
    VarPtr,

    /// `VARSEG`
    VarSeg,
}

const SORTED_BUILT_IN_FUNCTIONS: [BuiltInFunction; 25] = [
    BuiltInFunction::Chr,
    BuiltInFunction::Cvd,
    BuiltInFunction::Environ,
    BuiltInFunction::Eof,
    BuiltInFunction::Err,
    BuiltInFunction::InKey,
    BuiltInFunction::InStr,
    BuiltInFunction::LBound,
    BuiltInFunction::LCase,
    BuiltInFunction::Left,
    BuiltInFunction::Len,
    BuiltInFunction::LTrim,
    BuiltInFunction::Mid,
    BuiltInFunction::Mkd,
    BuiltInFunction::Peek,
    BuiltInFunction::Right,
    BuiltInFunction::RTrim,
    BuiltInFunction::Space,
    BuiltInFunction::Str,
    BuiltInFunction::String_,
    BuiltInFunction::UBound,
    BuiltInFunction::UCase,
    BuiltInFunction::Val,
    BuiltInFunction::VarPtr,
    BuiltInFunction::VarSeg,
];

const SORTED_BUILT_IN_FUNCTION_NAMES: [&str; 25] = [
    "Chr", "Cvd", "Environ", "Eof", "Err", "InKey", "InStr", "LBound", "LCase", "Left", "Len",
    "LTrim", "Mid", "Mkd", "Peek", "Right", "RTrim", "Space", "Str", "String", "UBound", "UCase",
    "Val", "VarPtr", "VarSeg",
];

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

impl From<&BuiltInFunction> for TypeQualifier {
    fn from(x: &BuiltInFunction) -> TypeQualifier {
        match x {
            BuiltInFunction::Chr => TypeQualifier::DollarString,
            BuiltInFunction::Cvd => TypeQualifier::HashDouble,
            BuiltInFunction::Environ => TypeQualifier::DollarString,
            BuiltInFunction::Eof => TypeQualifier::PercentInteger,
            BuiltInFunction::Err => TypeQualifier::PercentInteger,
            BuiltInFunction::InKey => TypeQualifier::DollarString,
            BuiltInFunction::InStr => TypeQualifier::PercentInteger,
            BuiltInFunction::LBound => TypeQualifier::PercentInteger,
            BuiltInFunction::LCase => TypeQualifier::DollarString,
            BuiltInFunction::Left => TypeQualifier::DollarString,
            BuiltInFunction::Len => TypeQualifier::PercentInteger,
            BuiltInFunction::LTrim => TypeQualifier::DollarString,
            BuiltInFunction::Mid => TypeQualifier::DollarString,
            BuiltInFunction::Mkd => TypeQualifier::DollarString,
            BuiltInFunction::Peek => TypeQualifier::PercentInteger,
            BuiltInFunction::Right => TypeQualifier::DollarString,
            BuiltInFunction::RTrim => TypeQualifier::DollarString,
            BuiltInFunction::Space => TypeQualifier::DollarString,
            BuiltInFunction::Str => TypeQualifier::DollarString,
            BuiltInFunction::String_ => TypeQualifier::DollarString,
            BuiltInFunction::UBound => TypeQualifier::PercentInteger,
            BuiltInFunction::UCase => TypeQualifier::DollarString,
            BuiltInFunction::Val => TypeQualifier::BangSingle,
            BuiltInFunction::VarPtr => TypeQualifier::PercentInteger,
            BuiltInFunction::VarSeg => TypeQualifier::PercentInteger,
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
        let opt_built_in: Option<BuiltInFunction> = n.bare_name().into();
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
                BuiltInFunction::Chr | BuiltInFunction::Str | BuiltInFunction::String_ => {
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
