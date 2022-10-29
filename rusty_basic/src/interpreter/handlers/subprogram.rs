use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::{Name, ParamName, QualifiedName};

pub fn begin_collect_arguments<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.context_mut().begin_collecting_arguments();
}

pub fn enqueue_to_return_stack<T: InterpreterTrait>(interpreter: &mut T, idx: &usize) {
    let v = interpreter.context()[*idx].clone();
    interpreter.by_ref_stack().push_back(v);
}

pub fn dequeue_from_return_stack<T: InterpreterTrait>(interpreter: &mut T) {
    let v = interpreter
        .by_ref_stack()
        .pop_front()
        .expect("by_ref_stack underflow");
    interpreter.registers_mut().set_a(v);
}

pub fn stash_function_return_value<T: InterpreterTrait>(
    interpreter: &mut T,
    function_name: &QualifiedName,
) {
    let name: Name = Name::from(function_name.clone());
    let v = interpreter
        .context_mut()
        .variables_mut()
        .get_or_create(name)
        .clone();
    interpreter.set_function_result(v);
}

pub fn un_stash_function_return_value<T: InterpreterTrait>(interpreter: &mut T) {
    let v = interpreter
        .take_function_result()
        .expect("Should have function result");
    interpreter.registers_mut().set_a(v);
}

pub fn push_unnamed_arg_by_val<T: InterpreterTrait>(interpreter: &mut T) {
    let v = interpreter.registers().get_a();
    interpreter
        .context_mut()
        .arguments_mut()
        .push_unnamed_by_val(v);
}

pub fn push_unnamed_arg_by_ref<T: InterpreterTrait>(interpreter: &mut T) {
    let path = interpreter
        .var_path_stack()
        .pop_back()
        .expect("Should have a VarPath");
    let v = interpreter.registers().get_a();
    interpreter
        .context_mut()
        .arguments_mut()
        .push_unnamed_by_ref(v, path);
}

pub fn push_a_to_named_arg<T: InterpreterTrait>(interpreter: &mut T, param_name: &ParamName) {
    let v = interpreter.registers().get_a();
    interpreter
        .context_mut()
        .arguments_mut()
        .push_named(param_name.clone(), v);
}
