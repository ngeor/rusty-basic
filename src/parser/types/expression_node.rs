use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::parser::types::*;
use crate::variant::{Variant, MIN_INTEGER, MIN_LONG};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Variable(Name, VariableInfo),
    FunctionCall(Name, ExpressionNodes),
    ArrayElement(
        // the name of the array (unqualified only for user defined types)
        Name,
        // the array indices
        Vec<ExpressionNode>,
        // the type of the elements (shared refers to the array itself)
        VariableInfo,
    ),
    BuiltInFunctionCall(BuiltInFunction, Vec<ExpressionNode>),
    BinaryExpression(
        Operator,
        Box<ExpressionNode>,
        Box<ExpressionNode>,
        ExpressionType,
    ),
    UnaryExpression(UnaryOperator, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),

    /// A property of a user defined type
    ///
    /// The left side is the expression owning the element,
    /// the right side is the element name.
    ///
    /// Examples:
    ///
    /// - A.B (A left, B right)
    /// - A(1).B ( A(1) left, B right)
    /// - A.B.C (A.B left, C right)
    Property(Box<Expression>, Name, ExpressionType),
}

pub type ExpressionNode = Locatable<Expression>;
pub type ExpressionNodes = Vec<ExpressionNode>;

impl From<f32> for Expression {
    fn from(f: f32) -> Expression {
        Expression::SingleLiteral(f)
    }
}

impl From<f64> for Expression {
    fn from(f: f64) -> Expression {
        Expression::DoubleLiteral(f)
    }
}

impl From<String> for Expression {
    fn from(f: String) -> Expression {
        Expression::StringLiteral(f)
    }
}

impl From<&str> for Expression {
    fn from(f: &str) -> Expression {
        f.to_string().into()
    }
}

impl From<i32> for Expression {
    fn from(f: i32) -> Expression {
        Expression::IntegerLiteral(f)
    }
}

impl From<i64> for Expression {
    fn from(f: i64) -> Expression {
        Expression::LongLiteral(f)
    }
}

impl Expression {
    pub fn from_constant(value: Variant) -> Result<Self, QError> {
        match value {
            Variant::VSingle(f) => Ok(f.into()),
            Variant::VDouble(d) => Ok(d.into()),
            Variant::VString(s) => Ok(s.into()),
            Variant::VInteger(i) => Ok(i.into()),
            Variant::VLong(l) => Ok(l.into()),
            _ => Err(QError::InvalidConstant),
        }
    }

    #[cfg(test)]
    pub fn func(s: &str, args: ExpressionNodes) -> Self {
        let name: Name = s.into();
        Expression::FunctionCall(name, args)
    }

    fn unary_minus(child_node: ExpressionNode) -> Self {
        let Locatable {
            element: child,
            pos,
        } = child_node;
        match child {
            Self::SingleLiteral(n) => Self::SingleLiteral(-n),
            Self::DoubleLiteral(n) => Self::DoubleLiteral(-n),
            Self::IntegerLiteral(n) => {
                if n <= MIN_INTEGER {
                    Self::LongLiteral(-n as i64)
                } else {
                    Self::IntegerLiteral(-n)
                }
            }
            Self::LongLiteral(n) => {
                if n <= MIN_LONG {
                    Self::DoubleLiteral(-n as f64)
                } else {
                    Self::LongLiteral(-n)
                }
            }
            _ => Self::UnaryExpression(
                UnaryOperator::Minus,
                Box::new(child.simplify_unary_minus_literals().at(pos)),
            ),
        }
    }

    pub fn simplify_unary_minus_literals(self) -> Self {
        match self {
            Self::UnaryExpression(op, child) => {
                let x: ExpressionNode = *child;
                match op {
                    UnaryOperator::Minus => Self::unary_minus(x),
                    _ => Self::UnaryExpression(op, Self::simplify_unary_minus_node(x)),
                }
            }
            Self::BinaryExpression(op, left, right, old_expression_type) => Self::BinaryExpression(
                op,
                Self::simplify_boxed_node(left),
                Self::simplify_boxed_node(right),
                old_expression_type,
            ),
            Self::Parenthesis(child) => Self::Parenthesis(Self::simplify_boxed_node(child)),
            Self::FunctionCall(name, args) => Self::FunctionCall(
                name,
                args.into_iter()
                    .map(|a| a.map(|x| x.simplify_unary_minus_literals()))
                    .collect(),
            ),
            _ => self,
        }
    }

