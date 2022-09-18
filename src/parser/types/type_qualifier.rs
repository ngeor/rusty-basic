use std::convert::TryFrom;
use std::fmt::Display;

use crate::common::{CanCastTo, QError};

use super::Operator;

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

impl TypeQualifier {
    pub fn bigger_numeric_type(&self, other: &Self) -> Option<Self> {
        match self {
            Self::BangSingle => match other {
                Self::BangSingle | Self::PercentInteger | Self::AmpersandLong => {
                    Some(Self::BangSingle)
                }
                Self::HashDouble => Some(Self::HashDouble),
                _ => None,
            },
            Self::HashDouble => match other {
                Self::BangSingle
                | Self::PercentInteger
                | Self::AmpersandLong
                | Self::HashDouble => Some(Self::HashDouble),
                _ => None,
            },
            Self::PercentInteger => match other {
                Self::BangSingle => Some(Self::BangSingle),
                Self::HashDouble => Some(Self::HashDouble),
                Self::PercentInteger => Some(Self::PercentInteger),
                Self::AmpersandLong => Some(Self::AmpersandLong),
                _ => None,
            },
            Self::AmpersandLong => match other {
                Self::BangSingle => Some(Self::BangSingle),
                Self::HashDouble => Some(Self::HashDouble),
                Self::PercentInteger | Self::AmpersandLong => Some(Self::AmpersandLong),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn cast_binary_op(&self, right: TypeQualifier, op: Operator) -> Option<Self> {
        match op {
            // 1. arithmetic operators
            // 1a. plus -> if we can cast self to right, that's the result
            Operator::Plus => {
                match self.bigger_numeric_type(&right) {
                    Some(result) => Some(result),
                    None => {
                        if *self == TypeQualifier::DollarString
                            && right == TypeQualifier::DollarString
                        {
                            // string concatenation
                            Some(*self)
                        } else {
                            None
                        }
                    }
                }
            }
            // 1b. minus, multiply, divide -> if we can cast self to right, and we're not a string, that's the result
            // MOD is covered later in logical operators because it's similar logic
            Operator::Minus | Operator::Multiply | Operator::Divide => {
                self.bigger_numeric_type(&right)
            }
            // 2. relational operators
            //    if we an cast self to right, the result is -1 or 0, therefore integer
            Operator::Less
            | Operator::LessOrEqual
            | Operator::Equal
            | Operator::GreaterOrEqual
            | Operator::Greater
            | Operator::NotEqual => {
                if self.can_cast_to(right) {
                    Some(TypeQualifier::PercentInteger)
                } else {
                    None
                }
            }
            // 3. logical operators, modulo operator
            //    they only work if both sides are cast-able to integer, which is also the result type
            Operator::And | Operator::Or | Operator::Modulo => {
                if self.can_cast_to(TypeQualifier::PercentInteger)
                    && right.can_cast_to(TypeQualifier::PercentInteger)
                {
                    Some(TypeQualifier::PercentInteger)
                } else {
                    None
                }
            }
        }
    }
}

// char -> TypeQualifier

impl TryFrom<char> for TypeQualifier {
    type Error = QError;

    fn try_from(ch: char) -> Result<TypeQualifier, QError> {
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
            Err(QError::syntax_error("Expected: %, &, !, # or $"))
        }
    }
}

// TypeQualifier -> char

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

impl PartialEq<char> for TypeQualifier {
    fn eq(&self, that: &char) -> bool {
        char::from(*self) == *that
    }
}

impl PartialEq<TypeQualifier> for char {
    fn eq(&self, that: &TypeQualifier) -> bool {
        that.eq(self)
    }
}

impl CanCastTo<TypeQualifier> for TypeQualifier {
    /// Checks if this `TypeQualifier` can be cast into the given one.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_basic::common::CanCastTo;
    /// use rusty_basic::parser::TypeQualifier;
    ///
    /// assert!(TypeQualifier::BangSingle.can_cast_to(TypeQualifier::PercentInteger));
    /// assert!(TypeQualifier::DollarString.can_cast_to(TypeQualifier::DollarString));
    /// assert!(!TypeQualifier::HashDouble.can_cast_to(TypeQualifier::DollarString));
    /// assert!(!TypeQualifier::DollarString.can_cast_to(TypeQualifier::AmpersandLong));
    /// ```
    fn can_cast_to(&self, other: Self) -> bool {
        match self {
            Self::DollarString => other == Self::DollarString,
            _ => other != Self::DollarString,
        }
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
