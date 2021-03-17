use crate::common::CmpIgnoreAsciiCase;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Keyword {
    /// ACCESS
    Access,
    /// AND
    And,
    /// APPEND
    Append,
    /// AS
    As,
    /// CASE
    Case,
    /// CLOSE
    Close,
    /// CONST
    Const,
    /// DECLARE
    Declare,
    /// DEFDBL
    DefDbl,
    /// DEFINT
    DefInt,
    /// DEFLNG
    DefLng,
    /// DEFSNG
    DefSng,
    /// DEFSTR
    DefStr,
    /// DIM
    Dim,
    // DO
    Do,
    /// DOUBLE
    Double,
    /// ELSE
    Else,
    /// ELSEIF
    ElseIf,
    /// END
    End,
    /// ERROR
    Error,
    /// EXIT
    Exit,
    /// FIELD
    Field,
    /// FOR
    For,
    /// FUNCTION
    Function,
    /// GET
    Get,
    /// GOSUB
    GoSub,
    /// GOTO
    GoTo,
    /// IF
    If,
    /// INPUT
    Input,
    /// INTEGER
    Integer,
    /// IS
    Is,
    /// LEN
    Len,
    /// LINE
    Line,
    /// LONG
    Long,
    /// LOOP
    Loop,
    /// LPRINT
    LPrint,
    /// LSET
    LSet,
    /// MOD
    Mod,
    /// NAME
    Name,
    /// NEXT
    Next,
    /// NOT
    Not,
    /// ON
    On,
    /// OPEN
    Open,
    /// OR
    Or,
    /// OUTPUT
    Output,
    /// PRINT
    Print,
    /// PUT
    Put,
    /// RANDOM
    Random,
    /// READ
    Read,
    /// RESUME
    Resume,
    /// RETURN
    Return,
    /// SELECT
    Select,
    /// SHARED
    Shared,
    /// SINGLE
    Single,
    /// STATIC
    Static,
    /// STEP
    Step,
    /// STRING
    String_,
    /// SUB
    Sub,
    /// SYSTEM
    System,
    /// THEN
    Then,
    /// TO
    To,
    /// TYPE
    Type,
    /// UNTIL
    Until,
    /// USING
    Using,
    /// VIEW
    View,
    /// WEND
    Wend,
    /// WHILE
    While,
}

const STR_ACCESS: &str = "ACCESS";
const STR_AND: &str = "AND";
const STR_APPEND: &str = "APPEND";
const STR_AS: &str = "AS";
const STR_CASE: &str = "CASE";
const STR_CLOSE: &str = "CLOSE";
const STR_CONST: &str = "CONST";
const STR_DECLARE: &str = "DECLARE";
const STR_DEFDBL: &str = "DEFDBL";
const STR_DEFINT: &str = "DEFINT";
const STR_DEFLNG: &str = "DEFLNG";
const STR_DEFSNG: &str = "DEFSNG";
const STR_DEFSTR: &str = "DEFSTR";
const STR_DIM: &str = "DIM";
const STR_DO: &str = "DO";
const STR_DOUBLE: &str = "DOUBLE";
const STR_ELSE: &str = "ELSE";
const STR_ELSEIF: &str = "ELSEIF";
const STR_END: &str = "END";
const STR_ERROR: &str = "ERROR";
const STR_EXIT: &str = "EXIT";
const STR_FIELD: &str = "FIELD";
const STR_FOR: &str = "FOR";
const STR_FUNCTION: &str = "FUNCTION";
const STR_GET: &str = "GET";
const STR_GO_SUB: &str = "GOSUB";
const STR_GOTO: &str = "GOTO";
const STR_IF: &str = "IF";
const STR_INPUT: &str = "INPUT";
const STR_INTEGER: &str = "INTEGER";
const STR_IS: &str = "IS";
const STR_LEN: &str = "LEN";
const STR_LINE: &str = "LINE";
const STR_LONG: &str = "LONG";
const STR_LOOP: &str = "LOOP";
const STR_LPRINT: &str = "LPRINT";
const STR_LSET: &str = "LSET";
const STR_MOD: &str = "MOD";
const STR_NAME: &str = "NAME";
const STR_NEXT: &str = "NEXT";
const STR_NOT: &str = "NOT";
const STR_ON: &str = "ON";
const STR_OPEN: &str = "OPEN";
const STR_OR: &str = "OR";
const STR_OUTPUT: &str = "OUTPUT";
const STR_PRINT: &str = "PRINT";
const STR_PUT: &str = "PUT";
const STR_RANDOM: &str = "RANDOM";
const STR_READ: &str = "READ";
const STR_RESUME: &str = "RESUME";
const STR_RETURN: &str = "RETURN";
const STR_SELECT: &str = "SELECT";
const STR_SHARED: &str = "SHARED";
const STR_SINGLE: &str = "SINGLE";
const STR_STATIC: &str = "STATIC";
const STR_STEP: &str = "STEP";
const STR_STRING: &str = "STRING";
const STR_SUB: &str = "SUB";
const STR_SYSTEM: &str = "SYSTEM";
const STR_THEN: &str = "THEN";
const STR_TO: &str = "TO";
const STR_TYPE: &str = "TYPE";
const STR_UNTIL: &str = "UNTIL";
const STR_USING: &str = "USING";
const STR_VIEW: &str = "VIEW";
const STR_WEND: &str = "WEND";
const STR_WHILE: &str = "WHILE";

