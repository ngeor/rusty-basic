// TODO #[cfg(test)]
use crate::ParseError;
use std::convert::TryFrom;
use std::fmt::Display;

/// The optional character postfix that specifies the type of a name.
/// Example: A$ denotes a string variable
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeQualifier {
    /// `!` Single-precision
    BangSingle,
    /// `#` Double-precision
    HashDouble,
    /// `$` String
    DollarString,
    /// `%` Integer
    PercentInteger,
    /// `&` Long-integer
    AmpersandLong,
}

// char -> TypeQualifier

// TODO #[cfg(test)]
impl TryFrom<char> for TypeQualifier {
    type Error = ParseError;

    fn try_from(ch: char) -> Result<TypeQualifier, ParseError> {
        if ch == '!' {
            Ok(TypeQualifier::BangSingle)
        } else if ch == '#' {
            Ok(TypeQualifier::HashDouble)
        } else if ch == '$' {
            Ok(TypeQualifier::DollarString)
        } else if ch == '%' {
            Ok(TypeQualifier::PercentInteger)
        } else if ch == '&' {
            Ok(TypeQualifier::AmpersandLong)
        } else {
            Err(ParseError::syntax_error("Expected: %, &, !, # or $"))
        }
    }
}

// TypeQualifier -> char (for implementing Display trait)

impl From<TypeQualifier> for char {
    fn from(q: TypeQualifier) -> char {
        match q {
            TypeQualifier::BangSingle => '!',
            TypeQualifier::HashDouble => '#',
            TypeQualifier::DollarString => '$',
            TypeQualifier::PercentInteger => '%',
            TypeQualifier::AmpersandLong => '&',
        }
    }
}

impl Display for TypeQualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        char::from(*self).fmt(f)
    }
}

#[cfg(test)]
impl PartialEq<char> for TypeQualifier {
    fn eq(&self, that: &char) -> bool {
        char::from(*self) == *that
    }
}

#[cfg(test)]
impl PartialEq<TypeQualifier> for char {
    fn eq(&self, that: &TypeQualifier) -> bool {
        that.eq(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert_eq!("!", format!("{}", TypeQualifier::BangSingle));
        assert_eq!("#", format!("{}", TypeQualifier::HashDouble));
        assert_eq!("$", format!("{}", TypeQualifier::DollarString));
        assert_eq!("%", format!("{}", TypeQualifier::PercentInteger));
        assert_eq!("&", format!("{}", TypeQualifier::AmpersandLong));
    }

    #[test]
    fn test_try_from_char() {
        assert_eq!(
            TypeQualifier::try_from('!').unwrap(),
            TypeQualifier::BangSingle
        );
        assert_eq!(
            TypeQualifier::try_from('#').unwrap(),
            TypeQualifier::HashDouble
        );
        assert_eq!(
            TypeQualifier::try_from('$').unwrap(),
            TypeQualifier::DollarString
        );
        assert_eq!(
            TypeQualifier::try_from('%').unwrap(),
            TypeQualifier::PercentInteger
        );
        assert_eq!(
            TypeQualifier::try_from('&').unwrap(),
            TypeQualifier::AmpersandLong
        );
        TypeQualifier::try_from('.').expect_err("should not parse dot");
    }

    #[test]
    fn test_char_from_type_qualifier() {
        assert_eq!(char::from(TypeQualifier::BangSingle), '!');
        assert_eq!(char::from(TypeQualifier::HashDouble), '#');
        assert_eq!(char::from(TypeQualifier::DollarString), '$');
        assert_eq!(char::from(TypeQualifier::PercentInteger), '%');
        assert_eq!(char::from(TypeQualifier::AmpersandLong), '&');
    }

    #[test]
    fn test_partial_eq_char() {
        // basic five types
        assert_eq!(TypeQualifier::BangSingle, '!');
        assert_eq!(TypeQualifier::HashDouble, '#');
        assert_eq!(TypeQualifier::DollarString, '$');
        assert_eq!(TypeQualifier::PercentInteger, '%');
        assert_eq!(TypeQualifier::AmpersandLong, '&');
        // reflexive
        assert_eq!('!', TypeQualifier::BangSingle);
        // ne
        assert_ne!(TypeQualifier::BangSingle, '#');
        // invalid characters
        assert_ne!(TypeQualifier::BangSingle, '.');
        assert_ne!('.', TypeQualifier::BangSingle);
    }
}
