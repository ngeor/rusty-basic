use crate::common::Result;
use crate::parser::{BareName, Block, Program, QName, QualifiedName, TopLevelToken, TypeQualifier};
use std::collections::HashMap;

mod assignment;
mod casting;
mod expression;
mod for_loop;
mod function_call;
mod function_context;
mod if_block;
mod statement;
mod stdlib;
mod sub_call;
mod variant;

#[cfg(test)]
mod test_utils;

use self::function_context::FunctionContext;
pub use self::stdlib::*;
use self::variant::*;

#[derive(Debug)]
pub enum Context {
    Root(HashMap<QualifiedName, Variant>),
    Nested(HashMap<QualifiedName, Variant>, QualifiedName, Box<Context>),
}

impl Context {
    pub fn new() -> Context {
        Context::Root(HashMap::new())
    }

    pub fn clone_variable_map(&self) -> HashMap<QualifiedName, Variant> {
        match self {
            Context::Root(m) | Context::Nested(m, _, _) => m.clone(),
        }
    }

    pub fn get(&self, name: &QualifiedName) -> Option<&Variant> {
        match self {
            Context::Root(m) | Context::Nested(m, _, _) => m.get(name),
        }
    }

    pub fn insert(&mut self, name: QualifiedName, value: Variant) -> Option<Variant> {
        match self {
            Context::Root(m) | Context::Nested(m, _, _) => m.insert(name, value),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter<S> {
    stdlib: S,
    context: Option<Context>, // declared as option to be able to use Option.take; it should always have Some value
    function_context: FunctionContext,
}

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib) -> Interpreter<TStdlib> {
        Interpreter {
            stdlib,
            context: Some(Context::new()),
            function_context: FunctionContext::new(),
        }
    }

    pub fn err<TResult, S: AsRef<str>>(&self, msg: S) -> Result<TResult> {
        Err(msg.as_ref().to_string())
    }

    pub fn interpret(&mut self, program: Program) -> Result<()> {
        let mut statements: Block = vec![];
        for top_level_token in program {
            match top_level_token {
                TopLevelToken::FunctionDeclaration(function_name, args) => {
                    self.function_context.add_function_declaration(
                        self.to_typed(function_name),
                        args.iter().map(|x| self.to_typed(x)).collect(),
                    )?;
                }
                TopLevelToken::FunctionImplementation(function_name, args, block) => {
                    self.function_context.add_function_implementation(
                        self.to_typed(function_name),
                        args.iter().map(|x| self.to_typed(x)).collect(),
                        block,
                    )?;
                }
                TopLevelToken::Statement(s) => {
                    statements.push(s);
                }
                TopLevelToken::EOF => (),
            }
        }
        self._search_for_unimplemented_declarations()?;
        self.statements(&statements)
    }

    pub fn push_context(&mut self, result_name: QualifiedName) {
        match self.context.take() {
            Some(old_context) => {
                self.context = Some(Context::Nested(
                    old_context.clone_variable_map(),
                    result_name,
                    Box::new(old_context),
                ));
            }
            None => panic!("Stack underflow"),
        }
    }

    pub fn pop_context(&mut self) {
        match self.context.take() {
            Some(old_context) => match old_context {
                Context::Root(_) => panic!("Stack underflow"),
                Context::Nested(_, _, b) => self.context = Some(*b),
            },
            None => panic!("Stack underflow"),
        }
    }

    fn _search_for_unimplemented_declarations(&mut self) -> Result<()> {
        for name in self.function_context.get_function_declarations() {
            if let None = self.function_context.get_function_implementation(name) {
                return self.err("Subprogram not defined");
            }
        }

        Ok(())
    }

    pub fn matches_result_name(&self, bare_name: &BareName) -> Option<QualifiedName> {
        match &self.context {
            Some(c) => match c {
                Context::Nested(_, result_name, _) => {
                    if &result_name.name == bare_name {
                        Some(result_name.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

//
// TypeQualifierResolver
//

trait TypeQualifierResolver<T> {
    fn effective_type_qualifier(&self, name: T) -> TypeQualifier;
}

impl<S: Stdlib> TypeQualifierResolver<&BareName> for Interpreter<S> {
    fn effective_type_qualifier(&self, _name: &BareName) -> TypeQualifier {
        TypeQualifier::BangSingle
    }
}

impl<S: Stdlib> TypeQualifierResolver<BareName> for Interpreter<S> {
    fn effective_type_qualifier(&self, name: BareName) -> TypeQualifier {
        self.effective_type_qualifier(&name)
    }
}

impl<S: Stdlib> TypeQualifierResolver<&QualifiedName> for Interpreter<S> {
    fn effective_type_qualifier(&self, name: &QualifiedName) -> TypeQualifier {
        name.qualifier.clone()
    }
}

impl<S: Stdlib> TypeQualifierResolver<QualifiedName> for Interpreter<S> {
    fn effective_type_qualifier(&self, name: QualifiedName) -> TypeQualifier {
        name.qualifier
    }
}

impl<S: Stdlib> TypeQualifierResolver<&QName> for Interpreter<S> {
    fn effective_type_qualifier(&self, name: &QName) -> TypeQualifier {
        match name {
            QName::Untyped(bare_name) => self.effective_type_qualifier(bare_name),
            QName::Typed(qualified_name) => self.effective_type_qualifier(qualified_name),
        }
    }
}

impl<S: Stdlib> TypeQualifierResolver<QName> for Interpreter<S> {
    fn effective_type_qualifier(&self, name: QName) -> TypeQualifier {
        match name {
            QName::Untyped(bare_name) => self.effective_type_qualifier(bare_name),
            QName::Typed(qualified_name) => self.effective_type_qualifier(qualified_name),
        }
    }
}

//
// QualifiedNameResolver
//

trait QualifiedNameResolver<T> {
    fn to_typed(&self, name: T) -> QualifiedName;
}

impl<S: Stdlib> QualifiedNameResolver<&BareName> for Interpreter<S> {
    fn to_typed(&self, bare_name: &BareName) -> QualifiedName {
        let q = self.effective_type_qualifier(bare_name);
        QualifiedName::new(bare_name.clone(), q)
    }
}

impl<S: Stdlib> QualifiedNameResolver<BareName> for Interpreter<S> {
    fn to_typed(&self, bare_name: BareName) -> QualifiedName {
        let q = self.effective_type_qualifier(&bare_name);
        QualifiedName::new(bare_name, q)
    }
}

impl<S: Stdlib> QualifiedNameResolver<&QualifiedName> for Interpreter<S> {
    fn to_typed(&self, name: &QualifiedName) -> QualifiedName {
        name.clone()
    }
}

impl<S: Stdlib> QualifiedNameResolver<QualifiedName> for Interpreter<S> {
    fn to_typed(&self, name: QualifiedName) -> QualifiedName {
        name
    }
}

impl<S: Stdlib> QualifiedNameResolver<&QName> for Interpreter<S> {
    fn to_typed(&self, name: &QName) -> QualifiedName {
        match name {
            QName::Untyped(bare_name) => self.to_typed(bare_name),
            QName::Typed(qualified_name) => self.to_typed(qualified_name),
        }
    }
}

impl<S: Stdlib> QualifiedNameResolver<QName> for Interpreter<S> {
    fn to_typed(&self, name: QName) -> QualifiedName {
        match name {
            QName::Untyped(bare_name) => self.to_typed(bare_name),
            QName::Typed(qualified_name) => self.to_typed(qualified_name),
        }
    }
}

//
// VariableGetter
//

trait VariableGetter<T> {
    fn get_variable(&self, variable_name: T) -> Result<&Variant>;
}

impl<S: Stdlib> VariableGetter<&QualifiedName> for Interpreter<S> {
    fn get_variable(&self, variable_name: &QualifiedName) -> Result<&Variant> {
        match &self.context {
            Some(c) => match c.get(variable_name) {
                Some(v) => Ok(v),
                None => self.err(format!("Variable {} not defined", variable_name)),
            },
            None => self.err("Stack underflow"),
        }
    }
}

impl<S: Stdlib> VariableGetter<&QName> for Interpreter<S> {
    fn get_variable(&self, variable_name: &QName) -> Result<&Variant> {
        match variable_name {
            QName::Untyped(bare_name) => {
                let typed = self.to_typed(bare_name);
                self.get_variable(&typed)
            }
            QName::Typed(q) => self.get_variable(q),
        }
    }
}

//
// VariableSetter
//

trait VariableSetter<T> {
    fn set_variable(&mut self, variable_name: T, variable_value: Variant);
}

impl<S: Stdlib> VariableSetter<QualifiedName> for Interpreter<S> {
    fn set_variable(&mut self, variable_name: QualifiedName, variable_value: Variant) {
        match &mut self.context {
            Some(c) => {
                c.insert(variable_name, variable_value);
            }
            None => panic!("stack underflow"),
        }
    }
}

impl<S: Stdlib> VariableSetter<&QualifiedName> for Interpreter<S> {
    fn set_variable(&mut self, variable_name: &QualifiedName, variable_value: Variant) {
        self.set_variable(variable_name.clone(), variable_value);
    }
}

impl<S: Stdlib> VariableSetter<QName> for Interpreter<S> {
    fn set_variable(&mut self, variable_name: QName, variable_value: Variant) {
        match variable_name {
            QName::Untyped(bare_name) => {
                let typed = self.to_typed(bare_name);
                self.set_variable(typed, variable_value)
            }
            QName::Typed(q) => self.set_variable(q, variable_value),
        }
    }
}

impl<S: Stdlib> VariableSetter<&QName> for Interpreter<S> {
    fn set_variable(&mut self, variable_name: &QName, variable_value: Variant) {
        match variable_name {
            QName::Untyped(bare_name) => {
                let typed = self.to_typed(bare_name);
                self.set_variable(typed, variable_value)
            }
            QName::Typed(q) => self.set_variable(q, variable_value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;

    #[test]
    fn test_interpret_print_hello_world() {
        let input = "PRINT \"Hello, world!\"";
        let stdlib = MockStdlib::new();
        interpret(input, stdlib).unwrap();
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
        stdlib.next_input = "10".to_string();
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
        stdlib.next_input = "11".to_string();
        interpret_file("FIB_FQ.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_input() {
        let stdlib = MockStdlib::new();
        interpret_file("INPUT.BAS", stdlib).unwrap();
    }
}
