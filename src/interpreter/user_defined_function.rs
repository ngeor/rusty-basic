use crate::common::{HasLocation, Location};
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::statement::StatementRunner;
use crate::interpreter::variable_getter::VariableGetter;
use crate::interpreter::variable_setter::VariableSetter;
use crate::interpreter::{InterpreterError, LookupFunctionImplementation, Result, Variant};
use crate::parser::{BlockNode, ExpressionNode, HasQualifier, NameNode, QualifiedName};

fn _get_variable_name_or_default<VG: VariableGetter>(
    variable_getter: &VG,
    function_name: &QualifiedName,
    pos: Location,
) -> Variant {
    match variable_getter.get_variable_at(function_name, pos) {
        Ok(v) => v.clone(),
        Err(_) => Variant::default_variant(function_name.qualifier()),
    }
}

pub fn supports_function<LI: LookupFunctionImplementation>(
    subprogram_context: &LI,
    function_name: &NameNode,
) -> bool {
    subprogram_context.has_function(function_name)
}

pub fn call_function<TI>(
    interpreter: &mut TI,
    function_name: &NameNode,
    args: &Vec<ExpressionNode>,
    arg_values: Vec<Variant>,
) -> Result<Variant>
where
    TI: VariableGetter
        + VariableSetter<NameNode>
        + ContextOwner
        + StatementRunner<BlockNode>
        + LookupFunctionImplementation,
{
    let function_implementation = interpreter.lookup_function_implementation(function_name)?;
    let function_parameters: Vec<QualifiedName> = function_implementation.parameters;
    if function_parameters.len() != args.len() {
        Err(InterpreterError::new_with_pos(
            format!(
                "Function {} expected {} parameters but {} were given",
                function_implementation.name,
                function_parameters.len(),
                args.len()
            ),
            function_name.location(),
        ))
    } else {
        interpreter.push_function(function_implementation.name.clone());
        interpreter.populate(function_parameters, arg_values, function_name.location())?;
        interpreter
            .run(&function_implementation.block)
            .map_err(|e| e.merge_pos(function_name.location()))?;
        let result = _get_variable_name_or_default(
            interpreter,
            &function_implementation.name,
            function_name.location(),
        );
        interpreter.pop();
        Ok(result)
    }
}
