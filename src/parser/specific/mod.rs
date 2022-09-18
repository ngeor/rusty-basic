pub mod csv;
pub mod keyword_choice;
pub mod token_type_map;
pub mod try_from_token_type;
pub mod with_pos;

use std::fs::File;
use std::str::Chars;

use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Location, QError};
use crate::parser::base::and_pc::{AndDemandTrait, AndTrait};
use crate::parser::base::parsers::*;
use crate::parser::base::readers::{file_char_reader, string_char_reader};
use crate::parser::base::recognizers::*;
use crate::parser::base::tokenizers::*;
use crate::parser::expression::expression_node_p;
use crate::parser::specific::csv::csv_zero_or_more_allow_missing;
use crate::parser::{
    Expression, ExpressionNode, ExpressionNodes, Keyword, Statement, SORTED_KEYWORDS_STR,
};

/// specific module contains implementation that mirrors the base module
/// but it is specific to QBasic
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenType {
    Unknown,
    Eol,
    Whitespace,
    Digits,
    LParen,
    RParen,
    Colon,
    Semicolon,
    Comma,
    SingleQuote,
    DoubleQuote,
    Dot,
    Equals,
    Greater,
    Less,
    GreaterEquals,
    LessEquals,
    NotEquals,
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    ExclamationMark,
    Pound,
    DollarSign,
    Percent,
    Keyword,
    Identifier,
    OctDigits,
    HexDigits,
}

#[derive(Clone, Copy)]
enum OctOrHex {
    Oct,
    Hex,
}

impl From<OctOrHex> for char {
    fn from(value: OctOrHex) -> Self {
        match value {
            OctOrHex::Oct => 'O',
            OctOrHex::Hex => 'H',
        }
    }
}

impl OctOrHex {
    fn is_digit(&self, ch: char) -> bool {
        match self {
            Self::Oct => ch >= '0' && ch <= '7',
            Self::Hex => is_digit(ch) || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F'),
        }
    }
}

struct OctHexDigitsRecognizer {
    mode: OctOrHex,
}

impl Recognizer for OctHexDigitsRecognizer {
    fn recognize(&self, buffer: &str) -> Recognition {
        let mut chars = buffer.chars();
        match chars.next() {
            Some('&') => self.after_ampersand(&mut chars),
            _ => Recognition::Negative,
        }
    }
}

impl OctHexDigitsRecognizer {
    fn after_ampersand(&self, chars: &mut Chars) -> Recognition {
        match chars.next() {
            Some(ch) => {
                let needle: char = self.mode.into();
                if ch == needle {
                    self.after_radix(chars)
                } else {
                    Recognition::Negative
                }
            }
            None => Recognition::Partial,
        }
    }

    fn after_radix(&self, chars: &mut Chars) -> Recognition {
        // might be a negative sign, which will lead into Overflow,
        // but needs to be recognized anyway
        match chars.next() {
            Some(ch) => {
                if ch == '-' {
                    self.after_minus(chars)
                } else {
                    self.first_possible_digit(chars, ch)
                }
            }
            None => Recognition::Partial,
        }
    }

    fn after_minus(&self, chars: &mut Chars) -> Recognition {
        match chars.next() {
            Some(ch) => self.first_possible_digit(chars, ch),
            None => Recognition::Partial,
        }
    }

    fn first_possible_digit(&self, chars: &mut Chars, first: char) -> Recognition {
        if self.mode.is_digit(first) {
            self.next_possible_digit(chars)
        } else {
            Recognition::Negative
        }
    }

    fn next_possible_digit(&self, chars: &mut Chars) -> Recognition {
        match chars.next() {
            Some(ch) => {
                if self.mode.is_digit(ch) {
                    self.next_possible_digit(chars)
                } else {
                    Recognition::Negative
                }
            }
            None => Recognition::Positive,
        }
    }
}

