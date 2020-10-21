use crate::parser::{BareName, ExpressionNodes, TypeQualifier};

#[derive(Clone, Debug, PartialEq)]
pub struct NameExpr {
    // A
    // A$
    // MID$(1, 2) = "hi"
    // A(1).Card
    // A.B.C
    // choices$() whole array as parameter
    pub bare_name: BareName,
    pub qualifier: Option<TypeQualifier>,
    pub arguments: Option<ExpressionNodes>,
    pub elements: Option<Vec<BareName>>,
}

impl NameExpr {
    pub fn bare<S: AsRef<str>>(bare_name: S) -> Self {
        Self {
            bare_name: bare_name.as_ref().into(),
            qualifier: None,
            arguments: None,
            elements: None,
        }
    }

    pub fn qualified<S: AsRef<str>>(bare_name: S, qualifier: TypeQualifier) -> Self {
        Self {
            bare_name: bare_name.as_ref().into(),
            qualifier: Some(qualifier),
            arguments: None,
            elements: None,
        }
    }
}
