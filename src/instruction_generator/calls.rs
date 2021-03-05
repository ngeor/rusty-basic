use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{AtLocation, Locatable, Location};
use crate::instruction_generator::{Instruction, InstructionGenerator};
use crate::interpreter::context::SubprogramName;
use crate::parser::*;

impl InstructionGenerator {
    pub fn generate_built_in_function_call_instructions(
        &mut self,
        function_name: BuiltInFunction,
        args: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.generate_push_unnamed_args_instructions(&args, pos);
        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInFunction(function_name), pos);
        self.generate_stash_by_ref_args(&args);
        self.generate_stash_function_return_value(function_name.into(), pos);
        self.push(Instruction::PopStack, pos);
        self.generate_un_stash_by_ref_args(&args);
        self.generate_un_stash_function_return_value(pos);
    }

    pub fn generate_built_in_sub_call_instructions(
        &mut self,
        name: BuiltInSub,
        args: Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.generate_push_unnamed_args_instructions(&args, pos);
        self.push(Instruction::PushStack, pos);
        self.push(Instruction::BuiltInSub(name), pos);
        self.generate_stash_by_ref_args(&args);
        self.push(Instruction::PopStack, pos);
        self.generate_un_stash_by_ref_args(&args);
    }

    pub fn generate_function_call_instructions(
        &mut self,
        function_name: NameNode,
        args: Vec<ExpressionNode>,
    ) {
        let Locatable { element: name, pos } = function_name;
        let qualified_name = name.demand_qualified();
        let bare_name: &BareName = qualified_name.as_ref();
        // cloning to fight the borrow checker
        let function_parameters = self
            .subprogram_parameters
            .get_function_parameters(&qualified_name)
            .clone();
        self.generate_push_named_args_instructions(&function_parameters, &args, pos);
        self.push_stack(SubprogramName::Function(qualified_name.clone()), pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_function(bare_name, pos);
        // TODO find different way for by ref args
        // stash by-ref variables
        self.generate_stash_by_ref_args(&args);
        // stash function name
        self.generate_stash_function_return_value(qualified_name, pos);
        // switch to parent context
        self.push(Instruction::PopStack, pos);
        // un-stash by-ref variables
        self.generate_un_stash_by_ref_args(&args);
        // un-stash function name
        self.generate_un_stash_function_return_value(pos);
    }

    pub fn generate_sub_call_instructions(
        &mut self,
        name_node: BareNameNode,
        args: Vec<ExpressionNode>,
    ) {
        let Locatable { element: name, pos } = name_node;
        // cloning to fight the borrow checker
        let sub_impl_parameters = self.subprogram_parameters.get_sub_parameters(&name).clone();
        self.generate_push_named_args_instructions(&sub_impl_parameters, &args, pos);
        self.push_stack(SubprogramName::Sub(name.clone()), pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos); // points to "generate_stash_by_ref_args"
        self.jump_to_sub(name, pos);
        self.generate_stash_by_ref_args(&args);
        self.push(Instruction::PopStack, pos);
        self.generate_un_stash_by_ref_args(&args);
    }

    fn generate_push_named_args_instructions(
        &mut self,
        param_names: &Vec<ParamName>,
        args: &Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectArguments, pos);
        for (param_name, Locatable { element: arg, pos }) in param_names.iter().zip(args.iter()) {
            self.generate_expression_instructions_casting(
                arg.clone().at(pos),
                param_name.expression_type(),
            );
            self.push(Instruction::PushNamed(param_name.clone()), *pos);
        }
    }

    fn generate_push_unnamed_args_instructions(
        &mut self,
        args: &Vec<ExpressionNode>,
        pos: Location,
    ) {
        self.push(Instruction::BeginCollectArguments, pos);
        for Locatable { element: arg, pos } in args {
            self.generate_expression_instructions(arg.clone().at(pos));
            self.push(Instruction::PushAToUnnamedArg, *pos);
        }
    }

    fn generate_stash_by_ref_args(&mut self, args: &Vec<ExpressionNode>) {
        let mut idx: usize = 0;
        for Locatable { element: arg, pos } in args {
            match arg {
                Expression::Variable(_, _)
                | Expression::Property(_, _, _)
                | Expression::ArrayElement(_, _, _) => {
                    // by ref
                    self.push(Instruction::EnqueueToReturnStack(idx), *pos);
                }
                _ => {}
            }

            idx += 1;
        }
    }

    fn generate_un_stash_by_ref_args(&mut self, args: &Vec<ExpressionNode>) {
        for Locatable { element: arg, pos } in args {
            match arg {
                Expression::Variable(_, _)
                | Expression::Property(_, _, _)
                | Expression::ArrayElement(_, _, _) => {
                    self.push(Instruction::DequeueFromReturnStack, *pos);
                    self.generate_fix_string_length(arg, *pos);
                    self.generate_store_instructions(arg.clone(), *pos);
                }
                _ => {}
            }
        }
    }

    fn generate_stash_function_return_value(
        &mut self,
        qualified_name: QualifiedName,
        pos: Location,
    ) {
        self.push(Instruction::StashFunctionReturnValue(qualified_name), pos);
    }

    fn generate_un_stash_function_return_value(&mut self, pos: Location) {
        self.push(Instruction::UnStashFunctionReturnValue, pos);
    }

    fn generate_fix_string_length(&mut self, arg: &Expression, pos: Location) {
        if let ExpressionType::FixedLengthString(l) = arg.expression_type() {
            self.push(Instruction::FixLength(l), pos);
        }
    }

    fn push_stack(&mut self, subprogram_name: SubprogramName, pos: Location) {
        if self
            .subprogram_parameters
            .get_subprogram_info(&subprogram_name)
            .is_static
        {
            self.push(Instruction::PushStaticStack(subprogram_name), pos);
        } else {
            self.push(Instruction::PushStack, pos);
        }
    }
}
