mod assignment;
mod built_in_functions;
mod built_in_subs;
mod casting;
mod context;
mod context_owner;
mod expression;
mod for_loop;
mod function_call;
mod function_context;
mod if_block;
mod interpreter_error;
mod statement;
mod stdlib;
mod sub_call;
mod sub_context;
mod subprogram_context;
mod type_resolver_impl;
mod user_defined_function;
mod user_defined_sub;
mod variable_getter;
mod variable_setter;
mod variant;
mod while_wend;

#[cfg(test)]
mod test_utils;

pub use self::interpreter_error::*;
pub use self::stdlib::*;
pub use self::variant::*;

use crate::common::{CaseInsensitiveString, HasLocation};
use crate::interpreter::context::Context;
use crate::interpreter::function_context::{FunctionContext, QualifiedFunctionImplementationNode};
use crate::interpreter::statement::StatementRunner;
use crate::interpreter::sub_context::{QualifiedSubImplementationNode, SubContext};
use crate::interpreter::type_resolver_impl::TypeResolverImpl;
use crate::parser::*;

use std::convert::TryInto;

#[derive(Debug)]
pub struct Interpreter<S> {
    stdlib: S,
    context: Option<Context>, // declared as option to be able to use Option.take; it should always have Some value
    function_context: FunctionContext,
    sub_context: SubContext,
    type_resolver: TypeResolverImpl,
}

pub type Result<T> = std::result::Result<T, InterpreterError>;

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib) -> Interpreter<TStdlib> {
        Interpreter {
            stdlib,
            context: Some(Context::new()),
            function_context: FunctionContext::new(),
            sub_context: SubContext::new(),
            type_resolver: TypeResolverImpl::new(),
        }
    }

    pub fn interpret(&mut self, program: ProgramNode) -> Result<()> {
        let mut statements: BlockNode = vec![];
        for top_level_token in program {
            match top_level_token {
                TopLevelTokenNode::FunctionDeclaration(f_name, f_params, f_pos) => {
                    self.function_context.add_declaration(
                        f_name,
                        f_params,
                        f_pos,
                        &self.type_resolver,
                    )?;
                }
                TopLevelTokenNode::SubDeclaration(s_name, s_params, s_pos) => {
                    self.sub_context.add_declaration(
                        s_name,
                        s_params,
                        s_pos,
                        &self.type_resolver,
                    )?;
                }
                TopLevelTokenNode::FunctionImplementation(f_name, f_params, f_body, f_pos) => {
                    self.function_context.add_implementation(
                        f_name,
                        f_params,
                        f_body,
                        f_pos,
                        &self.type_resolver,
                    )?;
                }
                TopLevelTokenNode::SubImplementation(s_name, s_params, s_body, s_pos) => {
                    self.sub_context.add_implementation(
                        s_name,
                        s_params,
                        s_body,
                        s_pos,
                        &self.type_resolver,
                    )?;
                }
                TopLevelTokenNode::Statement(s) => {
                    statements.push(s);
                }
                TopLevelTokenNode::DefType(x) => self.handle_def_type(x),
            }
        }
        self._search_for_unimplemented_declarations()?;
        self.run(&statements)
    }

    fn _search_for_unimplemented_declarations(&mut self) -> Result<()> {
        self.function_context
            .ensure_all_declared_programs_are_implemented()?;
        self.sub_context
            .ensure_all_declared_programs_are_implemented()?;
        Ok(())
    }

    fn context_ref(&self) -> &Context {
        match self.context.as_ref() {
            Some(x) => x,
            None => panic!("Stack underflow"),
        }
    }

    fn context_mut(&mut self) -> &mut Context {
        match self.context.as_mut() {
            Some(x) => x,
            None => panic!("Stack underflow"),
        }
    }

    fn handle_def_type(&mut self, x: DefTypeNode) {
        let q: TypeQualifier = x.qualifier();
        for r in x.ranges() {
            match *r {
                LetterRangeNode::Single(c) => self.type_resolver.set(c, c, q),
                LetterRangeNode::Range(start, stop) => self.type_resolver.set(start, stop, q),
            }
        }
    }

    /// Evaluates the condition of the given conditional block.
    fn evaluate_condition(&mut self, conditional_block: &ConditionalBlockNode) -> Result<bool> {
        let condition_value = self.evaluate_expression(&conditional_block.condition)?;
        condition_value
            .try_into()
            .map_err(|e| InterpreterError::new_with_pos(e, conditional_block.pos))
    }
}

