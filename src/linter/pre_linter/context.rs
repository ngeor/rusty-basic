use crate::common::Locatable;
use crate::linter::pre_linter::sub_program_context::{FunctionContext, SubContext};
use crate::linter::pre_linter::ConstantMap;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{TypeQualifier, UserDefinedTypes};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

macro_rules! ref_cell_struct {
    ($vis:vis struct $inner_struct_name:ident with cell struct $outer_struct_name:ident {
        $($field:ident: $field_type:tt $(read $read_fn:ident)? $(write $write_fn:ident)? ),+$(,)?
    }) => {
        $vis struct $inner_struct_name {
            // public fields because it is going to be used for destructuring
            $(pub $field: $field_type),+
        }

        $vis struct $outer_struct_name {
            $($field: RefCell<$field_type>),+
        }

        impl $outer_struct_name {
            // assumes all types have a ::new()
            pub fn new() -> Self {
                Self {
                    $($field: RefCell::new($field_type::new())),+
                }
            }

            $(
                // read accessor
                $(
                pub fn $read_fn(&self) -> Ref<'_, $field_type> {
                    self.$field.borrow()
                }
                )?

                // write accessor
                $(
                pub fn $write_fn(&self) -> RefMut<'_, $field_type> {
                    self.$field.borrow_mut()
                }
                )?
            )+

            // break out of the RefCells and return the inner structure

            pub fn into_inner(self) -> $inner_struct_name {
                $inner_struct_name {
                    $($field: self.$field.into_inner()),+
                }
            }
        }
    };
}

// Main context of the pre-linter

ref_cell_struct!(
    pub struct MainContextInner with cell struct MainContext {
        resolver: TypeResolverImpl write resolver_mut,
        user_defined_types: UserDefinedTypes read user_defined_types write user_defined_types_mut,
        functions: FunctionContext write functions_mut,
        subs: SubContext write subs_mut,
        global_constants: ConstantMap read global_constants write global_constants_mut,
    }
);

impl TypeResolver for MainContext {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().char_to_qualifier(ch)
    }
}

pub type MainContextWithPos = Locatable<Rc<MainContext>>;
