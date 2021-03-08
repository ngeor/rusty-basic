#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Operator {
    // relational
    Less,
    LessOrEqual,
    Equal,
    GreaterOrEqual,
    Greater,
    NotEqual,
    // arithmetic
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    // binary
    And,
    Or,
}

impl Operator {
    pub fn is_relational(&self) -> bool {
        match self {
            Self::Less
            | Self::LessOrEqual
            | Self::Equal
            | Self::GreaterOrEqual
            | Self::Greater
            | Self::NotEqual => true,
            _ => false,
        }
    }

    pub fn is_arithmetic(&self) -> bool {
        match self {
            Self::Plus | Self::Minus | Self::Multiply | Self::Divide | Self::Modulo => true,
            _ => false,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Self::And | Self::Or => true,
            _ => false,
        }
    }
}