pub fn create_recognizers() -> Vec<Box<dyn Recognizer>> {
    vec![
        Box::new(any_single_char_recognizer()),
        Box::new(single_new_line_recognizer()),
        Box::new(many_white_space_recognizer()),
        Box::new(many_digits_recognizer()),
        Box::new(single_char_recognizer('(')),
        Box::new(single_char_recognizer(')')),
        Box::new(single_char_recognizer(':')),
        Box::new(single_char_recognizer(',')),
        Box::new(single_char_recognizer('\'')),
        Box::new(single_char_recognizer('"')),
        Box::new(single_char_recognizer('.')),
        Box::new(single_char_recognizer('=')),
        Box::new(single_char_recognizer('>')),
        Box::new(single_char_recognizer('<')),
        Box::new(str_recognizer(">=")),
        Box::new(str_recognizer("<=")),
        Box::new(str_recognizer("<>")),
        Box::new(single_char_recognizer('+')),
        Box::new(single_char_recognizer('-')),
        Box::new(single_char_recognizer('*')),
        Box::new(single_char_recognizer('/')),
        Box::new(single_char_recognizer('&')),
        Box::new(single_char_recognizer('!')),
        Box::new(single_char_recognizer('#')),
        Box::new(single_char_recognizer('$')),
        Box::new(single_char_recognizer('%')),
        Box::new(keyword_recognizer(&SORTED_KEYWORDS_STR)),
        Box::new(leading_remaining_recognizer(is_letter, |ch| {
            is_letter(ch) || is_digit(ch)
        })),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Oct,
        }),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Hex,
        }),
    ]
}

pub fn create_file_tokenizer(input: File) -> impl Tokenizer {
    create_tokenizer(file_char_reader(input), create_recognizers())
}

#[cfg(test)]
pub fn create_string_tokenizer<T>(input: T) -> impl Tokenizer
where
    T: AsRef<[u8]>,
{
    create_tokenizer(string_char_reader(input), create_recognizers())
}

//
// KeywordParser
//

pub struct KeywordParser {
    keyword: Keyword,
}

impl TokenPredicate for KeywordParser {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Keyword as i32 && token.text == self.keyword.as_str()
    }
}

impl ErrorProvider for KeywordParser {
    fn provide_error(&self) -> QError {
        QError::SyntaxError(format!("Expected keyword {}", self.keyword))
    }
}

// TODO rename to keyword keep only one of those functions
pub fn keyword_p(keyword: Keyword) -> impl Parser<Output = Token> {
    KeywordParser { keyword }.parser()
}

// TODO rename to keyword_non_opt keep only one of those functions
pub fn keyword(keyword: Keyword) -> impl NonOptParser<Output = Token> {
    KeywordParser { keyword }.parser()
}

// TODO deprecate this
pub fn keyword_followed_by_whitespace_p(keyword: Keyword) -> impl Parser {
    keyword_p(keyword).and(whitespace())
}

// TODO rename to keyword_pair_opt
pub fn keyword_pair_p(first: Keyword, second: Keyword) -> impl Parser {
    keyword_p(first)
        .and_demand(whitespace())
        .and_demand(keyword(second))
}

// TODO rename to keyword_pair
pub fn demand_keyword_pair_p(first: Keyword, second: Keyword) -> impl NonOptParser {
    keyword(first)
        .and_demand(whitespace())
        .and_demand(keyword(second))
}

//
// TokenKindParser
//

pub struct TokenKindParser(TokenType);

impl TokenKindParser {
    pub fn new(token_type: TokenType) -> Self {
        Self(token_type)
    }
}

impl TokenPredicate for TokenKindParser {
    fn test(&self, token: &Token) -> bool {
        token.kind == self.0 as i32
    }
}

impl ErrorProvider for TokenKindParser {
    fn provide_error(&self) -> QError {
        // TODO use Display instead of Debug
        QError::SyntaxError(format!("Expected token of type {:?}", self.0))
    }
}

pub fn whitespace() -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser(TokenType::Whitespace).parser()
}

pub fn surrounded_by_opt_ws<P: Parser>(parser: P) -> impl Parser<Output = P::Output> {
    OptAndPC::new(whitespace(), parser)
        .and_opt(whitespace())
        .keep_middle()
}

// TODO deprecate this
pub fn item_p(ch: char) -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser(match ch {
        ',' => TokenType::Comma,
        '=' => TokenType::Equals,
        '$' => TokenType::DollarSign,
        '\'' => TokenType::SingleQuote,
        '-' => TokenType::Minus,
        '*' => TokenType::Star,
        '#' => TokenType::Pound,
        '.' => TokenType::Dot,
        ';' => TokenType::Semicolon,
        '>' => TokenType::Greater,
        '<' => TokenType::Less,
        ':' => TokenType::Colon,
        _ => panic!("not implemented {}", ch),
    })
    .parser()
}

