use super::Variant;
use crate::common::{CaseInsensitiveString, Locatable};
use crate::parser::{Element, UserDefinedTypes};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct UserDefinedTypeValue {
    type_name: CaseInsensitiveString,
    map: HashMap<CaseInsensitiveString, Variant>,
    indices: Vec<CaseInsensitiveString>,
}

impl UserDefinedTypeValue {
    pub fn new(type_name: &CaseInsensitiveString, types: &UserDefinedTypes) -> Self {
        let user_defined_type = types.get(type_name).expect("could not find type");
        let mut map: HashMap<CaseInsensitiveString, Variant> = HashMap::new();
        let mut indices: Vec<CaseInsensitiveString> = vec![];
        for Locatable {
            element: Element {
                name, element_type, ..
            },
            ..
        } in user_defined_type.elements()
        {
            let def_value: Variant = element_type.default_variant(types);
            map.insert(name.clone(), def_value);
            indices.push(name.clone());
        }

        Self {
            type_name: type_name.clone(),
            map,
            indices,
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

    pub fn size_in_bytes(&self) -> usize {
        self.map.values().map(Variant::size_in_bytes).sum()
    }

    pub fn address_of_property(&self, property: &CaseInsensitiveString) -> usize {
        self.indices
            .iter()
            .take_while(|p| *p != property)
            .map(|p| self.map.get(p).unwrap().size_in_bytes())
            .sum()
    }
}
