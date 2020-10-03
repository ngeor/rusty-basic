mod assignment;
mod bare_name_types;
mod constant;
mod context;
mod converter;
mod dim;
mod expression;
mod for_loop;
mod function_implementation;
mod if_blocks;
mod param_name;
mod print_node;
mod program;
mod select_case;
mod statement;
mod sub_call;
mod sub_implementation;
mod sub_program_type;
mod top_level_token;

use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::types::{FunctionMap, ProgramNode, SubMap, UserDefinedTypes};
use crate::parser;
use crate::parser::BareName;
use std::collections::HashSet;

pub fn convert(
    program: parser::ProgramNode,
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
