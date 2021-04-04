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

    /// `RIGHT$(str_expr$, count%)`
    Right,

    /// `RTRIM$`
    RTrim,

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
}

const SORTED_BUILT_IN_FUNCTIONS: [BuiltInFunction; 20] = [
    BuiltInFunction::Chr,
    BuiltInFunction::Cvd,
    BuiltInFunction::Environ,
    BuiltInFunction::Eof,
    BuiltInFunction::InStr,
    BuiltInFunction::LBound,
    BuiltInFunction::LCase,
    BuiltInFunction::Left,
    BuiltInFunction::Len,
    BuiltInFunction::LTrim,
    BuiltInFunction::Mid,
    BuiltInFunction::Mkd,
    BuiltInFunction::Right,
    BuiltInFunction::RTrim,
    BuiltInFunction::Str,
    BuiltInFunction::String_,
    BuiltInFunction::UBound,
    BuiltInFunction::UCase,
    BuiltInFunction::Val,
    BuiltInFunction::VarPtr,
];

const SORTED_BUILT_IN_FUNCTION_NAMES: [&str; 20] = [
    "Chr", "Cvd", "Environ", "Eof", "InStr", "LBound", "LCase", "Left", "Len", "LTrim", "Mid",
    "Mkd", "Right", "RTrim", "Str", "String", "UBound", "UCase", "Val", "VarPtr",
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
            BuiltInFunction::InStr => TypeQualifier::PercentInteger,
            BuiltInFunction::LBound => TypeQualifier::PercentInteger,
            BuiltInFunction::LCase => TypeQualifier::DollarString,
            BuiltInFunction::Left => TypeQualifier::DollarString,
            BuiltInFunction::Len => TypeQualifier::PercentInteger,
            BuiltInFunction::LTrim => TypeQualifier::DollarString,
            BuiltInFunction::Mid => TypeQualifier::DollarString,
            BuiltInFunction::Mkd => TypeQualifier::DollarString,
            BuiltInFunction::Right => TypeQualifier::DollarString,
            BuiltInFunction::RTrim => TypeQualifier::DollarString,
            BuiltInFunction::Str => TypeQualifier::DollarString,
            BuiltInFunction::String_ => TypeQualifier::DollarString,
            BuiltInFunction::UBound => TypeQualifier::PercentInteger,
            BuiltInFunction::UCase => TypeQualifier::DollarString,
            BuiltInFunction::Val => TypeQualifier::BangSingle,
            BuiltInFunction::VarPtr => TypeQualifier::PercentInteger,
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
                | BuiltInFunction::InStr
                | BuiltInFunction::Len
                | BuiltInFunction::LBound
                | BuiltInFunction::UBound
                | BuiltInFunction::Val
                | BuiltInFunction::VarPtr => demand_unqualified(b, n),
                BuiltInFunction::Environ
                | BuiltInFunction::LCase
                | BuiltInFunction::Left
                | BuiltInFunction::LTrim
                | BuiltInFunction::Mid
                | BuiltInFunction::Mkd
                | BuiltInFunction::Right
                | BuiltInFunction::RTrim
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

// ========================================================
// BuiltInSub
// ========================================================

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInSub {
    Close,
    Data,
    DefSeg,
    Environ,
    Field,
    Get,

    /// `INPUT [;] ["prompt"{; | ,}] variable-list`
    ///
    /// `INPUT #file-number%, variable-list`
    ///
    /// prompt - An optional string that is displayed before the user
    /// enters data. A semicolon after promp appends a question mark to the
    /// prompt string.
    ///
    /// variable names can consist of up to 40 characters and must begin
    /// with a letter. Valid characters are a-z, 0-9 and period (.).
    ///
    /// A semicolon immediately after INPUT keeps the cursor on the same line
    /// after the user presses the Enter key.
    ///
    Input,

    /// `KILL file-spec$` -> deletes files from disk
    Kill,

    /// `LINE INPUT` -> see [INPUT](Self::Input)
    ///
    /// `LINE INPUT [;] ["prompt";] variable$`
    ///
    /// `LINE INPUT #file-number%, variable$`
    LineInput,

    /// LOCATE moves the cursor to a specified position on the screen
    ///
    /// `LOCATE [row%] [,[column%] [,[cursor%] [,start% [,stop%]]]]`
    ///
    /// cursor 0 = invisible 1 = visible
    ///
    /// start and stop are 0..31 indicating scan lines
    Locate,
    LSet,

    /// `NAME old$ AS new$` Renames a file or directory.
    Name,

    /// `OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]`
    ///
    /// mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
    ///
    /// access: READ, WRITE, READ WRITE
    ///
    /// lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
    ///
    /// file-number a number in the range 1 through 255
    ///
    /// rec-len%: For random access files, the record length (default is 128 bytes)
    ///           For sequential files, the number of characters buffered (default is 512 bytes)
    Open,
    Put,
    Read,
    ViewPrint,
    Width,
}

impl BuiltInSub {
    /// Parses a built-in sub name which isn't implemented with a keyword.
    /// This sub would appear as a user defined SUB on the parser layer.
    ///
    /// Some statements are implemented a built-in subs (e.g. `CLOSE`, `OPEN`), but
    /// they can't hit this function, as they are represented by keywords and are
    /// parsed by custom parsers.
    pub fn parse_non_keyword_sub(s: &str) -> Option<BuiltInSub> {
        if s.eq_ignore_ascii_case("Environ") {
            Some(BuiltInSub::Environ)
        } else if s.eq_ignore_ascii_case("Kill") {
            Some(BuiltInSub::Kill)
        } else {
            None
        }
    }
}

pub mod parser {
    use crate::common::*;
    use crate::parser::pc::*;
    use crate::parser::{Expression, Statement};

    /// Parses built-in subs which have a special syntax.
    pub fn parse<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        crate::built_ins::close::parser::parse()
            .or(crate::built_ins::data::parser::parse())
            .or(crate::built_ins::def_seg::parser::parse())
            .or(crate::built_ins::field::parser::parse())
            .or(crate::built_ins::get::parser::parse())
            .or(crate::built_ins::input::parser::parse())
            .or(crate::built_ins::line_input::parser::parse())
            .or(crate::built_ins::locate::parser::parse())
            .or(crate::built_ins::lset::parser::parse())
            .or(crate::built_ins::name::parser::parse())
            .or(crate::built_ins::open::parser::parse())
            .or(crate::built_ins::put::parser::parse())
            .or(crate::built_ins::read::parser::parse())
            .or(crate::built_ins::view_print::parser::parse())
            .or(crate::built_ins::width::parser::parse())
    }

    // needed for built-in functions that are also keywords (e.g. LEN), so they
    // cannot be parsed by the `word` module.
    pub fn built_in_function_call_p<R>() -> impl Parser<R, Output = Expression>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        crate::built_ins::len::parser::parse().or(crate::built_ins::string_fn::parser::parse())
    }
}

