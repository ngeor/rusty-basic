use crate::{keyword_enum, BareName, Name, TypeQualifier};
use rusty_common::*;
use std::convert::TryFrom;

keyword_enum!(
pub enum BuiltInFunction SORTED_BUILT_IN_FUNCTIONS SORTED_BUILT_IN_FUNCTION_NAMES {
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
    String,

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
});

// BuiltInFunction -> CaseInsensitiveString

impl From<BuiltInFunction> for CaseInsensitiveString {
    fn from(x: BuiltInFunction) -> Self {
        Self::from(x.as_ref())
    }
}

// BuiltInFunction -> TypeQualifier

impl From<&BuiltInFunction> for TypeQualifier {
    fn from(x: &BuiltInFunction) -> Self {
        match x {
            BuiltInFunction::Chr => Self::DollarString,
            BuiltInFunction::Cvd => Self::HashDouble,
            BuiltInFunction::Environ => Self::DollarString,
            BuiltInFunction::Eof => Self::PercentInteger,
            BuiltInFunction::Err => Self::PercentInteger,
            BuiltInFunction::InKey => Self::DollarString,
            BuiltInFunction::InStr => Self::PercentInteger,
            BuiltInFunction::LBound => Self::PercentInteger,
            BuiltInFunction::LCase => Self::DollarString,
            BuiltInFunction::Left => Self::DollarString,
            BuiltInFunction::Len => Self::PercentInteger,
            BuiltInFunction::LTrim => Self::DollarString,
            BuiltInFunction::Mid => Self::DollarString,
            BuiltInFunction::Mkd => Self::DollarString,
            BuiltInFunction::Peek => Self::PercentInteger,
            BuiltInFunction::Right => Self::DollarString,
            BuiltInFunction::RTrim => Self::DollarString,
            BuiltInFunction::Space => Self::DollarString,
            BuiltInFunction::Str => Self::DollarString,
            BuiltInFunction::String => Self::DollarString,
            BuiltInFunction::UBound => Self::PercentInteger,
            BuiltInFunction::UCase => Self::DollarString,
            BuiltInFunction::Val => Self::BangSingle,
            BuiltInFunction::VarPtr => Self::PercentInteger,
            BuiltInFunction::VarSeg => Self::PercentInteger,
        }
    }
}

// BuiltInFunction -> QualifiedName

impl From<BuiltInFunction> for Name {
    fn from(built_in_function: BuiltInFunction) -> Self {
        let qualifier: TypeQualifier = (&built_in_function).into();
        let bare_name: BareName = built_in_function.into();
        Self::qualified(bare_name, qualifier)
    }
}

// CaseInsensitiveString -> BuiltInFunction

impl BuiltInFunction {
    pub fn try_parse(s: &BareName) -> Option<Self> {
        Self::try_from(s.as_str()).ok()
    }
}
