use crate::instruction_generator::{AddressOrLabel, Instruction, InstructionGenerator};
use crate::linter::SubprogramName;
use rusty_common::{AtLocation, Locatable, Location};
use rusty_parser::*;

impl InstructionGenerator {
    pub fn generate_built_in_function_call_instructions(
        &mut self,
        function_name: BuiltInFunction,
        args: ExpressionNodes,
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
        args: ExpressionNodes,
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
        args: ExpressionNodes,
    ) {
        let Locatable { element: name, pos } = function_name;
        let qualified_name = name.demand_qualified();
        let subprogram_name = SubprogramName::Function(qualified_name.clone());
        // cloning to fight the borrow checker
        let function_parameters: Vec<ParamName> = self
            .subprogram_info_repository
            .get_subprogram_info(&subprogram_name)
            .params
            .clone();
        self.generate_push_named_args_instructions(&function_parameters, &args, pos);
        self.push_stack(subprogram_name.clone(), pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos);
        self.jump_to_subprogram(&subprogram_name, pos);
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
        args: ExpressionNodes,
    ) {
        let Locatable { element: name, pos } = name_node;
        let subprogram_name = SubprogramName::Sub(name);
        // cloning to fight the borrow checker
        let sub_impl_parameters: Vec<ParamName> = self
            .subprogram_info_repository
            .get_subprogram_info(&subprogram_name)
            .params
            .clone();
        self.generate_push_named_args_instructions(&sub_impl_parameters, &args, pos);
        self.push_stack(subprogram_name.clone(), pos);
        let idx = self.instructions.len();
        self.push(Instruction::PushRet(idx + 2), pos); // points to "generate_stash_by_ref_args"
        self.jump_to_subprogram(&subprogram_name, pos);
        self.generate_stash_by_ref_args(&args);
        self.push(Instruction::PopStack, pos);
        self.generate_un_stash_by_ref_args(&args);
    }

    fn generate_push_named_args_instructions(
        &mut self,
        param_names: &[ParamName],
        args: &ExpressionNodes,
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

    fn generate_push_unnamed_args_instructions(&mut self, args: &ExpressionNodes, pos: Location) {
        self.push(Instruction::BeginCollectArguments, pos);
        for Locatable { element: arg, pos } in args {
            if arg.is_by_ref() {
                self.generate_expression_instructions_optionally_by_ref(arg.clone().at(pos), false);
                self.push(Instruction::PushUnnamedByRef, *pos);
            } else {
                self.generate_expression_instructions(arg.clone().at(pos));
                self.push(Instruction::PushUnnamedByVal, *pos);
            }
        }
    }

    fn generate_stash_by_ref_args(&mut self, args: &ExpressionNodes) {
        for (idx, Locatable { element: arg, pos }) in args.iter().enumerate() {
            if arg.is_by_ref() {
                self.push(Instruction::EnqueueToReturnStack(idx), *pos);
            }
        }
    }

    fn generate_un_stash_by_ref_args(&mut self, args: &ExpressionNodes) {
        for Locatable { element: arg, pos } in args {
            if arg.is_by_ref() {
                self.push(Instruction::DequeueFromReturnStack, *pos);
                self.generate_fix_string_length(arg, *pos);
                self.generate_store_instructions(arg.clone(), *pos);
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
            .subprogram_info_repository
            .get_subprogram_info(&subprogram_name)
            .is_static
        {
            self.push(Instruction::PushStaticStack(subprogram_name), pos);
        } else {
            self.push(Instruction::PushStack, pos);
        }
    }

    fn jump_to_subprogram(&mut self, subprogram_name: &SubprogramName, pos: Location) {
        let label: BareName = Self::format_subprogram_label(subprogram_name);
        self.push(Instruction::Jump(AddressOrLabel::Unresolved(label)), pos);
    }
}
