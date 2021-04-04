use super::Variant;
use crate::common::{CaseInsensitiveString, Locatable};
use crate::parser::{Element, UserDefinedTypes};
use std::collections::HashMap;

/// Holds a value of a user defined type.
#[derive(Clone, Debug)]
pub struct UserDefinedTypeValue {
    ordered_property_names: Vec<CaseInsensitiveString>,
    property_names_to_values: HashMap<CaseInsensitiveString, Variant>,
}

impl UserDefinedTypeValue {
    pub fn new(type_name: &CaseInsensitiveString, types: &UserDefinedTypes) -> Self {
        let user_defined_type = types.get(type_name).expect("could not find type");
        let mut property_names_to_values: HashMap<CaseInsensitiveString, Variant> = HashMap::new();
        let mut ordered_property_names: Vec<CaseInsensitiveString> = vec![];
        for Locatable {
            element: Element {
                name, element_type, ..
            },
            ..
        } in user_defined_type.elements()
        {
            let def_value: Variant = element_type.default_variant(types);
            property_names_to_values.insert(name.clone(), def_value);
            ordered_property_names.push(name.clone());
        }

        Self {
            property_names_to_values,
            ordered_property_names,
        }
    }

    pub fn get(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.property_names_to_values.get(name)
    }

    pub fn get_mut(&mut self, name: &CaseInsensitiveString) -> Option<&mut Variant> {
        self.property_names_to_values.get_mut(name)
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
            self.property_names_to_values.insert(first.clone(), value);
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
        self.property_names_to_values
            .values()
            .map(Variant::size_in_bytes)
            .sum()
    }

    pub fn address_of_property(&self, property: &CaseInsensitiveString) -> usize {
        self.ordered_property_names
            .iter()
            .take_while(|p| *p != property)
            .map(|p| {
                self.property_names_to_values
                    .get(p)
                    .unwrap()
                    .size_in_bytes()
            })
            .sum()
    }
}
