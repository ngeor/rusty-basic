use crate::common::*;
use crate::parser::char_reader::EolReader;
use crate::parser::pc::common::{and, demand, map_default_to_not_found, or, or_vec, seq2, seq3};
use crate::parser::pc::map::{and_then, map};
use crate::parser::pc::*;
use crate::parser::pc_specific::{any_letter, csv_zero_or_more, keyword};
use crate::parser::types::*;
use std::io::BufRead;

// DefType      ::= <DefKeyword><ws+><LetterRanges>
// DefKeyword   ::= DEFSNG|DEFDBL|DEFSTR|DEFINT|DEFLNG
// LetterRanges ::= <LetterRange> | <LetterRange><ws*>,<ws*><LetterRanges>
// LetterRange  ::= <Letter> | <Letter>-<Letter>
// Letter       ::= [a-zA-Z]

pub fn def_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, DefType, QError>> {
    map(
        seq3(
            def_keyword(),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace"),
            ),
            demand(
                letter_ranges(),
                QError::syntax_error_fn("Expected: letter ranges"),
            ),
        ),
        |(l, _, r)| DefType::new(l, r),
    )
}

fn def_keyword<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TypeQualifier, QError>> {
    or_vec(vec![
        map(keyword(Keyword::DefDbl), |_| TypeQualifier::HashDouble),
        map(keyword(Keyword::DefInt), |_| TypeQualifier::PercentInteger),
        map(keyword(Keyword::DefLng), |_| TypeQualifier::AmpersandLong),
        map(keyword(Keyword::DefSng), |_| TypeQualifier::BangSingle),
        map(keyword(Keyword::DefStr), |_| TypeQualifier::DollarString),
    ])
}

fn letter_ranges<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Vec<LetterRange>, QError>> {
    map_default_to_not_found(csv_zero_or_more(letter_range()))
}

fn letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, LetterRange, QError>> {
    or(
        two_letter_range(), // needs to be first because the second will match too
        single_letter_range(),
    )
}

fn single_letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, LetterRange, QError>> {
    map(any_letter(), |l| LetterRange::Single(l))
}

fn two_letter_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, LetterRange, QError>> {
    and_then(
        and(
            any_letter(),
            seq2(
                read('-'),
                demand(
                    any_letter(),
                    QError::syntax_error_fn("Expected: letter after dash"),
                ),
            ),
        ),
        |(l, (_, r))| {
            if l < r {
                Ok(LetterRange::Range(l, r))
            } else {
                Err(QError::SyntaxError("Invalid letter range".to_string()))
            }
        },
    )
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
            QError::SyntaxError("No separator: E".to_string(),)
        );
        assert_eq!(
            parse_err("DEFINT HELLO,Z"),
            QError::SyntaxError("No separator: E".to_string(),)
        );
        assert_eq!(
            parse_err("DEFINT A,HELLO"),
            QError::SyntaxError("No separator: E".to_string(),)
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
