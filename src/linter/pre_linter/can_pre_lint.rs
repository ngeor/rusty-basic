use crate::common::{HasLocation, Locatable, PatchErrPos, QErrorNode};
use std::rc::Rc;

/// Represents something that can be checked by the pre-linter.
pub trait CanPreLint {
    /// The context of the inspection.
    type Context;
    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode>;
}

// blanket for Locatable, if the T type is using an Rc<C> for a context.
// creates a new temporary context with is enhanced with the current Location.

impl<T, C> CanPreLint for Locatable<T>
where
    T: CanPreLint<Context = Locatable<Rc<C>>>,
{
    type Context = Rc<C>;

    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        let pos = self.pos();
        let pos_context = Locatable::new(Rc::clone(context), pos);
        self.as_ref().pre_lint(&pos_context).patch_err_pos(self)
    }
}

// blanket for Vec

impl<T> CanPreLint for Vec<T>
where
    T: CanPreLint,
{
    type Context = T::Context;

    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        self.iter().try_for_each(|item| item.pre_lint(context))
    }
}
