use super::*;
use crate::common::*;
use crate::parser::{
    BareName, Expression, ExpressionNode, FunctionImplementation, HasExpressionType, Name,
    ParamName, ProgramNode, QualifiedName, SubImplementation, TopLevelToken,
};
use crate::variant::Variant;
use std::collections::HashMap;

/// Generates instructions for the given program.
pub fn generate_instructions(program: ProgramNode) -> InstructionGenerator {
    // pass 1: collect function/sub names -> parameter names, in order to use them in function/sub calls
    // the parameter names and types are needed
    let mut parameter_collector = ParameterCollector::default();
    parameter_collector.visit(&program);
    let sub_program_parameters = parameter_collector.to_sub_program_parameters();
    // pass 2 generate with labels still unresolved
    let mut generator = InstructionGenerator::new(sub_program_parameters);
    generator.generate_unresolved(program);
    // pass 3 resolve labels to addresses
    generator.resolve_labels();
    generator
}

// TODO ^ split 3 passes to distinct modules, do not return the whole InstructionGenerator but only the relevant data,
// statement_addresses and instructions

type ParamMap = HashMap<CaseInsensitiveString, Vec<ParamName>>;

#[derive(Default)]
struct ParameterCollector {
    functions: ParamMap,
    subs: ParamMap,
}

impl ParameterCollector {
    pub fn visit(&mut self, program: &ProgramNode) {
        for Locatable { element, .. } in program {
            self.visit_top_level_token(element);
        }
    }

    pub fn to_sub_program_parameters(self) -> SubProgramParameters {
        SubProgramParameters::new(self.functions, self.subs)
    }

    fn visit_top_level_token(&mut self, top_level_token: &TopLevelToken) {
        match top_level_token {
            TopLevelToken::FunctionImplementation(f) => {
                self.visit_function_implementation(f);
            }
            TopLevelToken::SubImplementation(s) => {
                self.visit_sub_implementation(s);
            }
            _ => {}
        }
    }

    fn visit_function_implementation(&mut self, f: &FunctionImplementation) {
        self.functions.insert(
            f.name.bare_name().clone(),
            f.params.clone().strip_location(),
        );
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) {
        self.subs
            .insert(s.name.element.clone(), s.params.clone().strip_location());
    }
}

pub struct SubProgramParameters {
    functions: ParamMap,
    subs: ParamMap,
}

impl SubProgramParameters {
    pub fn new(functions: ParamMap, subs: ParamMap) -> Self {
        Self { functions, subs }
    }

    pub fn get_function_parameters(&self, name: &BareName) -> &Vec<ParamName> {
        self.functions.get(name).unwrap()
    }

    pub fn get_sub_parameters(&self, name: &BareName) -> &Vec<ParamName> {
        self.subs.get(name).unwrap()
    }
}

pub struct InstructionGenerator {
    pub instructions: Vec<InstructionNode>,
    pub statement_addresses: Vec<usize>,
    pub sub_program_parameters: SubProgramParameters,
}

fn build_label_to_address_map(
    instructions: &Vec<InstructionNode>,
) -> HashMap<CaseInsensitiveString, usize> {
    let mut result: HashMap<CaseInsensitiveString, usize> = HashMap::new();
    for j in 0..instructions.len() {
        if let Instruction::Label(y) = instructions[j].as_ref() {
            result.insert(y.clone(), j);
        }
    }
    result
}

impl InstructionGenerator {
    pub fn new(sub_program_parameters: SubProgramParameters) -> Self {
        Self {
            instructions: vec![],
            statement_addresses: vec![],
            sub_program_parameters,
        }
    }

    pub fn generate_unresolved(&mut self, program: ProgramNode) {
        let mut functions: Vec<(FunctionImplementation, Location)> = vec![];
        let mut subs: Vec<(SubImplementation, Location)> = vec![];

        for t in program {
            let Locatable {
                element: top_level_token,
                pos,
            } = t;
            match top_level_token {
                TopLevelToken::Statement(s) => {
                    self.generate_statement_node_instructions(s.at(pos));
                }
                TopLevelToken::FunctionImplementation(f) => functions.push((f, pos)),
                TopLevelToken::SubImplementation(s) => subs.push((s, pos)),
                _ => {}
            }
        }

        // add HALT instruction at end of program to separate from the functions and subs
        self.mark_statement_address();
        self.push(
            Instruction::Halt,
            Location::new(std::u32::MAX, std::u32::MAX),
        );

        // functions
        for (f, pos) in functions {
            let Locatable {
                element: function_name,
                ..
            } = f.name;
            if let Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) = function_name
            {
                let block = f.body;
                self.function_label(&bare_name, pos);
                // set default value
                self.push_load(qualifier, pos);
                self.generate_block_instructions(block);
                self.mark_statement_address();
                self.push(Instruction::PopRet, pos);
            } else {
                panic!("Unexpected bare function name {:?}", function_name);
            }
        }

