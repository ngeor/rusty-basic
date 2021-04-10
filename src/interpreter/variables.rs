use crate::common::{IndexedMap, QError};
use crate::instruction_generator::Path;
use crate::interpreter::arguments::{ArgumentInfo, Arguments};
use crate::interpreter::context::Segment;
use crate::parser::{
    BareName, DimName, DimType, Name, ParamName, ParamType, QualifiedName, TypeQualifier,
};
use crate::variant::{Variant, V_FALSE};

#[derive(Debug)]
pub struct Variables {
    map: IndexedMap<Name, RuntimeVariableInfo>,
}

#[derive(Debug)]
struct RuntimeVariableInfo {
    /// Holds the value of the variable.
    value: Variant,

    /// For anonymous by ref arguments, holds a resolved path that can be used
    /// to find the variable in the parent context.
    arg_path: Option<Path>,
}

impl RuntimeVariableInfo {
    pub fn new(value: Variant, arg_path: Option<Path>) -> Self {
        Self { value, arg_path }
    }
}

impl Variables {
    pub fn new() -> Self {
        Self {
            map: IndexedMap::new(),
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
        let name = Name::new(BareName::new(dummy_name), None);
        self.map
            .insert(name, RuntimeVariableInfo::new(value, arg_path));
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
        self.map.insert(name, RuntimeVariableInfo::new(value, None));
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
        &mut self
            .map
            .get_or_create(name, |n| {
                RuntimeVariableInfo::new(Self::default_value_for_name(n), None)
            })
            .value
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
    pub fn iter(&self) -> impl Iterator<Item = &Variant> {
        self.map.values().map(|r| &r.value)
    }

    pub fn get_built_in(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&Variant> {
        self.get_by_name(&QualifiedName::new(bare_name.clone(), qualifier).into())
    }

    pub fn get_user_defined(&self, bare_name: &BareName) -> Option<&Variant> {
        // TODO make a structure that allows to lookup by BareName and QualifiedName without the need to clone
        self.get_by_name(&bare_name.clone().into())
    }

    pub fn get_by_name(&self, name: &Name) -> Option<&Variant> {
        self.map.get(name).map(|r| &r.value)
    }

    pub fn get_by_name_mut(&mut self, name: &Name) -> Option<&mut Variant> {
        self.map.get_mut(name).map(|r| &mut r.value)
    }

    pub fn get(&self, idx: usize) -> Option<&Variant> {
        self.map.get_by_index(idx).map(|r| &r.value)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Variant> {
        self.map.get_by_index_mut(idx).map(|r| &mut r.value)
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

    pub fn get_arg_path(&self, idx: usize) -> Option<&Path> {
        self.map.get_by_index(idx).and_then(|r| r.arg_path.as_ref())
    }

    pub fn calculate_var_ptr(&self, name: &Name) -> usize {
        debug_assert!(self.map.get(name).is_some());
        self.map
            .keys()
            .take_while(|k| *k != name)
            .map(|k| self.get_by_name(k).unwrap())
            .map(Variant::size_in_bytes)
            .sum()
    }

    pub fn number_of_arrays(&self) -> usize {
        self.map
            .values()
            .map(|RuntimeVariableInfo { value, .. }| value)
            .filter(|v| v.is_array())
            .count()
    }

    pub fn number_of_arrays_until(&self, name: &Name) -> usize {
        let mut result: usize = 0;
        for key in self.map.keys() {
            if self.get_by_name(key).unwrap().is_array() {
                result += 1;
            }
            if key == name {
                break;
            }
        }
        result
    }

    pub fn segments(&self, index: usize) -> Vec<Segment> {
        let mut result: Vec<Segment> = vec![];
        result.push(Segment::Root(index));
        for name in self.map.keys() {
            let value = self.get_by_name(name).unwrap();
            if value.is_array() {
                result.push(Segment::Array(index, name.clone()));
            }
        }
        result
    }

    pub fn peek_non_array(&self, address: usize) -> Result<u8, QError> {
        let mut offset: usize = 0;
        for RuntimeVariableInfo { value, .. } in self.map.values() {
            if !value.is_array() {
                let len = value.size_in_bytes();
                if offset <= address && address < offset + len {
                    return value.peek_non_array(address - offset);
                }

                offset += len;
            }
        }
        Err(QError::InternalError(
            "Could not find variable at address".to_string(),
        ))
    }

    pub fn poke_non_array(&mut self, address: usize, byte_value: u8) -> Result<(), QError> {
        let mut offset: usize = 0;
        for RuntimeVariableInfo { value, .. } in self.map.values_mut() {
            if !value.is_array() {
                let len = value.size_in_bytes();
                if offset <= address && address < offset + len {
                    return value.poke_non_array(address - offset, byte_value);
                }

                offset += len;
            }
        }
        Err(QError::InternalError(
            "Could not find variable at address".to_string(),
        ))
    }
}

impl From<Arguments> for Variables {
    fn from(arguments: Arguments) -> Self {
        let mut variables: Self = Self::new();
        variables.apply_arguments(arguments);
        variables
    }
}
