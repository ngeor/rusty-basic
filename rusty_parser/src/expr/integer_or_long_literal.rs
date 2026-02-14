use rusty_pc::*;
use rusty_variant::{BitVec, BitVecIntOrLong, MAX_INTEGER, MAX_LONG};

use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::WithPos;
use crate::tokens::{TokenType, any_token_of};
use crate::{Expression, ExpressionPos};

// result ::= <digits> | <hex-digits> | <oct-digits>
pub(super) fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    any_token_of!(
        TokenType::Digits,
        TokenType::HexDigits,
        TokenType::OctDigits
    )
    .and_then(process_token)
    .with_pos()
}

fn process_token(token: Token) -> Result<Expression, ParserError> {
    match TokenType::from_token(&token) {
        TokenType::Digits => process_dec(token),
        TokenType::HexDigits => process_hex(token),
        TokenType::OctDigits => process_oct(token),
        _ => panic!("Should not have processed {}", token),
    }
}

fn process_dec(token: Token) -> Result<Expression, ParserError> {
    match token.to_string().parse::<u32>() {
        Ok(u) => {
            if u <= MAX_INTEGER as u32 {
                Ok(Expression::IntegerLiteral(u as i32))
            } else if u <= MAX_LONG as u32 {
                Ok(Expression::LongLiteral(u as i64))
            } else {
                Ok(Expression::DoubleLiteral(u as f64))
            }
        }
        Err(e) => Err(e.into()),
    }
}

fn process_hex(token: Token) -> Result<Expression, ParserError> {
    // token text is &HFFFF or &H-FFFF
    let mut s: String = token.to_string();
    // remove &
    s.remove(0);
    // remove H
    s.remove(0);
    if s.starts_with('-') {
        Err(ParserError::Overflow)
    } else {
        let mut result: BitVec = BitVec::new();
        for digit in s.chars().skip_while(|ch| *ch == '0') {
            let hex = convert_hex_digit(digit);
            result.push_hex(hex);
        }
        create_expression_from_bit_vec(result)
    }
}

fn convert_hex_digit(ch: char) -> u8 {
    if ch.is_ascii_digit() {
        (ch as u8) - b'0'
    } else if ('a'..='f').contains(&ch) {
        (ch as u8) - b'a' + 10
    } else if ('A'..='F').contains(&ch) {
        (ch as u8) - b'A' + 10
    } else {
        panic!("Unexpected hex digit: {}", ch)
    }
}

fn process_oct(token: Token) -> Result<Expression, ParserError> {
    let mut s: String = token.to_string();
    // remove &
    s.remove(0);
    // remove O
    s.remove(0);
    if s.starts_with('-') {
        Err(ParserError::Overflow)
    } else {
        let mut result: BitVec = BitVec::new();
        for digit in s.chars().skip_while(|ch| *ch == '0') {
            let oct = convert_oct_digit(digit);
            result.push_oct(oct);
        }
        create_expression_from_bit_vec(result)
    }
}

fn convert_oct_digit(ch: char) -> u8 {
    if ('0'..='7').contains(&ch) {
        (ch as u8) - b'0'
    } else {
        panic!("Unexpected oct digit: {}", ch)
    }
}

fn create_expression_from_bit_vec(bit_vec: BitVec) -> Result<Expression, ParserError> {
    bit_vec
        .convert_to_int_or_long_expr()
        .map(|x| match x {
            BitVecIntOrLong::Int(i) => Expression::IntegerLiteral(i),
            BitVecIntOrLong::Long(l) => Expression::LongLiteral(l),
        })
        .map_err(|_| ParserError::Overflow)
}

#[cfg(test)]
mod tests {
    use crate::{assert_parser_err, *};
    mod hexadecimal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &H-10", ParserError::Overflow);
            assert_parser_err!("PRINT &H100000000", ParserError::Overflow);
        }
    }

    mod octal {
        use super::*;

        #[test]
        fn test_overflow() {
            assert_parser_err!("PRINT &O-10", ParserError::Overflow);
            assert_parser_err!("PRINT &O40000000000", ParserError::Overflow);
        }
    }
}
