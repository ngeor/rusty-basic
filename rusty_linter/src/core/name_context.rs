#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameContext {
    Global,
    Sub,
    Function,
}
