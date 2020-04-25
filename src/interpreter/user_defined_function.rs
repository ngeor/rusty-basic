use crate::common::HasLocation;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::statement::StatementRunner;
use crate::interpreter::{InterpreterError, LookupFunctionImplementation, Result, Variant};
use crate::parser::{BlockNode, ExpressionNode, NameNode, QualifiedName};

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
    TI: ContextOwner + StatementRunner<BlockNode> + LookupFunctionImplementation,
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
        let result = interpreter
            .context_ref()
            .get_function_result(function_name.location());
        interpreter.pop();
        Ok(result)
    }
}
