use rusty_pc::*;

use crate::input::StringView;
use crate::pc_specific::WithPos;
use crate::tokens::{digits, dot, pound};
use crate::{ParserError, *};

// single ::= <digits> . <digits>
// single ::= . <digits> (without leading zero)
// double ::= <digits> . <digits> "#"
// double ::= . <digits> "#"

// TODO support more qualifiers besides '#'

pub fn parser() -> impl Parser<StringView, Output = ExpressionPos, Error = ParserError> {
    // read integer digits optionally (might start with . e.g. `.123`)
    digits()
        .to_option()
        // read dot and demand digits after decimal point
        // if dot is missing, the parser returns an empty result
        // the "deal breaker" is therefore the dot
        .and_keep_left(dot())
        // demand digits after decimal point
        .and_tuple(digits().to_fatal())
        // and parse optionally a type qualifier such as `#`
        .and_tuple(pound().to_option())
        // done parsing, flat map everything
        .and_then(|((opt_integer_digits, frac_digits), opt_pound)| {
            let left = opt_integer_digits
                .map(|token| token.to_string())
                .unwrap_or_else(|| "0".to_owned());
            let s = format!("{}.{}", left, frac_digits.as_str());
            if opt_pound.is_some() {
                match s.parse::<f64>() {
                    Ok(f) => Ok(Expression::DoubleLiteral(f)),
                    Err(err) => Err(err.into()),
                }
            } else {
                match s.parse::<f32>() {
                    Ok(f) => Ok(Expression::SingleLiteral(f)),
                    Err(err) => Err(err.into()),
                }
            }
        })
        .with_pos()
}
