use crate::common::Result;
use crate::parser::{Block, Parser, QName, TopLevelToken, TypeQualifier};
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
mod assignment;
mod casting;
mod context;
mod expression;
mod for_loop;
mod function_context;
mod statement;
mod stdlib;
mod sub_call;
mod variant;

#[cfg(test)]
mod test_utils;

use self::context::*;
use self::function_context::FunctionContext;
use self::stdlib::{DefaultStdlib, Stdlib};
use self::variant::*;

pub struct Interpreter<T, S> {
    parser: Parser<T>,
    stdlib: S,
    context_stack: Vec<Context>,
    function_context: FunctionContext,
}

impl<T: BufRead, TStdlib: Stdlib> Interpreter<T, TStdlib> {
    pub fn new(parser: Parser<T>, stdlib: TStdlib) -> Interpreter<T, TStdlib> {
        Interpreter {
            parser,
            stdlib,
            context_stack: vec![Context::new()],
            function_context: FunctionContext::new(),
        }
    }

    pub fn err<TResult, S: AsRef<str>>(&self, msg: S) -> Result<TResult> {
        self.parser.buf_lexer.err(msg)
    }

    pub fn interpret(&mut self) -> Result<()> {
        let program = self.parser.parse()?;
        let mut statements: Block = vec![];
        for top_level_token in program {
            match top_level_token {
                // TODO: search for duplicate declarations / conflicting types
                TopLevelToken::FunctionDeclaration(f, args) => {
                    self.function_context.add_function_declaration(f, args)?;
                }
                TopLevelToken::FunctionImplementation(f, args, block) => {
                    self.function_context
                        .add_function_implementation(f, args, block)?;
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

    pub fn push_context(&mut self) -> Result<()> {
        if self.context_stack.len() >= 1 {
            self.context_stack
                .push(self.context_stack[self.context_stack.len() - 1].clone());
            Ok(())
        } else {
            self.err("Stack underflow")
        }
    }

    pub fn pop_context(&mut self) -> Result<()> {
        if self.context_stack.len() > 1 {
            self.context_stack.remove(self.context_stack.len() - 1);
            Ok(())
        } else {
            self.err("Stack underflow")
        }
    }

    fn _search_for_unimplemented_declarations(&mut self) -> Result<()> {
        for name in self.function_context.get_function_declarations() {
            if let None = self.function_context.get_function_implementation(name) {
                return self.err(format!("Function {} is not implemented", name));
            }
        }

        Ok(())
    }

    pub fn effective_type_qualifier(&self, variable_name: &QName) -> TypeQualifier {
        match variable_name {
            QName::Untyped(_) => TypeQualifier::BangSingle,
            QName::Typed(_, type_qualifier) => type_qualifier.clone(),
        }
    }

    pub fn ensure_typed(&self, variable_name: &QName) -> QName {
        match variable_name {
            QName::Untyped(name) => {
                QName::Typed(name.clone(), self.effective_type_qualifier(variable_name))
            }
            _ => variable_name.clone(),
        }
    }
}

impl<T: BufRead, TStdlib: Stdlib> ReadOnlyContext for Interpreter<T, TStdlib> {
    fn get_variable(&self, variable_name: &QName) -> Result<Variant> {
        let typed = self.ensure_typed(variable_name);
        self.context_stack[self.context_stack.len() - 1].get_variable(&typed)
    }
}

impl<T: BufRead, TStdlib: Stdlib> ReadWriteContext for Interpreter<T, TStdlib> {
    fn set_variable(&mut self, variable_name: &QName, variable_value: Variant) -> Result<()> {
        let idx = self.context_stack.len() - 1;
        let typed = self.ensure_typed(variable_name);
        self.context_stack[idx].set_variable(&typed, variable_value)
    }
}

// bytes || &str -> Interpreter
impl<T> From<T> for Interpreter<BufReader<Cursor<T>>, DefaultStdlib>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        Interpreter::new(Parser::from(input), DefaultStdlib {})
    }
}

// File -> Interpreter
impl From<File> for Interpreter<BufReader<File>, DefaultStdlib> {
    fn from(input: File) -> Self {
        Interpreter::new(Parser::from(input), DefaultStdlib {})
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
    fn test_interpreter_fixture_fib() {
        let mut stdlib = MockStdlib::new();
        stdlib.next_input = "10".to_string();
        interpret_file("FIB.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_fib_fq() {
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
