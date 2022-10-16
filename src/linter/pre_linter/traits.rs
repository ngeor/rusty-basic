use crate::linter::{FunctionMap, SubMap};
use crate::parser::UserDefinedTypes;
use std::rc::Rc;

pub trait HasFunctions {
    fn functions(&self) -> &FunctionMap;
}

impl<T: HasFunctions> HasFunctions for Rc<T> {
    fn functions(&self) -> &FunctionMap {
        self.as_ref().functions()
    }
}

pub trait HasSubs {
    fn subs(&self) -> &SubMap;
}

impl<T: HasSubs> HasSubs for Rc<T> {
    fn subs(&self) -> &SubMap {
        self.as_ref().subs()
    }
}

pub trait HasUserDefinedTypes {
    fn user_defined_types(&self) -> &UserDefinedTypes;
}

impl<T: HasUserDefinedTypes> HasUserDefinedTypes for Rc<T> {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        self.as_ref().user_defined_types()
    }
}
