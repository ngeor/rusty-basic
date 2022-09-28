use crate::common::QError;
use crate::instruction_generator::{Path, RootPath};
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BareName;
use crate::variant::{QBNumberCast, Variant};

pub fn var_path_name<T: InterpreterTrait>(interpreter: &mut T, root_path: RootPath) {
    interpreter
        .var_path_stack()
        .push_back(Path::Root(root_path));
}

pub fn var_path_index<T: InterpreterTrait>(interpreter: &mut T) {
    let index_value = interpreter.registers().get_a();
    let old_name_ptr = interpreter
        .var_path_stack()
        .pop_back()
        .expect("Should have name_ptr");
    interpreter
        .var_path_stack()
        .push_back(old_name_ptr.append_array_element(index_value));
}

pub fn var_path_property<T: InterpreterTrait>(interpreter: &mut T, property_name: &BareName) {
    let old_name_ptr = interpreter
        .var_path_stack()
        .pop_back()
        .expect("Should have name_ptr");
    interpreter.var_path_stack().push_back(Path::Property(
        Box::new(old_name_ptr),
        property_name.clone(),
    ));
}

pub fn copy_a_to_var_path<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    let a = interpreter.registers().get_a();
    let v = resolve_name_ptr_mut(interpreter)?;
    *v = a;
    Ok(())
}

pub fn copy_var_path_to_a<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    let v = resolve_name_ptr_mut(interpreter)?;
    let v_copy = v.clone();
    interpreter.registers_mut().set_a(v_copy);
    Ok(())
}

pub fn pop_var_path<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    interpreter
        .var_path_stack()
        .pop_back()
        .ok_or(QError::Overflow)
        .map(|_| ())
}

fn resolve_name_ptr_mut<T: InterpreterTrait>(interpreter: &mut T) -> Result<&mut Variant, QError> {
    match interpreter.var_path_stack().pop_back() {
        Some(n) => {
            interpreter.var_path_stack().push_back(n.clone());
            resolve_some_name_ptr_mut(interpreter, n)
        }
        _ => panic!("Root name_ptr was None"),
    }
}

fn resolve_some_name_ptr_mut<T: InterpreterTrait>(
    interpreter: &mut T,
    name_ptr: Path,
) -> Result<&mut Variant, QError> {
    match name_ptr {
        Path::Root(RootPath { name, shared }) => {
            let variables = if shared {
                interpreter.context_mut().global_variables_mut()
            } else {
                interpreter.context_mut().variables_mut()
            };
            Ok(variables.get_or_create(name))
        }
        Path::ArrayElement(parent_name_ptr, indices) => {
            let parent_variant = resolve_some_name_ptr_mut(interpreter, *parent_name_ptr)?;
            resolve_array_mut(parent_variant, indices)
        }
        Path::Property(parent_name_ptr, property_name) => {
            let parent_variant = resolve_some_name_ptr_mut(interpreter, *parent_name_ptr)?;
            Ok(resolve_property_mut(parent_variant, &property_name))
        }
    }
}

fn resolve_array_mut(v: &mut Variant, indices: Vec<Variant>) -> Result<&mut Variant, QError> {
    match v {
        Variant::VArray(v_array) => {
            let int_indices: Vec<i32> = indices.try_cast()?;
            v_array.get_element_mut(&int_indices)
        }
        _ => panic!("Expected array, found {:?}", v),
    }
}

fn resolve_property_mut<'a>(v: &'a mut Variant, property_name: &BareName) -> &'a mut Variant {
    match v {
        Variant::VUserDefined(boxed_user_defined_value) => boxed_user_defined_value
            .get_mut(property_name)
            .expect("Property not defined, linter should have caught this"),
        _ => panic!("Expected user defined type, found {:?}", v),
    }
}
