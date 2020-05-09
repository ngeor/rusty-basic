use crate::common::Locatable;

pub type Error = Locatable<String>;
pub type Result<T> = std::result::Result<T, Error>;
