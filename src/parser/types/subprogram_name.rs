use super::{BareName, QualifiedName};
use std::fmt::Formatter;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SubprogramName {
    Function(QualifiedName),
    Sub(BareName),
}

impl std::fmt::Display for SubprogramName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(qualified_name) => {
                f.write_str(".F.")?;
                std::fmt::Display::fmt(qualified_name, f)
            }
            Self::Sub(bare_name) => {
                f.write_str(".S.")?;
                std::fmt::Display::fmt(bare_name, f)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{BareName, QualifiedName, SubprogramName};
    use std::convert::TryFrom;

    #[test]
    fn format_function() {
        // arrange
        let function_name = QualifiedName::try_from("Hello%").unwrap();
        let subprogram_name = SubprogramName::Function(function_name);
        // act
        let s = subprogram_name.to_string();
        // assert
        assert_eq!(s, ".F.Hello%");
    }

    #[test]
    fn format_sub() {
        // arrange
        let sub_name = BareName::from("Foo");
        let subprogram_name = SubprogramName::Sub(sub_name);
        // act
        let s = subprogram_name.to_string();
        // assert
        assert_eq!(s, ".S.Foo");
    }
}
