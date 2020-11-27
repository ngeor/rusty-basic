use super::Variant;
use crate::common::{CaseInsensitiveString, Locatable};
use crate::parser::UserDefinedTypes;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct UserDefinedTypeValue {
    type_name: CaseInsensitiveString,
    map: HashMap<CaseInsensitiveString, Variant>,
}

impl UserDefinedTypeValue {
    pub fn new(type_name: &CaseInsensitiveString, types: &UserDefinedTypes) -> Self {
        let user_defined_type = types.get(type_name).expect("could not find type");
        let mut map: HashMap<CaseInsensitiveString, Variant> = HashMap::new();
        for Locatable { element, .. } in user_defined_type.elements() {
            let def_value: Variant = element.element_type().default_variant(types);
            map.insert(element.as_ref().clone(), def_value);
        }

        Self {
            type_name: type_name.clone(),
            map,
        }
    }

    /// Gets the name of the user defined type.
    ///
    /// Currently used only by the LEN function.
    pub fn type_name(&self) -> &CaseInsensitiveString {
        &self.type_name
    }

    pub fn get(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.map.get(name)
    }

    pub fn get_mut(&mut self, name: &CaseInsensitiveString) -> Option<&mut Variant> {
        self.map.get_mut(name)
    }

    pub fn get_path(&self, names: &[CaseInsensitiveString]) -> Option<&Variant> {
        let (first, rest) = names.split_first().expect("empty names!");
        if rest.is_empty() {
            self.get(first)
        } else {
            let first_variant = self.get(first).expect("member missing!");
            match first_variant {
                Variant::VUserDefined(user_defined_value) => user_defined_value.get_path(rest),
                _ => panic!("cannot navigate simple variant"),
            }
        }
    }

    pub fn insert_path(&mut self, names: &[CaseInsensitiveString], value: Variant) {
        let (first, rest) = names.split_first().expect("empty names!");
        if rest.is_empty() {
            self.map.insert(first.clone(), value);
        } else {
            let first_variant = self.get_mut(first).expect("member missing!");
            match first_variant {
                Variant::VUserDefined(user_defined_value) => {
                    user_defined_value.insert_path(rest, value);
                }
                _ => panic!("cannot navigate simple variant"),
            }
        }
    }
}
