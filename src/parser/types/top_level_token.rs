use super::{Block, Name, Statement};

#[derive(Debug, PartialEq)]
pub enum TopLevelToken {
    FunctionDeclaration(Name, Vec<Name>),
    Statement(Statement),
    FunctionImplementation(Name, Vec<Name>, Block),
}
