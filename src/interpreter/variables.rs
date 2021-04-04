use crate::common::IndexedMap;
use crate::instruction_generator::Path;
use crate::interpreter::arguments::{ArgumentInfo, Arguments};
use crate::parser::{
    BareName, DimName, DimType, Name, ParamName, ParamType, QualifiedName, TypeQualifier,
};
use crate::variant::{Variant, V_FALSE};
use std::slice::Iter;

#[derive(Debug)]
pub struct Variables {
    map: IndexedMap<Name, Variant>,
    arg_paths: Vec<Option<Path>>,
}

impl Variables {
    pub fn new() -> Self {
        Self {
            map: IndexedMap::new(),
            arg_paths: Vec::new(),
        }
    }

    pub fn insert_built_in(
        &mut self,
        bare_name: BareName,
        qualifier: TypeQualifier,
        value: Variant,
    ) {
        self.insert(QualifiedName::new(bare_name, qualifier).into(), value);
    }

    pub fn insert_user_defined(&mut self, bare_name: BareName, value: Variant) {
        self.insert(bare_name.into(), value);
    }

    fn insert_unnamed(&mut self, value: Variant, arg_path: Option<Path>) {
        let dummy_name = format!("{}", self.map.len());
        self.map
            .insert(Name::new(BareName::new(dummy_name), None), value);
        self.arg_paths.push(arg_path);
    }

    pub fn insert_param(&mut self, param_name: ParamName, value: Variant) {
        self.insert(Self::param_to_name(param_name), value);
    }

    fn param_to_name(param_name: ParamName) -> Name {
        let ParamName {
            bare_name,
            param_type,
        } = param_name;
        match param_type {
            ParamType::Bare => panic!("Unresolved param {:?}", bare_name),
            ParamType::BuiltIn(q, _) => Name::new(bare_name, Some(q)),
            ParamType::UserDefined(_) => Name::new(bare_name, None),
            ParamType::Array(boxed_param_type) => {
                let dummy_param = ParamName::new(bare_name, *boxed_param_type);
                Self::param_to_name(dummy_param)
            }
        }
    }

    pub fn insert(&mut self, name: Name, value: Variant) {
        self.map.insert(name, value);
    }

    pub fn insert_dim(&mut self, dim_name: DimName, value: Variant) {
        let DimName {
            bare_name,
            dim_type,
            ..
        } = dim_name;
        self.insert_dim_internal(bare_name, dim_type, value);
    }

    fn insert_dim_internal(&mut self, bare_name: BareName, dim_type: DimType, value: Variant) {
        match dim_type {
            DimType::BuiltIn(qualifier, _) => {
                self.insert_built_in(bare_name, qualifier, value);
            }
            DimType::FixedLengthString(_, _) => {
                self.insert_built_in(bare_name, TypeQualifier::DollarString, value);
            }
            DimType::UserDefined(_) => {
                self.insert_user_defined(bare_name, value);
            }
            DimType::Array(_, box_element_type) => {
                self.insert_dim_internal(bare_name, *box_element_type, value);
            }
            DimType::Bare => panic!("Unresolved type"),
        }
    }

    pub fn get_or_create(&mut self, name: Name) -> &mut Variant {
        self.map.get_or_crate(name, Self::default_value_for_name)
    }

    // This is needed only when we're setting the default value for a function
    // that hasn't set a return value. As functions can only return built-in types,
    // the value for unqualified names is not important.
    fn default_value_for_name(name: &Name) -> Variant {
        if let Some(q) = name.qualifier() {
            Variant::from(q)
        } else {
            V_FALSE
        }
    }

    /// Gets the number of variables in this object.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Gets an iterator that returns the variables in this object.
    pub fn iter(&self) -> Iter<Variant> {
        self.map.values()
    }

    pub fn get_built_in(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&Variant> {
        self.get_by_name(&QualifiedName::new(bare_name.clone(), qualifier).into())
    }

    pub fn get_user_defined(&self, bare_name: &BareName) -> Option<&Variant> {
        self.get_by_name(&bare_name.clone().into())
    }

    pub fn get_by_name(&self, name: &Name) -> Option<&Variant> {
        self.map.get(name)
    }

    pub fn get(&self, idx: usize) -> Option<&Variant> {
        self.map.get_by_index(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Variant> {
        self.map.get_by_index_mut(idx)
    }

    pub fn apply_arguments(&mut self, arguments: Arguments) {
        for ArgumentInfo {
            value,
            param_name,
            arg_path,
        } in arguments.into_iter()
        {
            match param_name {
                Some(param_name) => self.insert_param(param_name, value),
                None => self.insert_unnamed(value, arg_path),
            }
        }
    }

    pub fn get_by_dim_name(&self, dim_name: &DimName) -> Option<&Variant> {
        self.get_by_dim_name_internal(dim_name.bare_name(), dim_name.dim_type())
    }

    fn get_by_dim_name_internal(
        &self,
        bare_name: &BareName,
        dim_type: &DimType,
    ) -> Option<&Variant> {
        match dim_type {
            DimType::BuiltIn(q, _) => self.get_built_in(bare_name, *q),
            DimType::FixedLengthString(_, _) => {
                self.get_built_in(bare_name, TypeQualifier::DollarString)
            }
            DimType::UserDefined(_) => self.get_user_defined(bare_name),
            DimType::Array(_, box_dim_type) => {
                self.get_by_dim_name_internal(bare_name, box_dim_type.as_ref())
            }
            DimType::Bare => panic!("Unresolved dim"),
        }
    }

    pub fn get_path(&self, idx: usize) -> Option<&Path> {
        match self.arg_paths.get(idx) {
            Some(opt_path) => opt_path.as_ref(),
            _ => None,
        }
    }

    pub fn calculate_var_ptr(&self, name: &Name) -> usize {
        debug_assert!(self.map.get(name).is_some());
        self.map
            .keys()
            .take_while(|k| *k != name)
            .map(|k| self.map.get(k).unwrap())
            .map(Variant::size_in_bytes)
            .sum()
    }
}

impl From<Arguments> for Variables {
    fn from(arguments: Arguments) -> Self {
        let mut variables: Self = Self::new();
        variables.apply_arguments(arguments);
        variables
    }
}
