use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;

use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, QErrorNode> {
    let next = lexer.peek()?;
    let opt_qualifier = match next.as_ref() {
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
    read_demand_whitespace(lexer, "Expected whitespace after DEF* keyword")?;
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
        let next = lexer.peek()?;
        match next.as_ref() {
            Lexeme::Word(w) => {
                lexer.read()?;
                if w.len() != 1 {
                    return Err(QError::SyntaxError("Expected single character".to_string()))
                        .with_err_at(&next);
                }
                if state == STATE_INITIAL || state == STATE_COMMA {
                    first_letter = w.chars().next().unwrap();
                    state = STATE_FIRST_LETTER;
                } else if state == STATE_DASH {
                    second_letter = w.chars().next().unwrap();
                    if first_letter > second_letter {
                        return Err(QError::SyntaxError(
                            "Invalid letter range".to_string().to_string(),
                        ))
                        .with_err_at(&next);
                    }
                    state = STATE_SECOND_LETTER;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(&next);
                }
            }
            Lexeme::Symbol('-') => {
                lexer.read()?;
                if state == STATE_FIRST_LETTER {
                    state = STATE_DASH;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(&next);
                }
            }
            Lexeme::Symbol(',') => {
                lexer.read()?;
                if state == STATE_FIRST_LETTER {
                    ranges.push(LetterRange::Single(first_letter));
                    state = STATE_COMMA;
                } else if state == STATE_SECOND_LETTER {
                    ranges.push(LetterRange::Range(first_letter, second_letter));
                    state = STATE_COMMA;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(&next);
                }
            }
            _ => {
                // bail out
                if state == STATE_DASH {
                    return Err(QError::SyntaxError(
                        "Expected letter after dash".to_string(),
                    ))
                    .with_err_at(&next);
                } else if state == STATE_COMMA {
                    return Err(QError::SyntaxError(
                        "Expected letter range after comma".to_string(),
                    ))
                    .with_err_at(&next);
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
                    .with_err_at(&next);
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
