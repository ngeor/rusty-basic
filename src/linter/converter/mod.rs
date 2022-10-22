//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
mod assignment;
mod const_rules;
mod converter;
mod dim_rules;
mod do_loop;
mod expr_rules;
mod for_loop;
mod function_implementation;
mod if_blocks;
mod names;
mod print_node;
mod select_case;
mod statement;
mod sub_call;
mod sub_implementation;
mod traits;

pub use self::converter::convert;
