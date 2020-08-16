use crate::common::Locatable;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

//
// TypeQualifier
//

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
    /// Not an actual type, but to be able to call PRINT #1, "hello", we define the file handle type
    FileHandle,
}

impl TypeQualifier {
    pub fn can_cast_to(&self, other: Self) -> bool {
        match self {
            Self::DollarString => other == Self::DollarString,
            Self::FileHandle => other == Self::FileHandle,
            _ => other != Self::DollarString && other != Self::FileHandle,
        }
    }

    /// Maps the given character into a `TypeQualifier`.
    /// If the character does not represent a type qualifier, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_basic::parser::TypeQualifier;
    /// assert_eq!(TypeQualifier::from_char('#'), Some(TypeQualifier::HashDouble));
    /// assert_eq!(TypeQualifier::from_char('1'), None);
    /// ```
    pub fn from_char(ch: char) -> Option<Self> {
        if ch == '!' {
            Some(TypeQualifier::BangSingle)
        } else if ch == '#' {
            Some(TypeQualifier::HashDouble)
        } else if ch == '$' {
            Some(TypeQualifier::DollarString)
        } else if ch == '%' {
            Some(TypeQualifier::PercentInteger)
        } else if ch == '&' {
            Some(TypeQualifier::AmpersandLong)
        } else {
            None
        }
    }
}

impl Display for TypeQualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeQualifier::BangSingle => write!(f, "!"),
            TypeQualifier::HashDouble => write!(f, "#"),
            TypeQualifier::DollarString => write!(f, "$"),
            TypeQualifier::PercentInteger => write!(f, "%"),
            TypeQualifier::AmpersandLong => write!(f, "&"),
            TypeQualifier::FileHandle => write!(f, "<file handle>"),
        }
    }
}

impl FromStr for TypeQualifier {
    type Err = String;
    fn from_str(s: &str) -> Result<TypeQualifier, String> {
        if s == "!" {
            Ok(TypeQualifier::BangSingle)
        } else if s == "#" {
            Ok(TypeQualifier::HashDouble)
        } else if s == "$" {
            Ok(TypeQualifier::DollarString)
        } else if s == "%" {
            Ok(TypeQualifier::PercentInteger)
        } else if s == "&" {
            Ok(TypeQualifier::AmpersandLong)
        } else {
            Err(format!("Invalid type qualifier {}", s))
        }
    }
}

impl TryFrom<char> for TypeQualifier {
    type Error = String;
    fn try_from(ch: char) -> Result<TypeQualifier, String> {
        TypeQualifier::from_char(ch).ok_or_else(|| format!("Invalid type qualifier {}", ch))
    }
}

impl TryFrom<TypeQualifier> for char {
    type Error = String;

    fn try_from(q: TypeQualifier) -> Result<char, String> {
        match q {
            TypeQualifier::BangSingle => Ok('!'),
            TypeQualifier::HashDouble => Ok('#'),
            TypeQualifier::DollarString => Ok('$'),
            TypeQualifier::PercentInteger => Ok('%'),
            TypeQualifier::AmpersandLong => Ok('&'),
            _ => Err(format!("Cannot convert {:?} to char", q)),
        }
    }
}

impl PartialEq<char> for TypeQualifier {
    fn eq(&self, that: &char) -> bool {
        match char::try_from(*self) {
            Ok(ch) => ch == *that,
            Err(_) => false,
        }
    }
}

impl PartialEq<TypeQualifier> for char {
    fn eq(&self, that: &TypeQualifier) -> bool {
        that.eq(self)
    }
}

//
// HasQualifier
//

pub trait HasQualifier {
    fn qualifier(&self) -> TypeQualifier;
}

impl<T: HasQualifier> HasQualifier for Locatable<T> {
    fn qualifier(&self) -> TypeQualifier {
        self.as_ref().qualifier()
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
        assert_eq!("<file handle>", format!("{}", TypeQualifier::FileHandle));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            TypeQualifier::from_str("!").unwrap(),
            TypeQualifier::BangSingle
        );
        assert_eq!(
            TypeQualifier::from_str("#").unwrap(),
            TypeQualifier::HashDouble
        );
        assert_eq!(
            TypeQualifier::from_str("$").unwrap(),
            TypeQualifier::DollarString
        );
        assert_eq!(
            TypeQualifier::from_str("%").unwrap(),
            TypeQualifier::PercentInteger
        );
        assert_eq!(
            TypeQualifier::from_str("&").unwrap(),
            TypeQualifier::AmpersandLong
        );
        TypeQualifier::from_str(".").expect_err("should not parse dot");
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
        assert_eq!(char::try_from(TypeQualifier::BangSingle).unwrap(), '!');
        assert_eq!(char::try_from(TypeQualifier::HashDouble).unwrap(), '#');
        assert_eq!(char::try_from(TypeQualifier::DollarString).unwrap(), '$');
        assert_eq!(char::try_from(TypeQualifier::PercentInteger).unwrap(), '%');
        assert_eq!(char::try_from(TypeQualifier::AmpersandLong).unwrap(), '&');
        assert_eq!(
            char::try_from(TypeQualifier::FileHandle).unwrap_err(),
            "Cannot convert FileHandle to char"
        );
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
        // file handle
        assert_ne!('!', TypeQualifier::FileHandle);
        assert_ne!(TypeQualifier::FileHandle, '$');
    }
}
