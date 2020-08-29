use crate::common::CmpIgnoreAsciiCase;
use std::fmt::Display;
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
    /// FOR
    For,
    /// FUNCTION
    Function,
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
    /// LINE
    Line,
    /// LONG
    Long,
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
    /// READ
    Read,
    /// SELECT
    Select,
    /// SINGLE
    Single,
    /// STEP
    Step,
    /// STRING
    String_,
    /// SUB
    Sub,
    /// THEN
    Then,
    /// TO
    To,
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
const STR_CONST: &str = "CONST";
const STR_DECLARE: &str = "DECLARE";
const STR_DEFDBL: &str = "DEFDBL";
const STR_DEFINT: &str = "DEFINT";
const STR_DEFLNG: &str = "DEFLNG";
const STR_DEFSNG: &str = "DEFSNG";
const STR_DEFSTR: &str = "DEFSTR";
const STR_DIM: &str = "DIM";
const STR_DOUBLE: &str = "DOUBLE";
const STR_ELSE: &str = "ELSE";
const STR_ELSEIF: &str = "ELSEIF";
const STR_END: &str = "END";
const STR_ERROR: &str = "ERROR";
const STR_FOR: &str = "FOR";
const STR_FUNCTION: &str = "FUNCTION";
const STR_GOTO: &str = "GOTO";
const STR_IF: &str = "IF";
const STR_INPUT: &str = "INPUT";
const STR_INTEGER: &str = "INTEGER";
const STR_IS: &str = "IS";
const STR_LINE: &str = "LINE";
const STR_LONG: &str = "LONG";
const STR_NAME: &str = "NAME";
const STR_NEXT: &str = "NEXT";
const STR_NOT: &str = "NOT";
const STR_ON: &str = "ON";
const STR_OPEN: &str = "OPEN";
const STR_OR: &str = "OR";
const STR_OUTPUT: &str = "OUTPUT";
const STR_READ: &str = "READ";
const STR_SELECT: &str = "SELECT";
const STR_SINGLE: &str = "SINGLE";
const STR_STEP: &str = "STEP";
const STR_STRING: &str = "STRING";
const STR_SUB: &str = "SUB";
const STR_THEN: &str = "THEN";
const STR_TO: &str = "TO";
const STR_WEND: &str = "WEND";
const STR_WHILE: &str = "WHILE";

const SORTED_KEYWORDS_STR: [&str; 44] = [
    STR_ACCESS,
    STR_AND,
    STR_APPEND,
    STR_AS,
    STR_CASE,
    STR_CONST,
    STR_DECLARE,
    STR_DEFDBL,
    STR_DEFINT,
    STR_DEFLNG,
    STR_DEFSNG,
    STR_DEFSTR,
    STR_DIM,
    STR_DOUBLE,
    STR_ELSE,
    STR_ELSEIF,
    STR_END,
    STR_ERROR,
    STR_FOR,
    STR_FUNCTION,
    STR_GOTO,
    STR_IF,
    STR_INPUT,
    STR_INTEGER,
    STR_IS,
    STR_LINE,
    STR_LONG,
    STR_NAME,
    STR_NEXT,
    STR_NOT,
    STR_ON,
    STR_OPEN,
    STR_OR,
    STR_OUTPUT,
    STR_READ,
    STR_SELECT,
    STR_SINGLE,
    STR_STEP,
    STR_STRING,
    STR_SUB,
    STR_THEN,
    STR_TO,
    STR_WEND,
    STR_WHILE,
];

const SORTED_KEYWORDS: [Keyword; 44] = [
    Keyword::Access,
    Keyword::And,
    Keyword::Append,
    Keyword::As,
    Keyword::Case,
    Keyword::Const,
    Keyword::Declare,
    Keyword::DefDbl,
    Keyword::DefInt,
    Keyword::DefLng,
    Keyword::DefSng,
    Keyword::DefStr,
    Keyword::Dim,
    Keyword::Double,
    Keyword::Else,
    Keyword::ElseIf,
    Keyword::End,
    Keyword::Error,
    Keyword::For,
    Keyword::Function,
    Keyword::GoTo,
    Keyword::If,
    Keyword::Input,
    Keyword::Integer,
    Keyword::Is,
    Keyword::Line,
    Keyword::Long,
    Keyword::Name,
    Keyword::Next,
    Keyword::Not,
    Keyword::On,
    Keyword::Open,
    Keyword::Or,
    Keyword::Output,
    Keyword::Read,
    Keyword::Select,
    Keyword::Single,
    Keyword::Step,
    Keyword::String_,
    Keyword::Sub,
    Keyword::Then,
    Keyword::To,
    Keyword::Wend,
    Keyword::While,
];

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Access => STR_ACCESS.fmt(f),
            Self::And => STR_AND.fmt(f),
            Self::Append => STR_APPEND.fmt(f),
            Self::As => STR_AS.fmt(f),
            Self::Case => STR_CASE.fmt(f),
            Self::Const => STR_CONST.fmt(f),
            Self::Declare => STR_DECLARE.fmt(f),
            Self::DefDbl => STR_DEFDBL.fmt(f),
            Self::DefInt => STR_DEFINT.fmt(f),
            Self::DefLng => STR_DEFLNG.fmt(f),
            Self::DefSng => STR_DEFSNG.fmt(f),
            Self::DefStr => STR_DEFSTR.fmt(f),
            Self::Dim => STR_DIM.fmt(f),
            Self::Double => STR_DOUBLE.fmt(f),
            Self::Else => STR_ELSE.fmt(f),
            Self::ElseIf => STR_ELSEIF.fmt(f),
            Self::End => STR_END.fmt(f),
            Self::Error => STR_ERROR.fmt(f),
            Self::For => STR_FOR.fmt(f),
            Self::Function => STR_FUNCTION.fmt(f),
            Self::GoTo => STR_GOTO.fmt(f),
            Self::If => STR_IF.fmt(f),
            Self::Input => STR_INPUT.fmt(f),
            Self::Integer => STR_INTEGER.fmt(f),
            Self::Is => STR_IS.fmt(f),
            Self::Line => STR_LINE.fmt(f),
            Self::Long => STR_LONG.fmt(f),
            Self::Name => STR_NAME.fmt(f),
            Self::Next => STR_NEXT.fmt(f),
            Self::Not => STR_NOT.fmt(f),
            Self::On => STR_ON.fmt(f),
            Self::Open => STR_OPEN.fmt(f),
            Self::Or => STR_OR.fmt(f),
            Self::Output => STR_OUTPUT.fmt(f),
            Self::Read => STR_READ.fmt(f),
            Self::Select => STR_SELECT.fmt(f),
            Self::Single => STR_SINGLE.fmt(f),
            Self::Step => STR_STEP.fmt(f),
            Self::String_ => STR_STRING.fmt(f),
            Self::Sub => STR_SUB.fmt(f),
            Self::Then => STR_THEN.fmt(f),
            Self::To => STR_TO.fmt(f),
            Self::Wend => STR_WEND.fmt(f),
            Self::While => STR_WHILE.fmt(f),
        }
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