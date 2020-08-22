use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

use crate::char_reader::{
    and_ng, and_skip_first, csv_one_or_more, map_ng, map_or_undo, or_ng, read_any_keyword,
    read_any_letter, read_some_letter, try_read_char, with_some_whitespace_between, EolReader,
    MapOrUndo,
};
use crate::parser::types::*;
use std::io::BufRead;

pub fn def_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<DefType, QErrorNode>)> {
    map_ng(
        with_some_whitespace_between(def_keyword(), letter_ranges(), || {
            QError::SyntaxError("Expected letter ranges".to_string())
        }),
        |(l, r)| DefType::new(l, r),
    )
}

fn def_keyword<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TypeQualifier, QErrorNode>)> {
    map_or_undo(read_any_keyword(), |(k, s)| match k {
        Keyword::DefDbl => MapOrUndo::Ok(TypeQualifier::HashDouble),
        Keyword::DefInt => MapOrUndo::Ok(TypeQualifier::PercentInteger),
        Keyword::DefLng => MapOrUndo::Ok(TypeQualifier::AmpersandLong),
        Keyword::DefSng => MapOrUndo::Ok(TypeQualifier::BangSingle),
        Keyword::DefStr => MapOrUndo::Ok(TypeQualifier::DollarString),
        _ => MapOrUndo::Undo((k, s)),
    })
}

fn letter_ranges<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Vec<LetterRange>, QErrorNode>)> {
    csv_one_or_more(letter_range(), || {
        QError::SyntaxError("Expected letter range".to_string())
    })
}

fn letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<LetterRange, QErrorNode>)> {
    or_ng(
        two_letter_range(), // needs to be first because the second will match too
        single_letter_range(),
    )
}

fn single_letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<LetterRange, QErrorNode>)> {
    map_ng(read_any_letter(), |l| LetterRange::Single(l))
}

fn two_letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<LetterRange, QErrorNode>)> {
    map_ng(
        and_ng(
            read_any_letter(),
            and_skip_first(
                try_read_char('-'),
                read_some_letter(|| QError::SyntaxError("Expected letter after dash".to_string())),
            ),
        ),
        |(l, r)| LetterRange::Range(l, r),
    )
}