pub mod linter {
    use crate::built_ins::{BuiltInFunction, BuiltInSub};
    use crate::common::QErrorNode;
    use crate::linter::NameContext;
    use crate::parser::ExpressionNodes;

    pub fn lint_sub_call(
        built_in_sub: &BuiltInSub,
        args: &ExpressionNodes,
        name_context: NameContext,
    ) -> Result<(), QErrorNode> {
        match built_in_sub {
            BuiltInSub::Close => crate::built_ins::close::linter::lint(args),
            BuiltInSub::Data => crate::built_ins::data::linter::lint(name_context),
            BuiltInSub::DefSeg => crate::built_ins::def_seg::linter::lint(args),
            BuiltInSub::Environ => crate::built_ins::environ_sub::linter::lint(args),
            BuiltInSub::Field => crate::built_ins::field::linter::lint(args),
            BuiltInSub::Get => crate::built_ins::get::linter::lint(args),
            BuiltInSub::Input => crate::built_ins::input::linter::lint(args),
            BuiltInSub::Kill => crate::built_ins::kill::linter::lint(args),
            BuiltInSub::LineInput => crate::built_ins::line_input::linter::lint(args),
            BuiltInSub::Locate => crate::built_ins::locate::linter::lint(args),
            BuiltInSub::LSet => crate::built_ins::lset::linter::lint(args),
            BuiltInSub::Name => crate::built_ins::name::linter::lint(args),
            BuiltInSub::Open => crate::built_ins::open::linter::lint(args),
            BuiltInSub::Put => crate::built_ins::put::linter::lint(args),
            BuiltInSub::Read => crate::built_ins::read::linter::lint(args),
            BuiltInSub::ViewPrint => crate::built_ins::view_print::linter::lint(args),
            BuiltInSub::Width => crate::built_ins::width::linter::lint(args),
        }
    }

    pub fn lint_function_call(
        built_in: &BuiltInFunction,
        args: &ExpressionNodes,
    ) -> Result<(), QErrorNode> {
        match built_in {
            BuiltInFunction::Chr => crate::built_ins::chr::linter::lint(args),
            BuiltInFunction::Cvd => crate::built_ins::cvd::linter::lint(args),
            BuiltInFunction::Environ => crate::built_ins::environ_fn::linter::lint(args),
            BuiltInFunction::Eof => crate::built_ins::eof::linter::lint(args),
            BuiltInFunction::InStr => crate::built_ins::instr::linter::lint(args),
            BuiltInFunction::LBound => crate::built_ins::lbound::linter::lint(args),
            BuiltInFunction::LCase => crate::built_ins::lcase::linter::lint(args),
            BuiltInFunction::Left => crate::built_ins::left::linter::lint(args),
            BuiltInFunction::Len => crate::built_ins::len::linter::lint(args),
            BuiltInFunction::LTrim => crate::built_ins::ltrim::linter::lint(args),
            BuiltInFunction::Mid => crate::built_ins::mid_fn::linter::lint(args),
            BuiltInFunction::Mkd => crate::built_ins::mkd::linter::lint(args),
            BuiltInFunction::Right => crate::built_ins::right::linter::lint(args),
            BuiltInFunction::RTrim => crate::built_ins::rtrim::linter::lint(args),
            BuiltInFunction::Str => crate::built_ins::str_fn::linter::lint(args),
            BuiltInFunction::String_ => crate::built_ins::string_fn::linter::lint(args),
            BuiltInFunction::UBound => crate::built_ins::ubound::linter::lint(args),
            BuiltInFunction::UCase => crate::built_ins::ucase::linter::lint(args),
            BuiltInFunction::Val => crate::built_ins::val::linter::lint(args),
            BuiltInFunction::VarPtr => crate::built_ins::varptr::linter::lint(args),
        }
    }
}

