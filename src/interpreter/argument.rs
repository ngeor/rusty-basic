use crate::linter::DimName;
use crate::variant::Variant;

#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    ByVal(Variant),
    ByRef(DimName),
}

impl From<Variant> for Argument {
    fn from(v: Variant) -> Self {
        Self::ByVal(v)
    }
}

impl From<DimName> for Argument {
    fn from(n: DimName) -> Self {
        Self::ByRef(n)
    }
}