const SORTED_KEYWORDS_STR: [&str; 67] = [
    STR_ACCESS,
    STR_AND,
    STR_APPEND,
    STR_AS,
    STR_CASE,
    STR_CLOSE,
    STR_CONST,
    STR_DECLARE,
    STR_DEFDBL,
    STR_DEFINT,
    STR_DEFLNG,
    STR_DEFSNG,
    STR_DEFSTR,
    STR_DIM,
    STR_DO,
    STR_DOUBLE,
    STR_ELSE,
    STR_ELSEIF,
    STR_END,
    STR_ERROR,
    STR_EXIT,
    STR_FIELD,
    STR_FOR,
    STR_FUNCTION,
    STR_GET,
    STR_GO_SUB,
    STR_GOTO,
    STR_IF,
    STR_INPUT,
    STR_INTEGER,
    STR_IS,
    STR_LEN,
    STR_LINE,
    STR_LONG,
    STR_LOOP,
    STR_LPRINT,
    STR_LSET,
    STR_MOD,
    STR_NAME,
    STR_NEXT,
    STR_NOT,
    STR_ON,
    STR_OPEN,
    STR_OR,
    STR_OUTPUT,
    STR_PRINT,
    STR_PUT,
    STR_RANDOM,
    STR_READ,
    STR_RESUME,
    STR_RETURN,
    STR_SELECT,
    STR_SHARED,
    STR_SINGLE,
    STR_STATIC,
    STR_STEP,
    STR_STRING,
    STR_SUB,
    STR_SYSTEM,
    STR_THEN,
    STR_TO,
    STR_TYPE,
    STR_UNTIL,
    STR_USING,
    STR_VIEW,
    STR_WEND,
    STR_WHILE,
];

const SORTED_KEYWORDS: [Keyword; 67] = [
    Keyword::Access,
    Keyword::And,
    Keyword::Append,
    Keyword::As,
    Keyword::Case,
    Keyword::Close,
    Keyword::Const,
    Keyword::Declare,
    Keyword::DefDbl,
    Keyword::DefInt,
    Keyword::DefLng,
    Keyword::DefSng,
    Keyword::DefStr,
    Keyword::Dim,
    Keyword::Do,
    Keyword::Double,
    Keyword::Else,
    Keyword::ElseIf,
    Keyword::End,
    Keyword::Error,
    Keyword::Exit,
    Keyword::Field,
    Keyword::For,
    Keyword::Function,
    Keyword::Get,
    Keyword::GoSub,
    Keyword::GoTo,
    Keyword::If,
    Keyword::Input,
    Keyword::Integer,
    Keyword::Is,
    Keyword::Len,
    Keyword::Line,
    Keyword::Long,
    Keyword::Loop,
    Keyword::LPrint,
    Keyword::LSet,
    Keyword::Mod,
    Keyword::Name,
    Keyword::Next,
    Keyword::Not,
    Keyword::On,
    Keyword::Open,
    Keyword::Or,
    Keyword::Output,
    Keyword::Print,
    Keyword::Put,
    Keyword::Random,
    Keyword::Read,
    Keyword::Resume,
    Keyword::Return,
    Keyword::Select,
    Keyword::Shared,
    Keyword::Single,
    Keyword::Static,
    Keyword::Step,
    Keyword::String_,
    Keyword::Sub,
    Keyword::System,
    Keyword::Then,
    Keyword::To,
    Keyword::Type,
    Keyword::Until,
    Keyword::Using,
    Keyword::View,
    Keyword::Wend,
    Keyword::While,
];

