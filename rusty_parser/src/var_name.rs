//! Used by dim_name and param_name who have almost identical parsing rules.

use crate::name::{bare_name_without_dots, name_with_dots};
use crate::pc::*;
use crate::pc_specific::*;
use crate::{
    BareName, Keyword, Name, ParseError, TypedName, VarTypeNewBuiltInCompact,
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
    array_p: impl Parser<RcStringView, Output = A>,
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
/// The parameters `name` and `array_param` have already been parsed by [var_name].
/// The `built_in_extended_factory` parses extended types (but only built-in).
fn name_chain<T, F, P>(
    name: &Name,
    built_in_extended_factory: F,
) -> impl Parser<RcStringView, Output = T>
where
    T: Clone + Default + 'static,
    T: VarTypeNewBuiltInCompact,
    T: VarTypeNewUserDefined,
    F: Fn() -> P,
    P: Parser<RcStringView, Output = T> + 'static,
{
    let has_dots = name.bare_name().contains('.');
    match name.qualifier() {
        // TODO do not use OrParser of 1 element as a workaround for dyn boxing
        // qualified name can't have an "AS" clause
        Some(q) => OrParser::new(vec![Box::new(once_p(T::new_built_in_compact(q)))]),
        // bare names might have an "AS" clause
        _ => OrParser::new(vec![Box::new(
            as_clause()
                .and_without_undo_keep_right(
                    iif_p(
                        has_dots,
                        built_in_extended_factory(),
                        any_extended(built_in_extended_factory()),
                    )
                    .no_incomplete(),
                )
                .or(once_p(T::default())),
        )]),
    }
}

fn as_clause() -> impl Parser<RcStringView, Output = (Token, Token, Token)> {
    seq2(
        whitespace().and_tuple(keyword(Keyword::As)),
        whitespace().no_incomplete(),
        |(a, b), c| (a, b, c),
    )
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

/// A parser that returns the given value only once.
#[deprecated]
fn once_p<V>(value: V) -> Once<V> {
    Once(value)
}

struct Once<V>(V);

impl<V: Clone> Parser<RcStringView> for Once<V> {
    type Output = V;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, Self::Output, ParseError> {
        // TODO remove the need for clone
        Ok((input, self.0.clone()))
    }
}
