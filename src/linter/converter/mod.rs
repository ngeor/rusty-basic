mod assignment;
mod context;
mod do_loop;
mod for_loop;
mod function_implementation;
mod if_blocks;
mod print_node;
mod program;
mod select_case;
mod statement;
mod sub_call;
mod sub_implementation;
mod top_level_token;

use crate::common::*;
use crate::linter::converter::context::Context;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{
    BareName, FunctionMap, ProgramNode, QualifiedNameNode, SubMap, TypeQualifier, UserDefinedTypes,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub fn convert(
    program: ProgramNode,
    f_c: &FunctionMap,
    s_c: &SubMap,
    user_defined_types: &UserDefinedTypes,
) -> Result<(ProgramNode, HashSet<BareName>), QErrorNode> {
    let mut converter = ConverterImpl::new(user_defined_types, f_c, s_c);
    let result = converter.convert(program)?;
    // consume
    let names_without_dot = converter.consume();
    Ok((result, names_without_dot))
}

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
type Implicits = Vec<QualifiedNameNode>;

/// Alias for the result of returning something together with any implicit
/// variables collected during its conversion.
type R<T> = Result<(T, Implicits), QErrorNode>;

//
// Converter trait
//

trait Converter<A, B> {
    fn convert(&mut self, a: A) -> Result<B, QErrorNode>;
}

// blanket for Vec
impl<T, A, B> Converter<Vec<A>, Vec<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Vec<A>) -> Result<Vec<B>, QErrorNode> {
        a.into_iter().map(|x| self.convert(x)).collect()
    }
}

// blanket for Option
impl<T, A, B> Converter<Option<A>, Option<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Option<A>) -> Result<Option<B>, QErrorNode> {
        match a {
            Some(x) => self.convert(x).map(|r| Some(r)),
            None => Ok(None),
        }
    }
}

// blanket for Locatable
impl<T, A, B> Converter<Locatable<A>, Locatable<B>> for T
where
    T: Converter<A, B>,
{
    fn convert(&mut self, a: Locatable<A>) -> Result<Locatable<B>, QErrorNode> {
        let Locatable { element, pos } = a;
        self.convert(element).map(|x| x.at(pos)).patch_err_pos(pos)
    }
}

//
// ConverterWithImplicitVariables
//

trait ConverterWithImplicitVariables<A, B> {
    fn convert_and_collect_implicit_variables(&mut self, a: A) -> R<B>;
}

// blanket for Option

impl<T, A, B> ConverterWithImplicitVariables<Option<A>, Option<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(&mut self, a: Option<A>) -> R<Option<B>> {
        match a {
            Some(a) => self
                .convert_and_collect_implicit_variables(a)
                .map(|(a, implicit_variables)| (Some(a), implicit_variables)),
            None => Ok((None, vec![])),
        }
    }
}

// blanket for Box

impl<T, A, B> ConverterWithImplicitVariables<Box<A>, Box<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(&mut self, a: Box<A>) -> R<Box<B>> {
        let unboxed: A = *a;
        let (converted, implicit_variables) =
            self.convert_and_collect_implicit_variables(unboxed)?;
        Ok((Box::new(converted), implicit_variables))
    }
}

// blanket for Vec

impl<T, A, B> ConverterWithImplicitVariables<Vec<A>, Vec<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(&mut self, a: Vec<A>) -> R<Vec<B>> {
        let mut result: Vec<B> = vec![];
        let mut total_implicit: Implicits = vec![];
        for i in a {
            let (b, mut implicit) = self.convert_and_collect_implicit_variables(i)?;
            result.push(b);
            total_implicit.append(&mut implicit);
        }
        Ok((result, total_implicit))
    }
}

//
// Converter
//

struct ConverterImpl<'a> {
    pub resolver: Rc<RefCell<TypeResolverImpl>>,
    pub functions: &'a FunctionMap,
    pub subs: &'a SubMap,
    pub user_defined_types: &'a UserDefinedTypes,
    pub context: Context<'a>,
}

impl<'a> ConverterImpl<'a> {
    pub fn new(
        user_defined_types: &'a UserDefinedTypes,
        functions: &'a FunctionMap,
        subs: &'a SubMap,
    ) -> Self {
        let resolver = Rc::new(RefCell::new(TypeResolverImpl::new()));
        Self {
            user_defined_types,
            resolver: Rc::clone(&resolver),
            functions,
            subs,
            context: Context::new(functions, subs, user_defined_types, Rc::clone(&resolver)),
        }
    }

    pub fn consume(self) -> HashSet<BareName> {
        self.context.names_without_dot()
    }

    pub fn merge_implicit_vars(lists: Vec<Implicits>) -> Implicits {
        let mut result: Implicits = vec![];
        for mut list in lists {
            result.append(&mut list);
        }
        result
    }
}

impl<'a> TypeResolver for ConverterImpl<'a> {
    fn resolve_char(&self, ch: char) -> TypeQualifier {
        self.resolver.borrow().resolve_char(ch)
    }
}
