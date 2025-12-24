use crate::converter::dim_rules::dim_list_state::DimListState;
use crate::core::TypeResolver;
use rusty_common::{HasPos, Position};
use rusty_parser::TypeQualifier;
use std::ops::{Deref, DerefMut};

pub struct DimNameState<'a, 'b> {
    ctx: &'a mut DimListState<'b>,
    shared: bool,
    pos: Position,
}

impl<'a, 'b> DimNameState<'a, 'b> {
    pub fn new(ctx: &'a mut DimListState<'b>, shared: bool, pos: Position) -> Self {
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

impl<'a, 'b> HasPos for DimNameState<'a, 'b> {
    fn pos(&self) -> Position {
        self.pos
    }
}

impl<'a, 'b> TypeResolver for DimNameState<'a, 'b> {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.ctx.char_to_qualifier(ch)
    }
}
