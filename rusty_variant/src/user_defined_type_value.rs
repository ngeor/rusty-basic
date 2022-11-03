use crate::Variant;
use rusty_common::{CaseInsensitiveString, IndexedMap};

/// Holds a value of a user defined type.
#[derive(Clone, Debug)]
pub struct UserDefinedTypeValue {
    map: IndexedMap<CaseInsensitiveString, Variant>,
}

impl UserDefinedTypeValue {
    pub fn new(map: IndexedMap<CaseInsensitiveString, Variant>) -> Self {
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

    pub fn property_keys(&self) -> impl Iterator<Item = &CaseInsensitiveString> + '_ {
        self.map.keys()
    }

    pub fn property_values(&self) -> impl Iterator<Item = &Variant> + '_ {
        self.map.values()
    }
}