    fn simplify_boxed_node(child: Box<ExpressionNode>) -> Box<ExpressionNode> {
        Self::simplify_unary_minus_node(*child)
    }

    fn simplify_unary_minus_node(child: ExpressionNode) -> Box<ExpressionNode> {
        let Locatable { element, pos } = child;
        let simplified = element.simplify_unary_minus_literals();
        Box::new(simplified.at(pos))
    }

    pub fn is_parenthesis(&self) -> bool {
        match self {
            Self::Parenthesis(_) => true,
            _ => false,
        }
    }

    pub fn is_by_ref(&self) -> bool {
        match self {
            Expression::Variable(_, _)
            | Expression::ArrayElement(_, _, _)
            | Expression::Property(_, _, _) => true,
            _ => false,
        }
    }

    /// Returns the name of this `Variable` or `Property` expression.
    /// For properties, this is the concatenated name of all elements in the property path.
    pub fn fold_name(&self) -> Option<Name> {
        match self {
            Self::Variable(n, _) => Some(n.clone()),
            Self::Property(left_side, property_name, _) => match left_side.fold_name() {
                Some(left_side_name) => left_side_name.try_concat_name(property_name.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn left_most_name(&self) -> Option<&Name> {
        match self {
            Self::Variable(n, _) | Self::FunctionCall(n, _) | Self::ArrayElement(n, _, _) => {
                Some(n)
            }
            Self::Property(left_side, _, _) => left_side.left_most_name(),
            _ => None,
        }
    }

    pub fn binary(
        left: ExpressionNode,
        right: ExpressionNode,
        op: Operator,
    ) -> Result<Self, QErrorNode> {
        // get the types
        let t_left = left.expression_type();
        let t_right = right.expression_type();
        // get the cast type
        match t_left.cast_binary_op(t_right, op) {
            Some(type_definition) => Ok(Expression::BinaryExpression(
                op,
                Box::new(left),
                Box::new(right),
                type_definition,
            )),
            None => Err(QError::TypeMismatch).with_err_at(&right),
        }
    }

    #[cfg(test)]
    pub fn var_unresolved(s: &str) -> Self {
        let name: Name = s.into();
        Expression::Variable(name, VariableInfo::unresolved())
    }

    #[cfg(test)]
    pub fn var_resolved(s: &str) -> Self {
        let name: Name = s.into();
        let expression_type = name.expression_type();
        Expression::Variable(name, VariableInfo::new_local(expression_type))
    }

    #[cfg(test)]
    pub fn var_user_defined(name: &str, type_name: &str) -> Self {
        Expression::Variable(
            name.into(),
            VariableInfo::new_local(ExpressionType::UserDefined(type_name.into())),
        )
    }

    fn should_flip_unary(&self, op: UnaryOperator) -> bool {
        match self {
            Expression::BinaryExpression(r_op, _, _, _) => {
                op == UnaryOperator::Minus || r_op.is_binary()
            }
            _ => false,
        }
    }

    fn should_flip_binary(&self) -> bool {
        match self {
            Expression::BinaryExpression(l_op, _, l_right, _) => match &l_right.element {
                Expression::BinaryExpression(r_op, _, _, _) => {
                    l_op.is_arithmetic() && (r_op.is_relational() || r_op.is_binary())
                        || l_op.is_relational() && r_op.is_binary()
                        || *l_op == Operator::And && *r_op == Operator::Or
                        || Self::flip_multiply_plus(l_op, r_op)
                        || Self::flip_plus_minus(l_op, r_op)
                        || Self::flip_multiply_divide(l_op, r_op)
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn flip_multiply_plus(l_op: &Operator, r_op: &Operator) -> bool {
        (l_op.is_multiply_or_divide() || *l_op == Operator::Modulo) && r_op.is_plus_or_minus()
    }

    fn flip_plus_minus(l_op: &Operator, r_op: &Operator) -> bool {
        //
        //  A + B - C is parsed as
        //
        //      +
        //   A     -
        //        B C
        //
        // needs to flip into
        //
        //      -
        //   +    C
        //  A B
        l_op.is_plus_or_minus() && r_op.is_plus_or_minus()
    }

    fn flip_multiply_divide(l_op: &Operator, r_op: &Operator) -> bool {
        l_op.is_multiply_or_divide() && r_op.is_multiply_or_divide()
    }
}

impl ExpressionNode {
    /// Flips a binary expression.
    ///
    /// `A AND B OR C` would be parsed as `A AND (B OR C)` but needs to be `(A AND B) OR C`.
    fn flip_binary(self) -> Self {
        let Self { element, pos } = self;
        if let Expression::BinaryExpression(l_op, l_left, l_right, _) = element {
            let Self {
                element: r_element,
                pos: r_pos,
            } = *l_right;
            if let Expression::BinaryExpression(r_op, r_left, r_right, _) = r_element {
                let new_left = l_left.binary_expr(l_op, *r_left, pos);
                new_left.binary_expr(r_op, *r_right, r_pos)
            } else {
                panic!("should_flip_binary internal error")
            }
        } else {
            panic!("should_flip_binary internal error")
        }
    }

    pub fn simplify_unary_minus_literals(self) -> Self {
        self.map(|x| x.simplify_unary_minus_literals())
    }

    pub fn apply_priority_order(
        self,
        right_side: ExpressionNode,
        op: Operator,
        pos: Location,
    ) -> ExpressionNode {
        self.binary_expr(op, right_side, pos)
    }

    fn binary_expr(self, op: Operator, right_side: Self, pos: Location) -> Self {
        let result = Expression::BinaryExpression(
            op,
            Box::new(self),
            Box::new(right_side),
            ExpressionType::Unresolved,
        )
        .at(pos);
        if result.should_flip_binary() {
            result.flip_binary()
        } else {
            result
        }
    }

    /// Applies the unary operator priority order.
    ///
    /// `NOT A AND B` would be parsed as `NOT (A AND B)`, needs to flip into `(NOT A) AND B`
    pub fn apply_unary_priority_order(self, op: UnaryOperator, op_pos: Location) -> ExpressionNode {
        if self.should_flip_unary(op) {
            let Locatable { element, pos } = self;
            match element {
                Expression::BinaryExpression(r_op, r_left, r_right, _) => {
                    // apply the unary operator to the left of the binary expr
                    let new_left = Expression::UnaryExpression(op, r_left).at(op_pos);
                    // and nest it as left inside a binary expr
                    new_left.binary_expr(r_op, *r_right, pos)
                }
                _ => panic!("should_flip_unary internal error"),
            }
        } else {
            Expression::UnaryExpression(op, Box::new(self)).at(op_pos)
        }
    }
}

impl HasExpressionType for Expression {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::SingleLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            Self::DoubleLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::HashDouble),
            Self::StringLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::DollarString),
            Self::IntegerLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
            Self::LongLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Variable(
                _,
                VariableInfo {
                    expression_type, ..
                },
            )
            | Self::Property(_, _, expression_type)
            | Self::BinaryExpression(_, _, _, expression_type) => expression_type.clone(),
            Self::ArrayElement(
                _,
                args,
                VariableInfo {
                    expression_type, ..
                },
            ) => {
                if args.is_empty() {
                    // this is the entire array
                    ExpressionType::Array(Box::new(expression_type.clone()))
                } else {
                    // an element of the array
                    expression_type.clone()
                }
            }
            Self::FunctionCall(Name::Qualified(QualifiedName { qualifier, .. }), _) => {
                ExpressionType::BuiltIn(*qualifier)
            }
            Self::BuiltInFunctionCall(f, _) => ExpressionType::BuiltIn(f.into()),
            Self::UnaryExpression(_, c) | Self::Parenthesis(c) => c.as_ref().expression_type(),
            Self::FunctionCall(Name::Bare(_), _) => ExpressionType::Unresolved,
        }
    }
}

impl CanCastTo<TypeQualifier> for Expression {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.expression_type().can_cast_to(other)
    }
}

impl<T: HasExpressionType> CanCastTo<&T> for Expression {
    fn can_cast_to(&self, other: &T) -> bool {
        let other_type_definition = other.expression_type();
        self.expression_type().can_cast_to(&other_type_definition)
    }
}

impl CanCastTo<&ExpressionNode> for Expression {
    fn can_cast_to(&self, other: &ExpressionNode) -> bool {
        self.can_cast_to(&other.element)
    }
}
