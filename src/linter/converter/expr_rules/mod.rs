use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::Convertible;
use crate::parser::*;
use crate::variant::Variant;
use expr_state::ExprState;
use pos_expr_state::PosExprState;
use std::convert::TryFrom;

mod binary;
mod built_in_function;
mod function;
mod property;
mod unary;
mod variable;

/// Indicates the context in which an expression is being resolved.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExprContext {
    /// Default context (typically r-side expression)
    Default,

    /// Assignment (typically l-side expression)
    Assignment,

    /// Function or sub argument
    Argument,

    /// Used in resolving left-side of property expressions
    ResolvingPropertyOwner,
}

impl Default for ExprContext {
    fn default() -> Self {
        Self::Default
    }
}

mod expr_state {
    use crate::linter::converter::converter::Context;
    use crate::linter::converter::expr_rules::ExprContext;
    use crate::linter::converter::traits::FromParentContext;
    use std::ops::{Deref, DerefMut};

    /// A context that is used when converting an [ExpressionNode].
    /// Enhances the parent [Context] with an [ExprContext].
    pub struct ExprState<'a> {
        ctx: &'a mut Context,
        expr_context: ExprContext,
    }

    impl<'a> ExprState<'a> {
        pub fn new(ctx: &'a mut Context, expr_context: ExprContext) -> Self {
            Self { ctx, expr_context }
        }

        pub fn expr_context(&self) -> ExprContext {
            self.expr_context
        }
    }

    impl<'a> FromParentContext<'a, Context, ExprContext> for ExprState<'a> {
        fn create_from_parent_context(parent: &'a mut Context, value: ExprContext) -> Self {
            Self::new(parent, value)
        }
    }

    impl<'a> Deref for ExprState<'a> {
        type Target = Context;

        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a> DerefMut for ExprState<'a> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }
}

mod pos_expr_state {

    //
    // PosExprState
    //

    use crate::common::{HasLocation, Location};
    use crate::linter::converter::expr_rules::expr_state::ExprState;
    use crate::linter::converter::traits::FromParentContext;
    use std::ops::{Deref, DerefMut};

    pub struct PosExprState<'a, 'b> {
        ctx: &'a mut ExprState<'b>,
        pos: Location,
    }

    impl<'a, 'b> PosExprState<'a, 'b> {
        pub fn new(ctx: &'a mut ExprState<'b>, pos: Location) -> Self {
            Self { ctx, pos }
        }
    }

    impl<'a, 'b> FromParentContext<'a, ExprState<'b>, Location> for PosExprState<'a, 'b> {
        fn create_from_parent_context(parent: &'a mut ExprState<'b>, value: Location) -> Self {
            Self::new(parent, value)
        }
    }

    impl<'a, 'b> Deref for PosExprState<'a, 'b> {
        type Target = ExprState<'b>;

        fn deref(&self) -> &Self::Target {
            self.ctx
        }
    }

    impl<'a, 'b> DerefMut for PosExprState<'a, 'b> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.ctx
        }
    }

    impl<'a, 'b> HasLocation for PosExprState<'a, 'b> {
        fn pos(&self) -> Location {
            self.pos
        }
    }
}

//
// ExpressionNode Convertible
//

impl<'a> Convertible<ExprState<'a>> for ExpressionNode {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, QErrorNode> {
        let Locatable { element: expr, pos } = self;
        match expr.convert_in(ctx, pos) {
            Ok(expr) => Ok(expr.at(pos)),
            Err(err) => Err(err.patch_pos(pos)),
        }
    }
}

impl<'a> Convertible<ExprState<'a>> for Box<ExpressionNode> {
    fn convert(self, ctx: &mut ExprState<'a>) -> Result<Self, QErrorNode> {
        let unboxed = *self;
        unboxed.convert(ctx).map(Box::new)
    }
}

//
// Expression Convertible
//

impl<'a, 'b> Convertible<PosExprState<'a, 'b>> for Expression {
    fn convert(self, ctx: &mut PosExprState<'a, 'b>) -> Result<Self, QErrorNode> {
        match self {
            // literals
            Expression::SingleLiteral(_)
            | Expression::DoubleLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::IntegerLiteral(_)
            | Expression::LongLiteral(_) => Ok(self),
            // parenthesis
            Expression::Parenthesis(box_child) => {
                box_child.convert(ctx).map(Expression::Parenthesis)
            }
            // unary
            Expression::UnaryExpression(unary_operator, box_child) => {
                unary::convert(ctx, unary_operator, *box_child)
            }
            // binary
            Expression::BinaryExpression(binary_operator, left, right, _expr_type) => {
                binary::convert(ctx, binary_operator, *left, *right)
            }
            // variables
            Expression::Variable(name, variable_info) => {
                variable::convert(ctx, name, variable_info)
            }
            Expression::ArrayElement(_name, _indices, _variable_info) => {
                panic!(
                    "Parser is not supposed to produce any ArrayElement expressions, only FunctionCall"
                )
            }
            Expression::Property(box_left_side, property_name, _expr_type) => {
                property::convert(ctx, box_left_side, property_name)
            }
            // function call
            Expression::FunctionCall(name, args) => function::convert(ctx, name, args),
            Expression::BuiltInFunctionCall(built_in_function, args) => {
                built_in_function::convert(ctx, built_in_function, args)
            }
        }
    }
}
