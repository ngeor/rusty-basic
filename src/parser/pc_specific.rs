use crate::built_ins::BuiltInSub;
/// Parser combinators specific to this project (e.g. for keywords)
use crate::common::*;
use crate::parser::expression::expression_node_p;
use crate::parser::pc::*;
use crate::parser::types::*;
use std::marker::PhantomData;

// ========================================================
// Undo support
// ========================================================

impl<R: Reader<Item = char>> Undo<CaseInsensitiveString> for R {
    fn undo(self, s: CaseInsensitiveString) -> Self {
        let inner: String = s.into();
        self.undo(inner)
    }
}

impl<R: Reader<Item = char>> Undo<Name> for R {
    fn undo(self, n: Name) -> Self {
        match n {
            Name::Bare(b) => self.undo(b),
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => {
                let first = self.undo(qualifier);
                first.undo(bare_name)
            }
        }
    }
}

impl<R: Reader<Item = char>> Undo<(Keyword, String)> for R {
    fn undo(self, s: (Keyword, String)) -> Self {
        self.undo(s.1)
    }
}

// ========================================================
// Miscellaneous
// ========================================================

// Reads any identifier. Note that the result might be a keyword.
// An identifier must start with a letter and consists of letters, numbers and the dot.
crate::char_sequence_p!(
    IdentifierWithDot,
    identifier_with_dot,
    is_letter,
    is_letter_or_digit_or_dot
);

fn is_letter_or_digit_or_dot(ch: char) -> bool {
    is_letter(ch) || is_digit(ch) || ch == '.'
}

// Parses an identifier that does not include a dot.
// This might be a keyword.
crate::char_sequence_p!(
    IdentifierWithoutDot,
    identifier_without_dot_p,
    is_letter,
    is_letter_or_digit
);

fn is_letter_or_digit(ch: char) -> bool {
    is_letter(ch) || is_digit(ch)
}

/// Recognizes the given keyword.
pub fn keyword_p<R>(keyword: Keyword) -> impl Parser<R, Output = (Keyword, String)>
where
    R: Reader<Item = char>,
{
    string_p(keyword.as_str())
        .unless_followed_by(if_p(is_not_whole_keyword))
        .map(move |keyword_as_str| (keyword, keyword_as_str))
}

/// Recognizes the given keyword, followed by whitespace.
///
/// This is an optimization over `keyword_p`, which checks if the keyword is a whole word,
/// but then undoes the whitespace.
pub fn keyword_followed_by_whitespace_p<R>(
    keyword: Keyword,
) -> impl Parser<R, Output = (Keyword, String)>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    string_p(keyword.as_str())
        .and(SpaceAfterKeyword(PhantomData, keyword))
        .map(move |(keyword_as_str, _)| (keyword, keyword_as_str))
}

pub struct SpaceAfterKeyword<R>(PhantomData<R>, Keyword);

impl<R> Parser<R> for SpaceAfterKeyword<R>
where
    R: Reader<Item = char, Err = QError>,
{
    type Output = String;
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        // read first character
        let (reader, opt_ch) = reader.read()?;
        // if $, rollback everything
        // if ' ', ok
        // if None or sth else, error
        match opt_ch {
            Some(ch) => {
                if ch == ' ' {
                    // ok, continue reading while whitespace
                    let mut buf = String::new();
                    buf.push(' ');
                    let mut r = reader;
                    loop {
                        let (tmp, opt_ch) = r.read()?;
                        r = tmp;
                        if let Some(' ') = opt_ch {
                            buf.push(' ');
                        } else {
                            r = r.undo(opt_ch);
                            break;
                        }
                    }
                    Ok((r, Some(buf)))
                } else if is_not_whole_keyword(ch) {
                    // undo and abort
                    Ok((reader.undo_item(ch), None))
                } else {
                    Err((
                        reader.undo_item(ch),
                        QError::SyntaxError(format!("Expected: whitespace after {}", self.1)),
                    ))
                }
            }
            None => Err((
                reader,
                QError::SyntaxError(format!("Expected: whitespace after {}", self.1)),
            )),
        }
    }
}

fn is_not_whole_keyword(ch: char) -> bool {
    is_letter(ch) || is_digit(ch) || ch == '.' || ch == '$'
}

pub fn keyword_pair_p<R>(
    first: Keyword,
    second: Keyword,
) -> impl Parser<R, Output = (Keyword, String, Keyword, String)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(first)
        .and_demand(
            keyword_p(second).or_syntax_error(format!("Expected: {} after {}", second, first)),
        )
        .map(|((first_k, first_s), (second_k, second_s))| (first_k, first_s, second_k, second_s))
}

pub fn demand_keyword_pair_p<R>(
    first: Keyword,
    second: Keyword,
) -> impl Parser<R, Output = (Keyword, String, Keyword, String)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_pair_p(first, second).or_syntax_error(format!("Expected: {} {}", first, second))
}

pub fn keyword_choice_p<R>(keywords: &[Keyword]) -> KeywordChoice<R> {
    KeywordChoice {
        reader: PhantomData,
        keywords,
    }
}

pub struct KeywordChoice<'a, R> {
    reader: PhantomData<R>,
    keywords: &'a [Keyword],
}

