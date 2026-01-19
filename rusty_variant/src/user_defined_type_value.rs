use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use rusty_common::CaseInsensitiveString;

use crate::Variant;

/// Holds a value of a user defined type.
#[derive(Clone, Debug)]
pub struct UserDefinedTypeValue {
    name_to_value: HashMap<CaseInsensitiveString, Variant>,
    ordered_property_names: Vec<CaseInsensitiveString>,
}

impl UserDefinedTypeValue {
    pub fn new(arr: Vec<(CaseInsensitiveString, Variant)>) -> Self {
        let mut name_to_value: HashMap<CaseInsensitiveString, Variant> = HashMap::new();
        let mut ordered_property_names: Vec<CaseInsensitiveString> = Vec::new();
        for (n, v) in arr {
            ordered_property_names.push(n.clone());
            name_to_value.insert(n, v);
        }
        Self {
            name_to_value,
            ordered_property_names,
        }
    }

    pub fn get<K>(&self, name: &K) -> Option<&Variant>
    where
        CaseInsensitiveString: Borrow<K>,
        K: Eq + Hash + ?Sized,
    {
        self.name_to_value.get(name)
    }

    pub fn get_mut<K>(&mut self, name: &K) -> Option<&mut Variant>
    where
        CaseInsensitiveString: Borrow<K>,
        K: Eq + Hash + ?Sized,
    {
        self.name_to_value.get_mut(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &CaseInsensitiveString> {
        self.ordered_property_names.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &Variant> + '_ {
        self.names().map(|name| self.get(name).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_by_case_insensitive_string() {
        let name = CaseInsensitiveString::from("Card");
        let u = UserDefinedTypeValue::new(vec![(name.clone(), Variant::VInteger(42))]);
        assert_eq!(u.get(&name), Some(&Variant::VInteger(42)));
    }

    #[test]
    fn test_get_mut() {
        let name = CaseInsensitiveString::from("Address");
        let mut u =
            UserDefinedTypeValue::new(vec![(name.to_owned(), Variant::VString("home".to_owned()))]);
        if let Some(v) = u.get_mut(&name) {
            *v = Variant::VString("work".to_owned());
        }
        assert_eq!(u.get(&name), Some(&Variant::VString("work".to_owned())));
    }

    #[test]
    fn test_names() {
        let u = UserDefinedTypeValue::new(vec![
            (CaseInsensitiveString::from("Row"), Variant::VInteger(1)),
            (CaseInsensitiveString::from("Col"), Variant::VInteger(2)),
        ]);
        let names: Vec<&CaseInsensitiveString> = u.names().collect();
        assert_eq!(
            names,
            vec![
                &CaseInsensitiveString::from("Row"),
                &CaseInsensitiveString::from("Col")
            ]
        );
    }

    #[test]
    fn test_values() {
        let u = UserDefinedTypeValue::new(vec![
            (CaseInsensitiveString::from("Row"), Variant::VInteger(1)),
            (CaseInsensitiveString::from("Col"), Variant::VInteger(2)),
        ]);
        let values: Vec<Variant> = u.values().cloned().collect();
        assert_eq!(values, vec![Variant::VInteger(1), Variant::VInteger(2)]);
    }
}
