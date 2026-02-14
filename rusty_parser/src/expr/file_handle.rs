//! Used by PRINT and built-ins

use rusty_common::*;
use rusty_pc::*;

use crate::error::ParserError;
use crate::expr::ws_expr_pos_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::{TokenType, any_token_of, pound};
use crate::*;

pub fn file_handle_p()
-> impl Parser<StringView, Output = Positioned<FileHandle>, Error = ParserError> {
    // # and digits
    // if # and 0 -> BadFileNameOrNumber
    // if # without digits -> SyntaxError (Expected: digits after #)
    pound()
        .with_pos()
        .and_tuple(any_token_of!(TokenType::Digits).or_expected("digits after #"))
        .and_then(|(pound, digits)| match digits.as_str().parse::<u8>() {
            Ok(d) if d > 0 => Ok(FileHandle::from(d).at_pos(pound.pos)),
            _ => Err(ParserError::BadFileNameOrNumber),
        })
}

/// Parses a file handle ( e.g. `#1` ) as an integer literal expression.
pub fn file_handle_as_expression_pos_p()
-> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    file_handle_p().map(|file_handle_pos| file_handle_pos.map(Expression::from))
}

pub fn guarded_file_handle_or_expression_p()
-> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    ws_file_handle().or(ws_expr_pos_p())
}

fn ws_file_handle() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    lead_ws(file_handle_as_expression_pos_p())
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::{assert_file_handle, assert_parser_err, *};

    #[test]
    fn test_valid_file_handles() {
        assert_file_handle!("CLOSE #1", 1);
        assert_file_handle!("CLOSE #2", 2);
        assert_file_handle!("CLOSE #255", 255); // max value
    }

    #[test]
    fn test_file_handle_zero() {
        let input = "CLOSE #0";
        assert_parser_err!(input, ParserError::BadFileNameOrNumber);
    }

    #[test]
    fn test_file_handle_overflow() {
        let input = "CLOSE #256";
        assert_parser_err!(input, ParserError::BadFileNameOrNumber);
    }

    #[test]
    fn test_file_handle_negative() {
        let input = "CLOSE #-1";
        assert_parser_err!(input, expected("digits after #"));
    }
}