//
// TODO Used only by COLOR and LOCATE, perhaps move elsewhere
//

/// Parses built-in subs with optional arguments
pub fn parse_built_in_sub_with_opt_args(
    keyword: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(keyword)
        .and_demand(csv_zero_or_more_allow_missing(expression_node_p()))
        .keep_right()
        .fn_map(move |opt_args| {
            Statement::BuiltInSubCall(built_in_sub, map_opt_args_to_flags(opt_args))
        })
}

/// Maps optional arguments to arguments, inserting a dummy first argument indicating
/// which arguments were present in the form of a bit mask.
fn map_opt_args_to_flags(args: Vec<Option<ExpressionNode>>) -> ExpressionNodes {
    let mut result: ExpressionNodes = vec![];
    let mut mask = 1;
    let mut flags = 0;
    for arg in args {
        if let Some(arg) = arg {
            flags |= mask;
            result.push(arg);
        }
        mask <<= 1;
    }
    result.insert(0, Expression::IntegerLiteral(flags).at(Location::start()));
    result
}

//
// In Parenthesis
//

pub fn in_parenthesis<P: NonOptParser>(parser: P) -> impl NonOptParser<Output = P::Output> {
    TokenKindParser(TokenType::LParen)
        .parser()
        .and_demand(parser)
        .and_demand(TokenKindParser(TokenType::RParen).parser())
        .keep_middle()
}

// TODO rename to opt
// TODO implementation is identical to above
pub fn in_parenthesis_p<P: NonOptParser>(parser: P) -> impl Parser<Output = P::Output> {
    TokenKindParser(TokenType::LParen)
        .parser()
        .and_demand(parser)
        .and_demand(TokenKindParser(TokenType::RParen).parser())
        .keep_middle()
}

// TODO deprecate this
pub fn identifier_without_dot_p() -> impl Parser<Output = Token> {
    TokenKindParser(TokenType::Identifier).parser()
}

pub struct LeadingWhitespace<P> {
    parser: P,
    needs_whitespace: bool,
}

impl<P> LeadingWhitespace<P> {
    pub fn new(parser: P, needs_whitespace: bool) -> Self {
        Self {
            parser,
            needs_whitespace,
        }
    }
}

impl<P> HasOutput for LeadingWhitespace<P>
where
    P: Parser,
{
    type Output = P::Output;
}

impl<P> Parser for LeadingWhitespace<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let opt_space = whitespace().parse(tokenizer)?;
        if self.needs_whitespace && opt_space.is_none() {
            Ok(None)
        } else {
            match self.parser.parse(tokenizer)? {
                Some(value) => Ok(Some(value)),
                None => {
                    if let Some(space) = opt_space {
                        tokenizer.unread(space);
                    }
                    Ok(None)
                }
            }
        }
    }
}

//
// MapErr
//

struct MapErrParser<P>(P, QError);

impl<P> HasOutput for MapErrParser<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<P> Parser for MapErrParser<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        self.0.parse(tokenizer).map_err(|_| self.1)
    }
}

impl<P> NonOptParser for MapErrParser<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer).map_err(|_| self.1)
    }
}

pub trait MapErrTrait {
    fn map_err(self, err: QError) -> MapErrParser<Self>
    where
        Self: Sized;
}

impl<S> MapErrTrait for S {
    fn map_err(self, err: QError) -> MapErrParser<Self> {
        MapErrParser(self, err)
    }
}

//
// Or Syntax Error
//

pub struct OrSyntaxError<'a, P>(P, &'a str);

impl<'a, P> HasOutput for OrSyntaxError<'a, P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<'a, P> NonOptParser for OrSyntaxError<'a, P>
where
    P: Parser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => Ok(value),
            None => Err(QError::syntax_error(self.1)),
        }
    }
}

pub trait OrSyntaxErrorTrait {
    fn or_syntax_error<'a>(self, msg: &str) -> OrSyntaxError<'a, Self>
    where
        Self: Sized;
}

impl<S> OrSyntaxErrorTrait for S {
    fn or_syntax_error<'a>(self, msg: &'a str) -> OrSyntaxError<'a, Self>
    where
        Self: Sized,
    {
        OrSyntaxError(self, msg)
    }
}
