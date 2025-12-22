//! Used by dim_name and param_name who have almost identical parsing rules.

use crate::name::{bare_name_without_dots, name_with_dots};
use crate::pc::boxed::boxed;
use crate::pc::supplier::supplier;
use crate::pc_specific::*;
use crate::{pc::*, TypeQualifier};
use crate::{
    BareName, Keyword, Name, TypedName, VarTypeNewBuiltInCompact, VarTypeNewUserDefined,
    VarTypeToArray,
};

/// Parses a variable name (dim name or param name).
///
/// Parameters:
///
/// `array_p`:
///   Parses an optional array indicator. In case of dim, it's the array dimensions.
///   In case of param, it's a `()` placeholder.
///
/// `built_in_extended_factory`:
///   A function that produces the parser of extended built-in types.
///   For `DIM`, this includes also `STRING * 5`.
///
/// Type parameters:
///
/// - `T`: The type of the variable (e.g. [DimType], [ParamType])
/// - `A`: The type of the array indicator
/// - `P`: The parser that parses `T` for extended built-in types.
pub fn var_name<T, A, P>(
    array_p: impl Parser<RcStringView, Output = A> + 'static,
    built_in_extended_factory: fn() -> P,
) -> impl Parser<RcStringView, Output = TypedName<T>>
where
    T: Clone + Default + 'static,
    T: VarTypeNewBuiltInCompact,
    T: VarTypeNewUserDefined,
    T: VarTypeToArray<ArrayType = A>,
    P: Parser<RcStringView, Output = T> + 'static,
{
    Seq2::new(name_with_dots(), array_p).chain(
        move |(name, _)| name_chain(name, built_in_extended_factory),
        |(name, array), var_type| {
            let bare_name: BareName = name.into();
            let final_type = var_type.to_array(array);
            TypedName::new(bare_name, final_type)
        },
    )
}

/// Used in combination with [var_name], produces the type of the variable
/// (e.g. [DimType] or [ParamType]).
///
/// The parameter `name` has already been parsed by [var_name].
/// The `built_in_extended_factory` parses extended types (but only built-in).
fn name_chain<T, F, P>(
    name: &Name,
    built_in_extended_factory: F,
) -> impl Parser<RcStringView, Output = T>
where
    T: Clone + Default + 'static,
    T: VarTypeNewBuiltInCompact,
    T: VarTypeNewUserDefined,
    F: Fn() -> P + 'static,
    P: Parser<RcStringView, Output = T> + 'static,
{
    match name.qualifier() {
        // qualified name can't have an "AS" clause
        Some(q) => boxed(qualified_type(q)),
        // bare names might have an "AS" clause
        _ => {
            let allow_user_defined = !name.bare_name().contains('.');

            boxed(
                as_clause()
                    .and_without_undo_keep_right(
                        extended(allow_user_defined, built_in_extended_factory).no_incomplete(),
                    )
                    .or(bare_type()),
            )
        }
    }
}

fn qualified_type<T>(q: TypeQualifier) -> impl Parser<RcStringView, Output = T>
where
    T: VarTypeNewBuiltInCompact,
{
    supplier(move || T::new_built_in_compact(q))
}

fn bare_type<T>() -> impl Parser<RcStringView, Output = T>
where
    T: Default,
{
    supplier(T::default)
}

fn as_clause() -> impl Parser<RcStringView, Output = Token> {
    keyword(Keyword::As).surround(whitespace(), whitespace())
}

fn extended<T, F, P>(
    allow_user_defined: bool,
    built_in_extended_factory: F,
) -> impl Parser<RcStringView, Output = T>
where
    T: VarTypeNewUserDefined + 'static,
    F: Fn() -> P,
    P: Parser<RcStringView, Output = T> + 'static,
{
    if allow_user_defined {
        boxed(any_extended(built_in_extended_factory()))
    } else {
        boxed(built_in_extended_factory())
    }
}

fn any_extended<T>(
    built_in_parser: impl Parser<RcStringView, Output = T> + 'static,
) -> impl Parser<RcStringView, Output = T>
where
    T: VarTypeNewUserDefined + 'static,
{
    OrParser::new(vec![
        Box::new(built_in_parser),
        Box::new(user_defined_type()),
    ])
    .with_expected_message("Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier")
}

fn user_defined_type<T>() -> impl Parser<RcStringView, Output = T>
where
    T: VarTypeNewUserDefined,
{
    bare_name_without_dots()
        .with_pos()
        .map(VarTypeNewUserDefined::new_user_defined)
}