pub mod interpreter {
    use crate::built_ins::{BuiltInFunction, BuiltInSub};
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run_sub<S: InterpreterTrait>(s: &BuiltInSub, interpreter: &mut S) -> Result<(), QError> {
        match s {
            BuiltInSub::Close => crate::built_ins::close::interpreter::run(interpreter),
            BuiltInSub::Data => crate::built_ins::data::interpreter::run(interpreter),
            BuiltInSub::DefSeg => crate::built_ins::def_seg::interpreter::run(interpreter),
            BuiltInSub::Environ => crate::built_ins::environ_sub::interpreter::run(interpreter),
            BuiltInSub::Field => crate::built_ins::field::interpreter::run(interpreter),
            BuiltInSub::Get => crate::built_ins::get::interpreter::run(interpreter),
            BuiltInSub::Input => crate::built_ins::input::interpreter::run(interpreter),
            BuiltInSub::Kill => crate::built_ins::kill::interpreter::run(interpreter),
            BuiltInSub::LineInput => crate::built_ins::line_input::interpreter::run(interpreter),
            BuiltInSub::Locate => crate::built_ins::locate::interpreter::run(interpreter),
            BuiltInSub::LSet => crate::built_ins::lset::interpreter::run(interpreter),
            BuiltInSub::Name => crate::built_ins::name::interpreter::run(interpreter),
            BuiltInSub::Open => crate::built_ins::open::interpreter::run(interpreter),
            BuiltInSub::Put => crate::built_ins::put::interpreter::run(interpreter),
            BuiltInSub::Read => crate::built_ins::read::interpreter::run(interpreter),
            BuiltInSub::ViewPrint => crate::built_ins::view_print::interpreter::run(interpreter),
            BuiltInSub::Width => crate::built_ins::width::interpreter::run(interpreter),
        }
    }

    pub fn run_function<S: InterpreterTrait>(
        f: &BuiltInFunction,
        interpreter: &mut S,
    ) -> Result<(), QError> {
        match f {
            BuiltInFunction::Chr => crate::built_ins::chr::interpreter::run(interpreter),
            BuiltInFunction::Cvd => crate::built_ins::cvd::interpreter::run(interpreter),
            BuiltInFunction::Environ => crate::built_ins::environ_fn::interpreter::run(interpreter),
            BuiltInFunction::Eof => crate::built_ins::eof::interpreter::run(interpreter),
            BuiltInFunction::InStr => crate::built_ins::instr::interpreter::run(interpreter),
            BuiltInFunction::LBound => crate::built_ins::lbound::interpreter::run(interpreter),
            BuiltInFunction::LCase => crate::built_ins::lcase::interpreter::run(interpreter),
            BuiltInFunction::Left => crate::built_ins::left::interpreter::run(interpreter),
            BuiltInFunction::Len => crate::built_ins::len::interpreter::run(interpreter),
            BuiltInFunction::LTrim => crate::built_ins::ltrim::interpreter::run(interpreter),
            BuiltInFunction::Mid => crate::built_ins::mid_fn::interpreter::run(interpreter),
            BuiltInFunction::Mkd => crate::built_ins::mkd::interpreter::run(interpreter),
            BuiltInFunction::Right => crate::built_ins::right::interpreter::run(interpreter),
            BuiltInFunction::RTrim => crate::built_ins::rtrim::interpreter::run(interpreter),
            BuiltInFunction::Str => crate::built_ins::str_fn::interpreter::run(interpreter),
            BuiltInFunction::String_ => crate::built_ins::string_fn::interpreter::run(interpreter),
            BuiltInFunction::UBound => crate::built_ins::ubound::interpreter::run(interpreter),
            BuiltInFunction::UCase => crate::built_ins::ucase::interpreter::run(interpreter),
            BuiltInFunction::Val => crate::built_ins::val::interpreter::run(interpreter),
            BuiltInFunction::VarPtr => crate::built_ins::varptr::interpreter::run(interpreter),
        }
    }
}

mod chr;
mod close;
mod cvd;
mod data;
mod def_seg;
mod environ_fn;
mod environ_sub;
mod eof;
mod field;
mod get;
mod input;
mod instr;
mod kill;
mod lbound;
mod lcase;
mod left;
mod len;
mod line_input;
mod locate;
mod lset;
mod ltrim;
mod mid_fn;
mod mkd;
mod name;
mod open;
mod put;
mod read;
mod right;
mod rtrim;
mod str_fn;
mod string_fn;
mod ubound;
mod ucase;
mod val;
mod varptr;
mod view_print;
mod width;
