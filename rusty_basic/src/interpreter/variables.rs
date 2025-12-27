use crate::instruction_generator::Path;
use crate::interpreter::arguments::{ArgumentInfo, Arguments};
use crate::interpreter::byte_size::QByteSize;
use crate::interpreter::handlers::allocation::allocate_built_in;
use crate::interpreter::indexed_map::IndexedMap;
use rusty_parser::{BareName, DimType, DimVar, Name, ParamType, Parameter, TypeQualifier};
use rusty_variant::{Variant, V_FALSE};

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
        self.insert(Name::qualified(bare_name, qualifier), value);
    }

    pub fn insert_user_defined(&mut self, bare_name: BareName, value: Variant) {
        self.insert(Name::bare(bare_name), value);
    }

    fn insert_unnamed(&mut self, value: Variant, arg_path: Option<Path>) {
        let dummy_name = format!("{}", self.map.len());
        let name = Name::bare(BareName::new(dummy_name));
        self.map
            .insert(name, RuntimeVariableInfo::new(value, arg_path));
    }

    pub fn insert_param(&mut self, param_name: Parameter, value: Variant) {
        self.insert(Self::param_to_name(param_name), value);
    }

    fn param_to_name(param_name: Parameter) -> Name {
        let (bare_name, param_type) = param_name.into();
        match param_type {
            ParamType::Bare => panic!("Unresolved param {:?}", bare_name),
            ParamType::BuiltIn(q, _) => Name::qualified(bare_name, q),
            ParamType::UserDefined(_) => Name::bare(bare_name),
            ParamType::Array(boxed_param_type) => {
                let dummy_param = Parameter::new(bare_name, *boxed_param_type);
                Self::param_to_name(dummy_param)
            }
        }
    }

    pub fn insert(&mut self, name: Name, value: Variant) {
        self.map.insert(name, RuntimeVariableInfo::new(value, None));
    }

    pub fn insert_dim(&mut self, dim_name: DimVar, value: Variant) {
        let (bare_name, dim_type) = dim_name.into();
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
            allocate_built_in(q)
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

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Variant> {
        self.map.values_mut().map(|r| &mut r.value)
    }

    pub fn get_built_in(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&Variant> {
        // TODO make a structure that allows to lookup by BareName and QualifiedName without the need to clone
        let temp = Name::qualified(bare_name.clone(), qualifier);
        self.get_by_name(&temp)
    }

    pub fn get_user_defined(&self, bare_name: &BareName) -> Option<&Variant> {
        // TODO make a structure that allows to lookup by BareName and QualifiedName without the need to clone
        let temp = Name::bare(bare_name.clone());
        self.get_by_name(&temp)
    }

    pub fn get_by_name(&self, name: &Name) -> Option<&Variant> {
        self.map.get(name).map(|r| &r.value)
    }

    pub fn get(&self, index: usize) -> Option<&Variant> {
        self.map.get_by_index(index).map(|r| &r.value)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Variant> {
        self.map.get_by_index_mut(index).map(|r| &mut r.value)
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

    pub fn get_by_dim_name(&self, dim_name: &DimVar) -> Option<&Variant> {
        self.get_by_dim_name_internal(dim_name.bare_name(), dim_name.var_type())
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

    pub fn get_arg_path(&self, index: usize) -> Option<&Path> {
        self.map
            .get_by_index(index)
            .and_then(|r| r.arg_path.as_ref())
    }

    pub fn calculate_var_ptr(&self, name: &Name) -> usize {
        debug_assert!(self.map.get(name).is_some());
        self.map
            .keys()
            .take_while(|k| *k != name)
            .map(|k| self.get_by_name(k).unwrap())
            .map(Variant::byte_size)
            .sum()
    }

    pub fn array_names(&self) -> impl Iterator<Item = &Name> {
        self.map.keys().filter(move |key| match self.map.get(*key) {
            Some(RuntimeVariableInfo {
                value: Variant::VArray(_),
                ..
            }) => true,
            _ => false,
        })
    }
}

impl From<Arguments> for Variables {
    fn from(arguments: Arguments) -> Self {
        let mut variables: Self = Self::new();
        variables.apply_arguments(arguments);
        variables
    }
}

impl QByteSize for Variables {
    fn byte_size(&self) -> usize {
        self.iter().map(Variant::byte_size).sum()
    }
}
