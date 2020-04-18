use super::{
    unexpected, DefTypeNode, LetterRangeNode, Parser, ParserError, TopLevelTokenNode, TypeQualifier,
};
use crate::common::Location;
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_def_type(
        &mut self,
        keyword: Keyword,
        pos: Location,
    ) -> Result<TopLevelTokenNode, ParserError> {
        let qualifier = match keyword {
            Keyword::DefDbl => TypeQualifier::HashDouble,
            Keyword::DefInt => TypeQualifier::PercentInteger,
            Keyword::DefLng => TypeQualifier::AmpersandLong,
            Keyword::DefSng => TypeQualifier::BangSingle,
            Keyword::DefStr => TypeQualifier::DollarString,
            _ => panic!(format!(
                "Unexpected keyword {}, should be one of DEF*",
                keyword
            )),
        };

        self.read_demand_whitespace("Expected whitespace after DEF* keyword")?;
        let mut ranges: Vec<LetterRangeNode> = vec![];
        const STATE_INITIAL: u8 = 0;
        const STATE_FIRST_LETTER: u8 = 1;
        const STATE_DASH: u8 = 2;
        const STATE_SECOND_LETTER: u8 = 3;
        const STATE_COMMA: u8 = 4;
        const STATE_EOL: u8 = 5;
        let mut state = STATE_INITIAL;
        let mut first_letter = ' ';
        let mut second_letter = ' ';
        while state != STATE_EOL {
            let next = self.read_skipping_whitespace()?;
            match &next {
                LexemeNode::EOF(_) | LexemeNode::EOL(_, _) => {
                    if state == STATE_DASH {
                        return unexpected("Expected letter after dash", next);
                    } else if state == STATE_COMMA {
                        return unexpected("Expected letter range after comma", next);
                    } else if state == STATE_FIRST_LETTER {
                        ranges.push(LetterRangeNode::Single(first_letter));
                        state = STATE_EOL;
                    } else if state == STATE_SECOND_LETTER {
                        ranges.push(LetterRangeNode::Range(first_letter, second_letter));
                        state = STATE_EOL;
                    } else {
                        return unexpected("Expected at least one letter range", next);
                    }
                }
                LexemeNode::Word(w, _) => {
                    if w.len() != 1 {
                        return unexpected("Expected single character", next);
                    }
                    if state == STATE_INITIAL || state == STATE_COMMA {
                        first_letter = w.chars().next().unwrap();
                        state = STATE_FIRST_LETTER;
                    } else if state == STATE_DASH {
                        second_letter = w.chars().next().unwrap();
                        if first_letter > second_letter {
                            return unexpected("Invalid letter range".to_string(), next);
                        }
                        state = STATE_SECOND_LETTER;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
                LexemeNode::Symbol('-', _) => {
                    if state == STATE_FIRST_LETTER {
                        state = STATE_DASH;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
                LexemeNode::Symbol(',', _) => {
                    if state == STATE_FIRST_LETTER {
                        ranges.push(LetterRangeNode::Single(first_letter));
                        state = STATE_COMMA;
                    } else if state == STATE_SECOND_LETTER {
                        ranges.push(LetterRangeNode::Range(first_letter, second_letter));
                        state = STATE_COMMA;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
                _ => return unexpected("Syntax error", next),
            }
        }
        Ok(TopLevelTokenNode::DefType(DefTypeNode::new(
            qualifier, ranges, pos,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::parser::HasQualifier;

    /// Asserts that the given input program contains a def type top level token.
    macro_rules! assert_def_type {
        ($input:expr, $expected_qualifier:expr, $expected_ranges:expr) => {
            match parse($input).demand_single() {
                TopLevelTokenNode::DefType(x) => {
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
            vec![LetterRangeNode::Range('A', 'Z')]
        );
    }

    #[test]
    fn test_parse_def_int_a() {
        assert_def_type!(
            "DEFINT A",
            TypeQualifier::PercentInteger,
            vec![LetterRangeNode::Single('A')]
        );
    }

    #[test]
    fn test_parse_def_str_a_b_c_mixed_whitespace() {
        assert_def_type!(
            "DEFSTR A, B,C  ",
            TypeQualifier::DollarString,
            vec![
                LetterRangeNode::Single('A'),
                LetterRangeNode::Single('B'),
                LetterRangeNode::Single('C')
            ]
        );
    }

    #[test]
    fn test_parse_def_lng_a_i_k_w_z() {
        assert_def_type!(
            "DEFLNG A-I, K-W, Z",
            TypeQualifier::AmpersandLong,
            vec![
                LetterRangeNode::Range('A', 'I'),
                LetterRangeNode::Range('K', 'W'),
                LetterRangeNode::Single('Z')
            ]
        );
    }

    #[test]
    fn test_parse_def_int_word_instead_of_letter() {
        assert_eq!(
            parse_err("DEFINT HELLO"),
            ParserError::Unexpected(
                "Expected single character".to_string(),
                LexemeNode::Word("HELLO".to_string(), Location::new(1, 8))
            )
        );
        assert_eq!(
            parse_err("DEFINT HELLO,Z"),
            ParserError::Unexpected(
                "Expected single character".to_string(),
                LexemeNode::Word("HELLO".to_string(), Location::new(1, 8))
            )
        );
        assert_eq!(
            parse_err("DEFINT A,HELLO"),
            ParserError::Unexpected(
                "Expected single character".to_string(),
                LexemeNode::Word("HELLO".to_string(), Location::new(1, 10))
            )
        );
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_eq!(
            parse_err("DEFINT Z-A"),
            ParserError::Unexpected(
                "Invalid letter range".to_string(),
                LexemeNode::Word("A".to_string(), Location::new(1, 10))
            )
        );
    }
}
