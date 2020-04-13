use super::{DefTypeNode, LetterRangeNode, Parser, TopLevelTokenNode, TypeQualifier};
use crate::common::Location;
use crate::lexer::{Keyword, LexemeNode, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_def_type(&mut self) -> Result<Option<TopLevelTokenNode>, LexerError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(k, _, pos) => self._try_parse(k, pos),
            _ => Ok(None),
        }
    }

    fn _try_parse(
        &mut self,
        keyword: Keyword,
        pos: Location,
    ) -> Result<Option<TopLevelTokenNode>, LexerError> {
        let opt_qualifier = match keyword {
            Keyword::DefDbl => Some(TypeQualifier::HashDouble),
            Keyword::DefInt => Some(TypeQualifier::PercentInteger),
            Keyword::DefLng => Some(TypeQualifier::AmpersandLong),
            Keyword::DefSng => Some(TypeQualifier::BangSingle),
            Keyword::DefStr => Some(TypeQualifier::DollarString),
            _ => None,
        };

        match opt_qualifier {
            Some(qualifier) => {
                self.buf_lexer.consume();
                self._parse_def_type(qualifier, pos).map(|x| Some(x))
            }
            None => Ok(None),
        }
    }

    fn _parse_def_type(
        &mut self,
        qualifier: TypeQualifier,
        pos: Location,
    ) -> Result<TopLevelTokenNode, LexerError> {
        self.buf_lexer.demand_whitespace()?;
        let mut has_more = true;
        let mut ranges: Vec<LetterRangeNode> = vec![];
        while has_more {
            let (letter1, _) = self._demand_letter()?;
            self.buf_lexer.skip_whitespace()?;
            if self.buf_lexer.try_consume_symbol('-')?.is_some() {
                // range, like A-Z
                ranges.push(self._add_letter_range(letter1)?);
            } else {
                // single letter
                ranges.push(LetterRangeNode::Single(letter1));
            }

            has_more = self.buf_lexer.try_consume_symbol(',')?.is_some();
            self.buf_lexer.skip_whitespace()?;
        }
        self.buf_lexer.demand_eol_or_eof()?;
        Ok(TopLevelTokenNode::DefType(DefTypeNode::new(
            qualifier, ranges, pos,
        )))
    }

    fn _add_letter_range(&mut self, letter1: char) -> Result<LetterRangeNode, LexerError> {
        // range, like A-Z
        self.buf_lexer.skip_whitespace()?;
        let (letter2, pos) = self._demand_letter()?;
        self.buf_lexer.skip_whitespace()?;
        if letter1 > letter2 {
            Err(LexerError::Unexpected(
                "Invalid letter range".to_string(),
                LexemeNode::Word(letter2.to_string(), pos),
            ))
        } else {
            Ok(LetterRangeNode::Range(letter1, letter2))
        }
    }

    fn _demand_letter(&mut self) -> Result<(char, Location), LexerError> {
        let next = self.buf_lexer.read()?;
        self.buf_lexer.consume();
        match &next {
            LexemeNode::Word(s, pos) => {
                if s.len() == 1 {
                    Ok((s.chars().next().unwrap(), *pos))
                } else {
                    Err(LexerError::Unexpected(
                        "Expected single character".to_string(),
                        next,
                    ))
                }
            }
            _ => Err(LexerError::Unexpected(
                "Expected single character".to_string(),
                next,
            )),
        }
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
            LexerError::Unexpected(
                "Expected single character".to_string(),
                LexemeNode::Word("HELLO".to_string(), Location::new(1, 8))
            )
        );
        assert_eq!(
            parse_err("DEFINT HELLO,Z"),
            LexerError::Unexpected(
                "Expected single character".to_string(),
                LexemeNode::Word("HELLO".to_string(), Location::new(1, 8))
            )
        );
        assert_eq!(
            parse_err("DEFINT A,HELLO"),
            LexerError::Unexpected(
                "Expected single character".to_string(),
                LexemeNode::Word("HELLO".to_string(), Location::new(1, 10))
            )
        );
    }

    #[test]
    fn test_parse_def_int_reverse_range() {
        assert_eq!(
            parse_err("DEFINT Z-A"),
            LexerError::Unexpected(
                "Invalid letter range".to_string(),
                LexemeNode::Word("A".to_string(), Location::new(1, 10))
            )
        );
    }
}
