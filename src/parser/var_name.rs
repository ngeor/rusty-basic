//! Used by dim_name and param_name who have almost identical parsing rules.

use crate::common::QError;
use crate::parser::name::{bare_name_without_dots, name_with_dots};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::{
    BareName, Keyword, Name, TypeQualifier, VarName, VarTypeNewBuiltInCompact,
    VarTypeNewUserDefined, VarTypeToArray,
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
    array_p: impl Parser<Output = A> + NonOptParser,
    built_in_extended_factory: fn() -> P,
) -> impl Parser<Output = VarName<T>>
where
    T: Default,
    T: VarTypeNewBuiltInCompact,
    T: VarTypeNewUserDefined,
    T: VarTypeToArray<ArrayType = A>,
    P: Parser<Output = T> + 'static,
{
    Seq2::new(name_with_dots(), array_p)
        .chain(move |(name, array)| name_chain(name, array, built_in_extended_factory))
        .map(move |(name, array, var_type)| {
            let bare_name: BareName = name.into();
            let final_type = var_type.to_array(array);
            VarName::new(bare_name, final_type)
        })
}

/// Used in combination with [var_name], produces the type of the variable
/// (e.g. [DimType] or [ParamType]).
///
/// The parameters `name` and `array_param` have already been parsed by [var_name].
/// The `built_in_extended_factory` parses extended types (but only built-in).
fn name_chain<A, T, F, P>(
    name: Name,
    array_param: A,
    built_in_extended_factory: F,
) -> impl ParserOnce<Output = (Name, A, T)>
where
    T: Default,
    T: VarTypeNewBuiltInCompact,
    T: VarTypeNewUserDefined,
    F: Fn() -> P,
    P: Parser<Output = T>,
{
    let has_dots = name.bare_name().contains('.');
    match_option_p(
        name.qualifier(),
        // qualified name can't have an "AS" clause
        |q: TypeQualifier| once_p(T::new_built_in_compact(q)),
        // bare names might have an "AS" clause
        move || {
            as_clause()
                .then_demand(
                    iif_p(
                        has_dots,
                        built_in_extended_factory(),
                        any_extended(built_in_extended_factory()),
                    )
                    .no_incomplete(),
                )
                .to_parser_once()
                .or(once_p(T::default()))
        },
    )
    .map(|param_type| (name, array_param, param_type))
}

fn as_clause() -> impl Parser<Output = (Token, Token, Token)> {
    seq2(
        whitespace().and(keyword(Keyword::As)),
        whitespace().no_incomplete(),
        |(a, b), c| (a, b, c),
    )
}

fn any_extended<T>(built_in_parser: impl Parser<Output = T>) -> impl Parser<Output = T>
where
    T: VarTypeNewUserDefined,
{
    built_in_parser
        .or(user_defined_type())
        .map_incomplete_err(QError::expected(
            "Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier",
        ))
}

fn user_defined_type<T>() -> impl Parser<Output = T>
where
    T: VarTypeNewUserDefined,
{
    bare_name_without_dots()
        .with_pos()
        .map(VarTypeNewUserDefined::new_user_defined)
}
