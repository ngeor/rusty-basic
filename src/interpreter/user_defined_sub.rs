use crate::common::HasLocation;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::statement::StatementRunner;
use crate::interpreter::variable_getter::VariableGetter;
use crate::interpreter::variable_setter::VariableSetter;
use crate::interpreter::{InterpreterError, LookupSubImplementation, Result, Variant};
use crate::parser::{BareNameNode, BlockNode, ExpressionNode, NameNode, QualifiedName};

pub fn supports_sub<LI: LookupSubImplementation>(
    subprogram_context: &LI,
    sub_name: &BareNameNode,
) -> bool {
    subprogram_context.has_sub(sub_name)
}

pub fn call_sub<TI>(
    interpreter: &mut TI,
    sub_name: &BareNameNode,
    args: &Vec<ExpressionNode>,
    arg_values: Vec<Variant>,
) -> Result<()>
where
    TI: VariableGetter
        + VariableSetter<NameNode>
        + ContextOwner
        + StatementRunner<BlockNode>
        + LookupSubImplementation,
{
    let sub_implementation = interpreter.get_sub(sub_name);
    let sub_parameters: Vec<QualifiedName> = sub_implementation.parameters;
    if sub_parameters.len() != args.len() {
        Err(InterpreterError::new_with_pos(
            format!(
                "Sub {} expected {} parameters but {} were given",
                sub_implementation.name,
                sub_parameters.len(),
                args.len()
            ),
            sub_name.location(),
        ))
    } else {
        interpreter.push_sub();
        interpreter.populate(sub_parameters, arg_values, sub_name.location())?;
        interpreter
            .run(&sub_implementation.block)
            .map_err(|e| e.merge_pos(sub_name.location()))?;
        interpreter.pop();
        Ok(())
    }
}
