pub mod csv;
pub mod in_parenthesis;
pub mod keyword_choice;
pub mod token_type_map;
pub mod try_from_token_type;
pub mod whitespace;
pub mod with_pos;

use std::convert::TryFrom;
use std::fs::File;
use std::str::Chars;

use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Location, QError};
use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::parsers::*;
use crate::parser::base::readers::file_char_reader;
use crate::parser::base::recognizers::*;
use crate::parser::base::tokenizers::*;
use crate::parser::expression::expression_node_p;
use crate::parser::specific::csv::csv_zero_or_more_allow_missing;
use crate::parser::specific::whitespace::WhitespaceTrait;
use crate::parser::{
    Expression, ExpressionNode, ExpressionNodes, Keyword, Statement, SORTED_KEYWORDS_STR,
};

/// specific module contains implementation that mirrors the base module
/// but it is specific to QBasic
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenType {
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
    // keyword needs to be before Identifier
    Keyword,
    Identifier,
    OctDigits,
    HexDigits,

    // unknown must be last
    Unknown,
}

impl TryFrom<i32> for TokenType {
    type Error = QError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let all_tokens = [
            TokenType::Eol,
            TokenType::Whitespace,
            TokenType::Digits,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::Colon,
            TokenType::Semicolon,
            TokenType::Comma,
            TokenType::SingleQuote,
            TokenType::DoubleQuote,
            TokenType::Dot,
            TokenType::Equals,
            TokenType::Greater,
            TokenType::Less,
            TokenType::GreaterEquals,
            TokenType::LessEquals,
            TokenType::NotEquals,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::Ampersand,
            TokenType::ExclamationMark,
            TokenType::Pound,
            TokenType::DollarSign,
            TokenType::Percent,
            TokenType::Keyword,
            TokenType::Identifier,
            TokenType::OctDigits,
            TokenType::HexDigits,
            TokenType::Unknown,
        ];
        if value >= 0 && value < all_tokens.len() as i32 {
            Ok(all_tokens[value as usize])
        } else {
            Err(QError::InternalError(format!(
                "Token index {} out of bounds",
                value
            )))
        }
    }
}

impl TryFrom<TokenType> for char {
    type Error = QError;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        match value {
            TokenType::Semicolon => Ok(';'),
            _ => Err(QError::InternalError(format!("not implemented"))),
        }
    }
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
        Box::new(single_new_line_recognizer()),
        Box::new(many_white_space_recognizer()),
        Box::new(many_digits_recognizer()),
        Box::new(single_char_recognizer('(')),
        Box::new(single_char_recognizer(')')),
        Box::new(single_char_recognizer(':')),
        Box::new(single_char_recognizer(';')),
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
            is_letter(ch) || is_digit(ch) || ch == '.'
        })),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Oct,
        }),
        Box::new(OctHexDigitsRecognizer {
            mode: OctOrHex::Hex,
        }),
        Box::new(any_single_char_recognizer()),
    ]
}

pub fn create_file_tokenizer(input: File) -> impl Tokenizer {
    create_tokenizer(file_char_reader(input), create_recognizers())
}

#[cfg(test)]
use crate::parser::base::readers::string_char_reader;
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

struct KeywordParser {
    keyword: Keyword,
}

impl HasOutput for KeywordParser {
    type Output = Token;
}

impl Parser for KeywordParser {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.keyword == token {
                    // check for trailing dollar sign
                    match tokenizer.read()? {
                        Some(follow_up) => {
                            if follow_up.kind == TokenType::DollarSign as i32 {
                                tokenizer.unread(follow_up);
                                tokenizer.unread(token);
                                Ok(None)
                            } else {
                                tokenizer.unread(follow_up);
                                Ok(Some(token))
                            }
                        }
                        None => Ok(Some(token)),
                    }
                } else {
                    tokenizer.unread(token);
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl NonOptParser for KeywordParser {
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(token) => {
                if self.keyword == token {
                    // check for trailing dollar sign
                    match tokenizer.read()? {
                        Some(follow_up) => {
                            if follow_up.kind == TokenType::DollarSign as i32 {
                                tokenizer.unread(follow_up);
                                tokenizer.unread(token);
                                Err(QError::SyntaxError(format!("Expected: {}", self.keyword)))
                            } else {
                                tokenizer.unread(follow_up);
                                Ok(token)
                            }
                        }
                        None => Ok(token),
                    }
                } else {
                    tokenizer.unread(token);
                    Err(QError::SyntaxError(format!("Expected: {}", self.keyword)))
                }
            }
            None => Err(QError::InputPastEndOfFile),
        }
    }
}

