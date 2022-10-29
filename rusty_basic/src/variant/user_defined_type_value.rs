use super::Variant;
use crate::parser::{Element, UserDefinedTypes};
use crate::variant::AsciiSize;
use rusty_common::{CaseInsensitiveString, IndexedMap, Locatable};

/// Holds a value of a user defined type.
#[derive(Clone, Debug)]
pub struct UserDefinedTypeValue {
    map: IndexedMap<CaseInsensitiveString, Variant>,
}

impl UserDefinedTypeValue {
    pub fn new(type_name: &CaseInsensitiveString, types: &UserDefinedTypes) -> Self {
        let user_defined_type = types.get(type_name).expect("could not find type");
        let mut map: IndexedMap<CaseInsensitiveString, Variant> = IndexedMap::new();
        for Locatable {
            element: Element {
                name, element_type, ..
            },
            ..
        } in user_defined_type.elements()
        {
            let def_value: Variant = element_type.default_variant(types);
            map.insert(name.clone(), def_value);
        }

        Self { map }
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

    pub fn address_offset_of_property(&self, property: &CaseInsensitiveString) -> usize {
        self.map
            .keys()
            .take_while(|p| *p != property)
            .map(|p| self.property_size_in_bytes(p))
            .sum()
    }

    fn property_size_in_bytes(&self, property: &CaseInsensitiveString) -> usize {
        self.map
            .get(property)
            .map(Variant::ascii_size)
            .unwrap_or_default()
    }
}

impl AsciiSize for UserDefinedTypeValue {
    fn ascii_size(&self) -> usize {
        self.map.values().map(Variant::ascii_size).sum()
    }
}
