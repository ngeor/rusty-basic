use crate::linter::DimName;
use crate::parser::BareName;
use crate::variant::Variant;

pub enum Path {
    Root(DimName),
    ArrayElement(Box<Path>, Vec<Variant>),
    Property(Box<Path>, BareName),
}

impl Path {
    pub fn append_array_element(self, index: Variant) -> Self {
        match self {
            Self::Root(r) => Self::ArrayElement(Box::new(Self::Root(r)), vec![index]),
            Self::ArrayElement(parent, mut indices) => {
                indices.push(index);
                Self::ArrayElement(parent, indices)
            }
            _ => panic!("unexpected NamePtr"),
        }
    }
}
