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
        matches!(
            self,
            Self::Less
                | Self::LessOrEqual
                | Self::Equal
                | Self::GreaterOrEqual
                | Self::Greater
                | Self::NotEqual
        )
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Plus | Self::Minus | Self::Multiply | Self::Divide | Self::Modulo
        )
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    pub fn is_plus_or_minus(&self) -> bool {
        matches!(self, Self::Plus | Self::Minus)
    }

    pub fn is_multiply_or_divide(&self) -> bool {
        matches!(self, Self::Multiply | Self::Divide)
    }
}