impl Keyword {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Access => STR_ACCESS,
            Self::And => STR_AND,
            Self::Append => STR_APPEND,
            Self::As => STR_AS,
            Self::Case => STR_CASE,
            Self::Close => STR_CLOSE,
            Self::Const => STR_CONST,
            Self::Declare => STR_DECLARE,
            Self::DefDbl => STR_DEFDBL,
            Self::DefInt => STR_DEFINT,
            Self::DefLng => STR_DEFLNG,
            Self::DefSng => STR_DEFSNG,
            Self::DefStr => STR_DEFSTR,
            Self::Dim => STR_DIM,
            Self::Do => STR_DO,
            Self::Double => STR_DOUBLE,
            Self::Else => STR_ELSE,
            Self::ElseIf => STR_ELSEIF,
            Self::End => STR_END,
            Self::Error => STR_ERROR,
            Self::Exit => STR_EXIT,
            Self::Field => STR_FIELD,
            Self::For => STR_FOR,
            Self::Function => STR_FUNCTION,
            Self::Get => STR_GET,
            Self::GoSub => STR_GO_SUB,
            Self::GoTo => STR_GOTO,
            Self::If => STR_IF,
            Self::Input => STR_INPUT,
            Self::Integer => STR_INTEGER,
            Self::Is => STR_IS,
            Self::Len => STR_LEN,
            Self::Line => STR_LINE,
            Self::Long => STR_LONG,
            Self::Loop => STR_LOOP,
            Self::LPrint => STR_LPRINT,
            Self::LSet => STR_LSET,
            Self::Mod => STR_MOD,
            Self::Name => STR_NAME,
            Self::Next => STR_NEXT,
            Self::Not => STR_NOT,
            Self::On => STR_ON,
            Self::Open => STR_OPEN,
            Self::Or => STR_OR,
            Self::Output => STR_OUTPUT,
            Self::Print => STR_PRINT,
            Self::Put => STR_PUT,
            Self::Random => STR_RANDOM,
            Self::Read => STR_READ,
            Self::Resume => STR_RESUME,
            Self::Return => STR_RETURN,
            Self::Select => STR_SELECT,
            Self::Shared => STR_SHARED,
            Self::Single => STR_SINGLE,
            Self::Static => STR_STATIC,
            Self::Step => STR_STEP,
            Self::String_ => STR_STRING,
            Self::Sub => STR_SUB,
            Self::System => STR_SYSTEM,
            Self::Then => STR_THEN,
            Self::To => STR_TO,
            Self::Type => STR_TYPE,
            Self::Until => STR_UNTIL,
            Self::Using => STR_USING,
            Self::View => STR_VIEW,
            Self::Wend => STR_WEND,
            Self::While => STR_WHILE,
        }
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Keyword {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        SORTED_KEYWORDS_STR
            .binary_search_by(|p| CmpIgnoreAsciiCase::compare_ignore_ascii_case(*p, s))
            .map(|idx| SORTED_KEYWORDS[idx])
            .map_err(|_| format!("Not a keyword: {}", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_sanity_checks() {
        // equal size of the two arrays
        assert_eq!(SORTED_KEYWORDS.len(), SORTED_KEYWORDS_STR.len());
        for i in 0..SORTED_KEYWORDS.len() {
            // canonical form is uppercase
            assert_eq!(
                SORTED_KEYWORDS_STR[i].to_uppercase(),
                SORTED_KEYWORDS_STR[i]
            );
            // can convert keyword to string
            assert_eq!(SORTED_KEYWORDS[i].to_string(), SORTED_KEYWORDS_STR[i]);
            // can parse string to keyword
            assert_eq!(SORTED_KEYWORDS[i], SORTED_KEYWORDS_STR[i].parse().unwrap());
            // can parse lowercase string to keyword
            assert_eq!(
                SORTED_KEYWORDS[i],
                SORTED_KEYWORDS_STR[i].to_lowercase().parse().unwrap()
            );
        }
        // sort order is correct
        for i in 1..SORTED_KEYWORDS.len() {
            assert!(SORTED_KEYWORDS_STR[i] > SORTED_KEYWORDS_STR[i - 1]);
        }
    }

    #[test]
    fn test_from_string_not_a_keyword() {
        assert_eq!("Not a keyword: foo", "foo".parse::<Keyword>().unwrap_err());
    }
}