#[deprecated]
pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, QErrorNode> {
    let next = lexer.peek_ref_dp()?;
    if next.is_none() {
        return Ok(None);
    }
    let opt_qualifier = match next.unwrap().as_ref() {
        Lexeme::Keyword(keyword, _) => match keyword {
            Keyword::DefDbl => Some(TypeQualifier::HashDouble),
            Keyword::DefInt => Some(TypeQualifier::PercentInteger),
            Keyword::DefLng => Some(TypeQualifier::AmpersandLong),
            Keyword::DefSng => Some(TypeQualifier::BangSingle),
            Keyword::DefStr => Some(TypeQualifier::DollarString),
            _ => None,
        },
        _ => None,
    };
    if opt_qualifier.is_none() {
        return Ok(None);
    }

    let pos = lexer.read()?.pos(); // read DEF* keyword
    read_whitespace(lexer, "Expected whitespace after DEF* keyword")?;
    let mut ranges: Vec<LetterRange> = vec![];
    const STATE_INITIAL: u8 = 0;
    const STATE_FIRST_LETTER: u8 = 1;
    const STATE_DASH: u8 = 2;
    const STATE_SECOND_LETTER: u8 = 3;
    const STATE_COMMA: u8 = 4;
    const STATE_EOL: u8 = 5;
    let mut state = STATE_INITIAL;
    let mut first_letter = ' ';
    let mut second_letter = ' ';

    lexer.begin_transaction();
    while state != STATE_EOL {
        skip_whitespace(lexer)?;
        // TODO add helper for the peek_dp / read_dp pattern
        match lexer.peek_ref_dp()? {
            Some(Locatable {
                element: Lexeme::Word(w),
                pos,
            }) => {
                if w.len() != 1 {
                    return Err(QError::SyntaxError("Expected single character".to_string()))
                        .with_err_at(*pos);
                }
                if state == STATE_INITIAL || state == STATE_COMMA {
                    first_letter = w.chars().next().unwrap();
                    state = STATE_FIRST_LETTER;
                    lexer.read_dp()?;
                } else if state == STATE_DASH {
                    second_letter = w.chars().next().unwrap();
                    if first_letter > second_letter {
                        return Err(QError::SyntaxError(
                            "Invalid letter range".to_string().to_string(),
                        ))
                        .with_err_at(*pos);
                    }
                    state = STATE_SECOND_LETTER;
                    lexer.read_dp()?;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(*pos);
                }
            }
            Some(Locatable {
                element: Lexeme::Symbol('-'),
                pos,
            }) => {
                if state == STATE_FIRST_LETTER {
                    state = STATE_DASH;
                    lexer.read_dp()?;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(*pos);
                }
            }
            Some(Locatable {
                element: Lexeme::Symbol(','),
                pos,
            }) => {
                if state == STATE_FIRST_LETTER {
                    ranges.push(LetterRange::Single(first_letter));
                    state = STATE_COMMA;
                    lexer.read_dp()?;
                } else if state == STATE_SECOND_LETTER {
                    ranges.push(LetterRange::Range(first_letter, second_letter));
                    state = STATE_COMMA;
                    lexer.read_dp()?;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(*pos);
                }
            }
            _ => {
                // bail out
                if state == STATE_DASH {
                    return Err(QError::SyntaxError(
                        "Expected letter after dash".to_string(),
                    ))
                    .with_err_at(lexer.pos());
                } else if state == STATE_COMMA {
                    return Err(QError::SyntaxError(
                        "Expected letter range after comma".to_string(),
                    ))
                    .with_err_at(lexer.pos());
                } else if state == STATE_FIRST_LETTER {
                    ranges.push(LetterRange::Single(first_letter));
                    state = STATE_EOL;
                } else if state == STATE_SECOND_LETTER {
                    ranges.push(LetterRange::Range(first_letter, second_letter));
                    state = STATE_EOL;
                } else {
                    return Err(QError::SyntaxError(
                        "Expected at least one letter range".to_string(),
                    ))
                    .with_err_at(lexer.pos());
                }
            }
        }
    }
    lexer.commit_transaction();
    Ok(Some(
        TopLevelToken::DefType(DefType::new(opt_qualifier.unwrap(), ranges)).at(pos),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::parser::{HasQualifier, Statement};

    /// Asserts that the given input program contains a def type top level token.
    macro_rules! assert_def_type {
        ($input:expr, $expected_qualifier:expr, $expected_ranges:expr) => {
            match parse($input).demand_single().as_ref() {
                TopLevelToken::DefType(x) => {
                    assert_eq!(x.qualifier(), $expected_qualifier);
                    assert_eq!(x.ranges(), &$expected_ranges);
                }
                _ => panic!(format!("{:?}", $input)),
            }
        };
    }

    #[test]
    fn test_parse_def_int_a_z() {
        assert_def_type!(
            "DEFINT A-Z",
            TypeQualifier::PercentInteger,
            vec![LetterRange::Range('A', 'Z')]
        );
    }

    #[test]
    fn test_parse_def_int_a() {
        assert_def_type!(
            "DEFINT A",
            TypeQualifier::PercentInteger,
            vec![LetterRange::Single('A')]
        );
    }

    #[test]
    fn test_parse_def_str_a_b_c_mixed_whitespace() {
        assert_def_type!(
            "DEFSTR A, B,C  ",
            TypeQualifier::DollarString,
            vec![
                LetterRange::Single('A'),
                LetterRange::Single('B'),
                LetterRange::Single('C')
            ]
        );
    }

    #[test]
    fn test_parse_def_lng_a_i_k_w_z() {
        assert_def_type!(
            "DEFLNG A-I, K-W, Z",
            TypeQualifier::AmpersandLong,
            vec![
                LetterRange::Range('A', 'I'),
                LetterRange::Range('K', 'W'),
                LetterRange::Single('Z')
            ]
        );
    }

    #[test]
    fn test_parse_def_int_word_instead_of_letter() {
        assert_eq!(
            parse_err("DEFINT HELLO"),
            QError::SyntaxError("Expected single character".to_string(),)
        );
        assert_eq!(
            parse_err("DEFINT HELLO,Z"),
            QError::SyntaxError("Expected single character".to_string(),)
        );
        assert_eq!(
            parse_err("DEFINT A,HELLO"),
            QError::SyntaxError("Expected single character".to_string(),)
        );
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_eq!(
            parse_err("DEFINT Z-A"),
            QError::SyntaxError("Invalid letter range".to_string(),)
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = r#"
        DEFINT A-Z ' Improve performance
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::DefType(DefType::new(
                    TypeQualifier::PercentInteger,
                    vec![LetterRange::Range('A', 'Z')]
                ))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" Improve performance".to_string()))
                    .at_rc(2, 20),
            ]
        );
    }
}