impl<TStdlib: Stdlib> TypeResolver for Interpreter<TStdlib> {
    fn resolve(&self, name: &CaseInsensitiveString) -> TypeQualifier {
        self.type_resolver.resolve(name)
    }
}

pub trait LookupFunctionImplementation {
    fn has_function(&self, function_name: &NameNode) -> bool;

    fn lookup_function_implementation(
        &self,
        function_name: &NameNode,
    ) -> Result<QualifiedFunctionImplementationNode>;
}

impl<S: Stdlib> LookupFunctionImplementation for Interpreter<S> {
    fn has_function(&self, function_name: &NameNode) -> bool {
        self.function_context
            .has_implementation(function_name.as_ref())
    }

    fn lookup_function_implementation(
        &self,
        function_name: &NameNode,
    ) -> Result<QualifiedFunctionImplementationNode> {
        let bare_name: &CaseInsensitiveString = function_name.as_ref();
        self.function_context
            .get_implementation(bare_name)
            .ok_or(InterpreterError::new_with_pos(
                "Unknown function",
                function_name.location(),
            ))
            .and_then(|implementation| {
                match function_name.as_ref() {
                    Name::Bare(_) => Ok(implementation),
                    Name::Typed(qualified_function_name) => {
                        if implementation.name.qualifier() != qualified_function_name.qualifier() {
                            // the function is defined as A#
                            // but is being called as A!
                            Err(InterpreterError::new_with_pos(
                                "Duplicate definition",
                                function_name.location(),
                            ))
                        } else {
                            Ok(implementation)
                        }
                    }
                }
            })
    }
}

pub trait LookupSubImplementation {
    fn has_sub(&self, sub_name: &BareNameNode) -> bool;

    fn get_sub(&self, sub_name: &BareNameNode) -> QualifiedSubImplementationNode;
}

impl<S: Stdlib> LookupSubImplementation for Interpreter<S> {
    fn has_sub(&self, sub_name: &BareNameNode) -> bool {
        self.sub_context.has_implementation(sub_name.as_ref())
    }

    fn get_sub(&self, sub_name: &BareNameNode) -> QualifiedSubImplementationNode {
        self.sub_context
            .get_implementation(sub_name.as_ref())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;

    #[test]
    fn test_interpret_print_hello_world() {
        let input = "PRINT \"Hello, world!\"";
        assert_eq!(interpret(input).stdlib.output, vec!["Hello, world!"]);
    }

    #[test]
    fn test_interpreter_fixture_hello1() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO1.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello2() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO2.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello_s() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO_S.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_for_print_10() {
        let stdlib = MockStdlib::new();
        interpret_file("FOR_PRINT_10.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_for_nested() {
        let stdlib = MockStdlib::new();
        interpret_file("FOR_NESTED.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_fib_bas() {
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("10");
        let interpreter = interpret_file("FIB.BAS", stdlib).unwrap();
        let output = interpreter.stdlib.output;
        assert_eq!(
            output,
            vec![
                "Enter the number of fibonacci to calculate",
                "Fibonacci of 0 is 0",
                "Fibonacci of 1 is 1",
                "Fibonacci of 2 is 1",
                "Fibonacci of 3 is 2",
                "Fibonacci of 4 is 3",
                "Fibonacci of 5 is 5",
                "Fibonacci of 6 is 8",
                "Fibonacci of 7 is 13",
                "Fibonacci of 8 is 21",
                "Fibonacci of 9 is 34",
                "Fibonacci of 10 is 55"
            ]
        );
    }

    #[test]
    fn test_interpreter_fixture_fib_fq_bas() {
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("11");
        interpret_file("FIB_FQ.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_input() {
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("");
        interpret_file("INPUT.BAS", stdlib).unwrap();
    }
}
