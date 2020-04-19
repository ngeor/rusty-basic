use crate::common::CaseInsensitiveString;
use crate::parser::*;

mod assignment;
mod casting;
mod context;
mod expression;
mod for_loop;
mod function_call;
mod function_context;
mod if_block;
mod interpreter_error;
mod statement;
mod stdlib;
mod sub_call;
mod variable_getter;
mod variable_setter;
mod variant;
mod while_wend;

#[cfg(test)]
mod test_utils;

pub use self::context::*;
pub use self::function_context::*;
pub use self::interpreter_error::*;
pub use self::stdlib::*;
pub use self::variant::*;
use crate::interpreter::function_context::LookupImplementation;
use crate::interpreter::statement::StatementRunner;
use std::convert::TryInto;

#[derive(Debug)]
pub struct Interpreter<S> {
    stdlib: S,
    context: Option<Context>, // declared as option to be able to use Option.take; it should always have Some value
    function_context: FunctionContext,
    resolver_helper: ResolverHelper,
}

pub type Result<T> = std::result::Result<T, InterpreterError>;

pub trait PushPopContext {
    fn push_context(&mut self, result_name: QualifiedName);

    fn pop_context(&mut self);
}

impl<S: Stdlib> LookupImplementation for Interpreter<S> {
    fn lookup_implementation(
        &self,
        function_name: &NameNode,
    ) -> Result<Option<QualifiedFunctionImplementationNode>> {
        self.function_context.lookup_implementation(function_name)
    }
}

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib) -> Interpreter<TStdlib> {
        Interpreter {
            stdlib,
            context: Some(Context::new()),
            function_context: FunctionContext::new(),
            resolver_helper: ResolverHelper::new(),
        }
    }

    pub fn interpret(&mut self, program: ProgramNode) -> Result<()> {
        let mut statements: BlockNode = vec![];
        for top_level_token in program {
            match top_level_token {
                TopLevelTokenNode::FunctionDeclaration(f) => {
                    self.function_context
                        .add_function_declaration(f, &self.resolver_helper)?;
                }
                TopLevelTokenNode::FunctionImplementation(f) => {
                    self.function_context
                        .add_function_implementation(f, &self.resolver_helper)?;
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
        for name in self.function_context.get_function_declarations() {
            if let None = self.function_context.get_function_implementation(name) {
                return Err(InterpreterError::new_with_pos(
                    "Subprogram not defined",
                    self.function_context
                        .get_function_declaration_pos(name)
                        .unwrap(),
                ));
            }
        }

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
        let q = x.qualifier();
        for r in x.ranges() {
            match *r {
                LetterRangeNode::Single(c) => self.resolver_helper.set(c, c, q),
                LetterRangeNode::Range(start, stop) => self.resolver_helper.set(start, stop, q),
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

impl<S: Stdlib> PushPopContext for Interpreter<S> {
    fn push_context(&mut self, result_name: QualifiedName) {
        self.context = self.context.take().map(|x| x.push(result_name));
    }

    fn pop_context(&mut self) {
        self.context = self.context.take().map(|x| x.pop());
    }
}

#[derive(Debug)]
struct ResolverHelper {
    ranges: [TypeQualifier; 26],
}

fn char_to_alphabet_index(ch: char) -> usize {
    let upper = ch.to_ascii_uppercase();
    if upper >= 'A' && upper <= 'Z' {
        ((upper as u8) - ('A' as u8)) as usize
    } else {
        panic!(format!("Not a latin letter {}", ch))
    }
}

impl ResolverHelper {
    pub fn new() -> Self {
        ResolverHelper {
            ranges: [TypeQualifier::BangSingle; 26],
        }
    }

    pub fn set(&mut self, start: char, stop: char, qualifier: TypeQualifier) {
        let mut x: usize = char_to_alphabet_index(start);
        let y: usize = char_to_alphabet_index(stop);
        while x <= y {
            self.ranges[x] = qualifier;
            x += 1;
        }
    }
}

impl TypeResolver for ResolverHelper {
    fn resolve(&self, name: &CaseInsensitiveString) -> TypeQualifier {
        let x = char_to_alphabet_index(name.first_char());
        self.ranges[x]
    }
}

impl<TStdlib: Stdlib> TypeResolver for Interpreter<TStdlib> {
    fn resolve(&self, name: &CaseInsensitiveString) -> TypeQualifier {
        self.resolver_helper.resolve(name)
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
