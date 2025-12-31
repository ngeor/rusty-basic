use rusty_common::{CaseInsensitiveString, Positioned};

pub type BareName = CaseInsensitiveString;
pub type BareNamePos = Positioned<BareName>;

pub trait AsBareName {
    fn as_bare_name(&self) -> &BareName;
}

pub trait ToBareName {
    fn to_bare_name(self) -> BareName;
}

impl AsBareName for BareName {
    fn as_bare_name(&self) -> &BareName {
        self
    }
}

impl ToBareName for BareName {
    fn to_bare_name(self) -> BareName {
        self
    }
}

impl<T> AsBareName for Positioned<T>
where
    T: AsBareName,
{
    fn as_bare_name(&self) -> &BareName {
        self.element.as_bare_name()
    }
}

impl<T> ToBareName for Positioned<T>
where
    T: ToBareName,
{
    fn to_bare_name(self) -> BareName {
        self.element.to_bare_name()
    }
}