        // subs
        for (s, pos) in subs {
            let name = s.name;
            let bare_name: &BareName = name.as_ref();
            let block = s.body;
            self.sub_label(bare_name, pos);
            self.generate_block_instructions(block);
            self.mark_statement_address();
            self.push(Instruction::PopRet, pos);
        }
    }

    pub fn resolve_labels(&mut self) {
        let label_to_address = build_label_to_address_map(&self.instructions);
        for instruction_node in self.instructions.iter_mut() {
            let Locatable {
                element: instruction,
                ..
            } = instruction_node;
            Self::resolve_label(instruction, &label_to_address);
        }
    }

    fn resolve_label(
        instruction: &mut Instruction,
        label_to_address: &HashMap<CaseInsensitiveString, usize>,
    ) {
        match instruction {
            Instruction::Jump(AddressOrLabel::Unresolved(x)) => {
                *instruction =
                    Instruction::Jump(AddressOrLabel::Resolved(*label_to_address.get(x).unwrap()));
            }
            Instruction::JumpIfFalse(AddressOrLabel::Unresolved(x)) => {
                *instruction = Instruction::JumpIfFalse(AddressOrLabel::Resolved(
                    *label_to_address
                        .get(x)
                        .expect(&format!("Label {} not found", x)),
                ));
            }
            Instruction::OnErrorGoTo(AddressOrLabel::Unresolved(x)) => {
                *instruction = Instruction::OnErrorGoTo(AddressOrLabel::Resolved(
                    *label_to_address.get(x).unwrap(),
                ));
            }
            Instruction::GoSub(AddressOrLabel::Unresolved(x)) => {
                *instruction =
                    Instruction::GoSub(AddressOrLabel::Resolved(*label_to_address.get(x).unwrap()));
            }
            Instruction::Return(Some(AddressOrLabel::Unresolved(label))) => {
                *instruction = Instruction::Return(Some(AddressOrLabel::Resolved(
                    *label_to_address.get(label).unwrap(),
                )));
            }
            Instruction::ResumeLabel(AddressOrLabel::Unresolved(label)) => {
                *instruction = Instruction::ResumeLabel(AddressOrLabel::Resolved(
                    *label_to_address.get(label).unwrap(),
                ));
            }
            _ => {}
        }
    }

    pub fn push(&mut self, i: Instruction, pos: Location) {
        self.instructions.push(i.at(pos));
    }

    /// Adds a Load instruction, converting the given value into a Variant
    /// and storing it in register A.
    pub fn push_load<T>(&mut self, value: T, pos: Location)
    where
        Variant: From<T>,
    {
        self.push(Instruction::LoadIntoA(value.into()), pos);
    }

    /// Adds a Load instruction, converting the given value into a Variant
    /// and storing it in register A, followed by a PushUnnamed instruction.
    pub fn push_load_unnamed_arg<T>(&mut self, value: T, pos: Location)
    where
        Variant: From<T>,
    {
        self.push_load(value, pos);
        self.push(Instruction::PushAToUnnamedArg, pos);
    }

    pub fn jump_if_false<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::JumpIfFalse(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix.as_ref(), pos),
            ))),
            pos,
        );
    }

    pub fn jump<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!("_{}_{:?}", prefix.as_ref(), pos),
            ))),
            pos,
        );
    }

    pub fn label<S: AsRef<str>>(&mut self, prefix: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                "_{}_{:?}",
                prefix.as_ref(),
                pos
            ))),
            pos,
        );
    }

    pub fn function_label<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                ":fun:{}",
                name.as_ref(),
            ))),
            pos,
        );
    }

    pub fn jump_to_function<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!(":fun:{}", name.as_ref(),),
            ))),
            pos,
        );
    }

    pub fn sub_label<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Label(CaseInsensitiveString::new(format!(
                ":sub:{}",
                name.as_ref(),
            ))),
            pos,
        );
    }

    pub fn jump_to_sub<S: AsRef<str>>(&mut self, name: S, pos: Location) {
        self.push(
            Instruction::Jump(AddressOrLabel::Unresolved(CaseInsensitiveString::new(
                format!(":sub:{}", name.as_ref(),),
            ))),
            pos,
        );
    }

    pub fn generate_assignment_instructions(
        &mut self,
        l: Expression,
        r: ExpressionNode,
        pos: Location,
    ) {
        let left_type = l.expression_type();
        self.generate_expression_instructions_casting(r, left_type);
        self.generate_store_instructions(l, pos);
    }

    pub fn generate_store_instructions(&mut self, l: Expression, pos: Location) {
        self.generate_path_instructions(l.at(pos));
        self.push(Instruction::CopyAToVarPath, pos);
    }

    pub fn generate_load_instructions(&mut self, l: Expression, pos: Location) {
        self.generate_path_instructions(l.at(pos));
        self.push(Instruction::CopyVarPathToA, pos);
    }
}
