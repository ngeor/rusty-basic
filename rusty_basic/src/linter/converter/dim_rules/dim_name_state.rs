use crate::linter::converter::dim_rules::dim_list_state::DimListState;
use crate::linter::type_resolver::TypeResolver;
use crate::parser::TypeQualifier;
use rusty_common::{HasLocation, Location};
use std::ops::{Deref, DerefMut};

pub struct DimNameState<'a, 'b> {
    ctx: &'a mut DimListState<'b>,
    shared: bool,
    pos: Location,
}

impl<'a, 'b> DimNameState<'a, 'b> {
    pub fn new(ctx: &'a mut DimListState<'b>, shared: bool, pos: Location) -> Self {
        Self { ctx, shared, pos }
    }

    pub fn is_shared(&self) -> bool {
        self.shared
    }
}

impl<'a, 'b> Deref for DimNameState<'a, 'b> {
    type Target = DimListState<'b>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a, 'b> DerefMut for DimNameState<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ctx
    }
}

impl<'a, 'b> HasLocation for DimNameState<'a, 'b> {
    fn pos(&self) -> Location {
        self.pos
    }
}

impl<'a, 'b> TypeResolver for DimNameState<'a, 'b> {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.ctx.char_to_qualifier(ch)
    }
}