pub fn keyword(keyword: Keyword) -> impl Parser<Output = Token> + NonOptParser<Output = Token> {
    KeywordParser { keyword }
}

// TODO deprecate this
pub fn keyword_followed_by_whitespace_p(k: Keyword) -> impl Parser {
    keyword(k).followed_by_req_ws()
}

pub fn keyword_pair(first: Keyword, second: Keyword) -> impl Parser + NonOptParser {
    keyword(first)
        .followed_by_req_ws()
        .and_demand(keyword(second))
}

//
// TokenKindParser
//

pub struct TokenKindParser {
    token_type: TokenType,
}

impl TokenKindParser {
    pub fn new(token_type: TokenType) -> Self {
        Self { token_type }
    }
}

impl TokenPredicate for TokenKindParser {
    fn test(&self, token: &Token) -> bool {
        token.kind == self.token_type as i32
    }
}

impl ErrorProvider for TokenKindParser {
    fn provide_error(&self) -> QError {
        match char::try_from(self.token_type) {
            Ok(ch) => QError::SyntaxError(format!("Expected: {}", ch)),
            _ => {
                // TODO use Display instead of Debug
                QError::SyntaxError(format!("Expected: token of type {:?}", self.token_type))
            }
        }
    }
}

// TODO deprecate this
pub fn item_p(ch: char) -> TokenPredicateParser<TokenKindParser> {
    TokenKindParser::new(match ch {
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
// MapErr
//

pub struct MapErrParser<P>(P, QError);

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
        self.0.parse(tokenizer).map_err(|_| self.1.clone())
    }
}

impl<P> NonOptParser for MapErrParser<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        self.0.parse_non_opt(tokenizer).map_err(|_| self.1.clone())
    }
}

// TODO remove MapErrTrait

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
// OrError
//

pub struct OrError<P>(P, QError);

impl<P> HasOutput for OrError<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<P> NonOptParser for OrError<P>
where
    P: Parser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match self.0.parse(tokenizer)? {
            Some(value) => Ok(value),
            _ => Err(self.1.clone()),
        }
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

pub trait OrErrorTrait
where
    Self: Sized,
{
    fn or_error(self, err: QError) -> OrError<Self>;

    fn or_syntax_error(self, msg: &str) -> OrSyntaxError<Self>;
}

impl<S> OrErrorTrait for S {
    fn or_error(self, err: QError) -> OrError<Self> {
        OrError(self, err)
    }

    fn or_syntax_error(self, msg: &str) -> OrSyntaxError<Self> {
        OrSyntaxError(self, msg)
    }
}

// IdentifierOrKeyword

struct IdentifierOrKeyword;

impl TokenPredicate for IdentifierOrKeyword {
    fn test(&self, token: &Token) -> bool {
        token.kind == TokenType::Keyword as i32 || token.kind == TokenType::Identifier as i32
    }
}

pub fn identifier_or_keyword() -> impl Parser<Output = Token> {
    IdentifierOrKeyword.parser()
}

struct IdentifierOrKeywordWithoutDot;

impl TokenPredicate for IdentifierOrKeywordWithoutDot {
    fn test(&self, token: &Token) -> bool {
        (token.kind == TokenType::Keyword as i32 || token.kind == TokenType::Identifier as i32)
            && !token.text.contains('.')
    }
}

pub fn identifier_or_keyword_without_dot() -> impl Parser<Output = Token> {
    IdentifierOrKeywordWithoutDot.parser()
}
