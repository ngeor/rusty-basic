use crate::interpreter::arguments::Arguments;
use crate::parser::{
    BareName, DimName, DimType, ExpressionType, HasExpressionType, Name, ParamName, ParamType,
    QualifiedName, TypeQualifier,
};
use crate::variant::{Variant, V_FALSE};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Variables {
    name_to_index: HashMap<Name, usize>,
    values: Vec<Variant>,
}

impl Variables {
    pub fn new() -> Self {
        Self {
            name_to_index: HashMap::new(),
            values: Vec::new(),
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

    pub fn insert_unnamed(&mut self, value: Variant) {
        self.values.push(value);
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
        match self.name_to_index.get(&name) {
            Some(idx) => {
                self.values[*idx] = value;
            }
            None => {
                self.name_to_index.insert(name, self.values.len());
                self.values.push(value);
            }
        }
    }

    pub fn insert_dim(&mut self, dim_name: DimName, value: Variant) {
        let DimName {
            bare_name,
            dim_type,
            ..
        } = dim_name;
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
                let element_type = box_element_type.expression_type();
                match element_type {
                    ExpressionType::BuiltIn(q) => {
                        self.insert_built_in(bare_name, q, value);
                    }
                    ExpressionType::FixedLengthString(_) => {
                        self.insert_built_in(bare_name, TypeQualifier::DollarString, value);
                    }
                    _ => self.insert_user_defined(bare_name, value),
                }
            }
            DimType::Bare => panic!("Unresolved type"),
        }
    }

    pub fn get_or_create(&mut self, name: Name) -> &mut Variant {
        match self.name_to_index.get(&name) {
            Some(idx) => self.values.get_mut(*idx).expect("Should have variable"),
            _ => {
                let value = Self::default_value_for_name(&name);
                self.name_to_index.insert(name, self.values.len());
                self.values.push(value);
                self.values.last_mut().expect("Should have variable")
            }
        }
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

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get_built_in(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&Variant> {
        self.get_by_name(&QualifiedName::new(bare_name.clone(), qualifier).into())
    }

    pub fn get_user_defined(&self, bare_name: &BareName) -> Option<&Variant> {
        self.get_by_name(&bare_name.clone().into())
    }

    pub fn get_by_name(&self, name: &Name) -> Option<&Variant> {
        self.name_to_index.get(name).and_then(|idx| self.get(*idx))
    }

    pub fn get(&self, idx: usize) -> Option<&Variant> {
        self.values.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Variant> {
        self.values.get_mut(idx)
    }

    pub fn apply_arguments(&mut self, arguments: Arguments) {
        for (opt_param, arg) in arguments.into_iter() {
            match opt_param {
                Some(param_name) => self.insert_param(param_name, arg),
                None => self.insert_unnamed(arg),
            }
        }
    }

    pub fn get_by_dim_name(&self, dim_name: &DimName) -> Option<&Variant> {
        match dim_name.dim_type() {
            DimType::BuiltIn(q, _) => self.get_built_in(dim_name.bare_name(), *q),
            _ => {
                // TODO fix this
                None
            }
        }
    }
}

impl From<Arguments> for Variables {
    fn from(arguments: Arguments) -> Self {
        let mut variables: Self = Self::new();
        variables.apply_arguments(arguments);
        variables
    }
}