impl<'a, R> Parser<R> for KeywordChoice<'a, R>
where
    R: Reader<Item = char>,
{
    type Output = (Keyword, String);
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        // collect characters we read
        let mut buf = String::new();

        // candidate keywords and their character representations
        let mut candidates: Vec<(Keyword, Vec<char>)> = self
            .keywords
            .iter()
            .map(|k| (*k, k.as_str().chars().collect()))
            .collect();

        // we need a mutable reader as we're on a loop
        let mut r = reader;

        // the character index of the character we're trying to match currently
        let mut char_index: usize = 0;

        while !candidates.is_empty() {
            let (tmp, opt_item) = r.read()?;
            r = tmp;

            if let Some(ch) = opt_item {
                buf.push(ch);

                let mut i = 0;
                while i < candidates.len() {
                    let (candidate_keyword, candidate_chars) = candidates.get(i).unwrap();
                    if Self::matches(candidate_chars, char_index, ch) {
                        // possibly found the entire keyword, check that it is followed by None, or symbol, except dollar sign
                        if candidate_chars.len() == char_index + 1 {
                            let (tmp, opt_separator) = r.read()?;
                            r = tmp.undo(opt_separator); // put it back
                            if Self::is_keyword_separator(opt_separator) {
                                // found it
                                return Ok((r, Some((*candidate_keyword, buf))));
                            } else {
                                // candidate may not stay
                                candidates.remove(i);
                            }
                        } else {
                            // candidate can stay
                            i += 1;
                        }
                    } else {
                        // remove candidate
                        candidates.remove(i);
                    }
                }

                char_index += 1;
            } else {
                // could've just said "break;" here
                candidates.clear();
            }
        }

        r = r.undo(buf);
        Ok((r, None))
    }
}

impl<'a, R> KeywordChoice<'a, R> {
    fn is_keyword_separator(opt_separator: Option<char>) -> bool {
        match opt_separator {
            None => true,
            Some(ch) => !is_not_whole_keyword(ch),
        }
    }

    fn matches(candidate_chars: &Vec<char>, idx: usize, ch: char) -> bool {
        match candidate_chars.get(idx) {
            Some(x) => x.eq_ignore_ascii_case(&ch),
            _ => false,
        }
    }
}

/// Parses open and closing parenthesis around the given source.
/// Additional whitespace is allowed inside the parenthesis.
/// If the parser returns `None`, the left parenthesis is undone.
pub fn in_parenthesis_p<R, S>(source: S) -> impl Parser<R, Output = S::Output>
where
    R: Reader<Item = char, Err = QError> + 'static,
    S: Parser<R> + 'static,
{
    lparen_opt_ws_p()
        .and(source)
        .and_demand(
            item_p(')')
                .preceded_by_opt_ws()
                .or_syntax_error("Expected: closing parenthesis"),
        )
        .keep_middle()
}

crate::char_sequence_p!(
    LParenOptWhitespace,
    lparen_opt_ws_p,
    is_left_parenthesis,
    is_whitespace
);
fn is_left_parenthesis(ch: char) -> bool {
    ch == '('
}

/// Offers chaining methods for parsers specific to rusty_basic.
pub trait PcSpecific<R>: Parser<R> + Sized
where
    R: Reader<Item = char, Err = QError>,
{
    /// Throws a syntax error if this parser returns `None`.
    fn or_syntax_error<S: AsRef<str>>(self, msg: S) -> OrThrowVal<Self, QError> {
        OrThrowVal::new(self, QError::syntax_error(msg))
    }

    /// Parses one or more items provided by the given source, separated by commas.
    /// Trailing commas are not allowed. Space is allowed around commas.
    fn csv(self) -> OneOrMoreDelimited<Self, SurroundedByOptWhitespace<Item<R>>, QError> {
        self.one_or_more_delimited_by(
            item_p(',').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing comma"),
        )
    }

    /// Parses one or more items provided by the given source, separated by commas.
    /// Items are allowed to be missing.
    /// Trailing commas are not allowed. Space is allowed around commas.
    fn csv_allow_missing(
        self,
    ) -> OneOrMoreDelimitedAllowMissing<Self, SurroundedByOptWhitespace<Item<R>>, QError> {
        self.one_or_more_delimited_by_allow_missing(
            item_p(',').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing comma"),
        )
    }
}

impl<R: Reader<Item = char, Err = QError>, T> PcSpecific<R> for T where T: Parser<R> {}

/// Parses built-in subs with optional arguments
pub fn parse_built_in_sub_with_opt_args<R>(
    keyword: Keyword,
    built_in_sub: BuiltInSub,
) -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(keyword)
        .and_opt(expression_node_p().csv_allow_missing())
        .keep_right()
        .map(move |opt_args| {
            Statement::BuiltInSubCall(built_in_sub, map_opt_args_to_flags(opt_args))
        })
}

/// Maps optional arguments to arguments, inserting a dummy first argument indicating
/// which arguments were present in the form of a bit mask.
pub fn map_opt_args_to_flags(args: Option<Vec<Option<ExpressionNode>>>) -> ExpressionNodes {
    let mut result: ExpressionNodes = vec![];
    let mut mask = 1;
    let mut flags = 0;
    if let Some(args) = args {
        for arg in args {
            if let Some(arg) = arg {
                flags |= mask;
                result.push(arg);
            }
            mask <<= 1;
        }
    }
    result.insert(0, Expression::IntegerLiteral(flags).at(Location::start()));
    result
}
