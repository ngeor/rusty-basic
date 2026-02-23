#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameScope {
    Global,
    Sub,
    Function,
}
