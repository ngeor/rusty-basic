use crate::common::*;
use crate::linter::converter::context::Context;
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::parser::{BareName, FunctionMap, QualifiedNameNode, SubMap, UserDefinedTypes};
use std::collections::HashSet;

//
// Converter trait
//

pub trait Converter<A, B> {
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

pub trait ConverterWithImplicitVariables<A, B> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: A,
    ) -> Result<(B, Vec<QualifiedNameNode>), QErrorNode>;
}

// blanket for Option

impl<T, A, B> ConverterWithImplicitVariables<Option<A>, Option<B>> for T
where
    T: ConverterWithImplicitVariables<A, B>,
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: Option<A>,
    ) -> Result<(Option<B>, Vec<QualifiedNameNode>), QErrorNode> {
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
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: Box<A>,
    ) -> Result<(Box<B>, Vec<QualifiedNameNode>), QErrorNode> {
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
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: Vec<A>,
    ) -> Result<(Vec<B>, Vec<QualifiedNameNode>), QErrorNode> {
        let mut result: Vec<B> = vec![];
        let mut total_implicit: Vec<QualifiedNameNode> = vec![];
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

#[derive(Debug)]
pub struct ConverterImpl<'a> {
    pub resolver: TypeResolverImpl,
    pub context: Context<'a>,
    pub functions: &'a FunctionMap,
    pub subs: &'a SubMap,
    pub user_defined_types: &'a UserDefinedTypes,
}

impl<'a> ConverterImpl<'a> {
    pub fn new(
        user_defined_types: &'a UserDefinedTypes,
        functions: &'a FunctionMap,
        subs: &'a SubMap,
    ) -> Self {
        Self {
            user_defined_types,
            resolver: TypeResolverImpl::new(),
            context: Context::new(user_defined_types),
            functions,
            subs,
        }
    }

    fn take_context(&mut self) -> Context<'a> {
        let tmp = Context::new(&self.user_defined_types);
        std::mem::replace(&mut self.context, tmp)
    }

    pub fn push_function_context(&mut self, bare_function_name: BareName) {
        let old = self.take_context();
        self.context = old.push_function_context(bare_function_name);
    }

    pub fn push_sub_context(&mut self, sub_name: BareName) {
        let old = self.take_context();
        self.context = old.push_sub_context(sub_name);
    }

    pub fn pop_context(&mut self) {
        let old = self.take_context();
        self.context = old.pop_context();
    }

    pub fn consume(self) -> HashSet<BareName> {
        self.context.names_without_dot()
    }

    pub fn merge_implicit_vars(lists: Vec<Vec<QualifiedNameNode>>) -> Vec<QualifiedNameNode> {
        let mut result: Vec<QualifiedNameNode> = vec![];
        for mut list in lists {
            result.append(&mut list);
        }
        result
    }
}
