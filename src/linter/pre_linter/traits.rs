use crate::linter::{FunctionMap, SubMap};
use crate::parser::UserDefinedTypes;

pub trait HasFunctions {
    fn functions(&self) -> &FunctionMap;
}

pub trait HasSubs {
    fn subs(&self) -> &SubMap;
}

pub trait HasUserDefinedTypes {
    fn user_defined_types(&self) -> &UserDefinedTypes;
}
