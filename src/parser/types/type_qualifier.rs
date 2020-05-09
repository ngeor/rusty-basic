use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

/// The optional character postfix that specifies the type of a name.
/// Example: A$ denotes a string variable
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeQualifier {
    /// ! Single-precision
    BangSingle,
    /// # Double-precision
    HashDouble,
    /// $ String
    DollarString,
    /// % Integer
    PercentInteger,
    /// & Long-integer
    AmpersandLong,
}

impl TypeQualifier {
    pub fn can_cast_to(&self, other: Self) -> bool {
        match self {
            Self::DollarString => other == Self::DollarString,
            _ => other != Self::DollarString,
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
            Err(format!("Invalid type qualifier {}", ch))
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
}
