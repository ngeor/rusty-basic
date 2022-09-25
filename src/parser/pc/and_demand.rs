//
// And Demand
//

use crate::parser::pc::{HasOutput, NonOptSeq2};

pub trait AndDemandTrait: Sized + HasOutput {
    fn and_demand<R>(self, right: R) -> NonOptSeq2<Self, R>;
}

impl<S> AndDemandTrait for S
where
    S: HasOutput,
{
    fn and_demand<R>(self, right: R) -> NonOptSeq2<Self, R> {
        NonOptSeq2::new(self, right)
    }
}
