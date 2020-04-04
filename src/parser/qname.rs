use super::{Parser, TypeQualifier};
use crate::common::Result;
use crate::lexer::Lexeme;
use std::convert::TryFrom;
use std::fmt::Display;
use std::io::BufRead;
use std::str::FromStr;

/// BareName is a variable or function name without a type qualifier
pub type BareName = String;

/// QualifiedName is a variable or function name with a type qualifier
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct QualifiedName {
    pub name: BareName,
    pub qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(name: BareName, qualifier: TypeQualifier) -> QualifiedName {
        QualifiedName { name, qualifier }
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, self.qualifier)
    }
}

/// QName is either a BareName or a QualifiedName
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum QName {
    Untyped(BareName),
    Typed(QualifiedName),
}

impl Display for QName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QName::Untyped(name) => write!(f, "{}", name),
            QName::Typed(qualified_name) => write!(f, "{}", qualified_name),
        }
    }
}

impl QName {
    pub fn new(name: String, optional_qualifier: Option<TypeQualifier>) -> QName {
        match optional_qualifier {
            Some(qualifier) => QName::Typed(QualifiedName::new(name, qualifier)),
            None => QName::Untyped(name),
        }
    }

    pub fn get_bare_name(&self) -> &String {
        match self {
            QName::Untyped(bare_name) => bare_name,
            QName::Typed(qualified_name) => &qualified_name.name,
        }
    }
}

impl FromStr for QName {
    type Err = String;
    fn from_str(s: &str) -> Result<QName> {
        let mut buf = s.to_string();
        match buf.pop() {
            None => Err("empty string".to_string()),
            Some(ch) => match TypeQualifier::try_from(ch) {
                Ok(type_qualifier) => Ok(QName::Typed(QualifiedName::new(buf, type_qualifier))),
                Err(_) => {
                    buf.push(ch);
                    Ok(QName::Untyped(buf))
                }
            },
        }
    }
}

impl<T: BufRead> Parser<T> {
    pub fn try_parse_name_with_type_qualifier(&mut self) -> Result<Option<QName>> {
        let next = self.buf_lexer.try_consume_any_word()?;
        match next {
            Some(word) => {
                let optional_type_qualifier = self.try_parse_type_qualifier()?;
                Ok(Some(QName::new(word, optional_type_qualifier)))
            }
            None => Ok(None),
        }
    }

    pub fn demand_name_with_type_qualifier(&mut self) -> Result<QName> {
        let name = self.buf_lexer.demand_any_word()?;
        let optional_type_qualifier = self.try_parse_type_qualifier()?;
        Ok(QName::new(name, optional_type_qualifier))
    }

    pub fn try_parse_type_qualifier(&mut self) -> Result<Option<TypeQualifier>> {
        let next = self.buf_lexer.read()?;
        match next {
            Lexeme::Symbol(ch) => match TypeQualifier::try_from(ch) {
                Ok(t) => {
                    self.buf_lexer.consume();
                    Ok(Some(t))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert_eq!("A", format!("{}", QName::Untyped("A".to_string())));
        assert_eq!(
            "B$",
            format!(
                "{}",
                QName::Typed(QualifiedName::new(
                    "B".to_string(),
                    TypeQualifier::DollarString
                ))
            )
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            QName::from_str("A").unwrap(),
            QName::Untyped("A".to_string())
        );
        assert_eq!(
            QName::from_str("Fib").unwrap(),
            QName::Untyped("Fib".to_string())
        );
        assert_eq!(
            QName::from_str("A$").unwrap(),
            QName::Typed(QualifiedName::new(
                "A".to_string(),
                TypeQualifier::DollarString
            ))
        );
        assert_eq!(
            QName::from_str("Fib%").unwrap(),
            QName::Typed(QualifiedName::new(
                "Fib".to_string(),
                TypeQualifier::PercentInteger
            ))
        );
    }
}
