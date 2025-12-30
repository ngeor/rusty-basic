use rusty_pc::boxed::boxed;
use rusty_pc::supplier::supplier;
use rusty_pc::*;

use crate::input::RcStringView;
use crate::specific::core::name::{bare_name_without_dots, name_with_dots};
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::ParseError;

/// A variable name with a type.
///
/// This is an abstraction to address the similarities between [DimVar]
/// and [Parameter].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedName<T: VarType> {
    bare_name: BareName,
    var_type: T,
}

impl<T: VarType> TypedName<T> {
    pub fn new(bare_name: BareName, var_type: T) -> Self {
        Self {
            bare_name,
            var_type,
        }
    }

    pub fn var_type(&self) -> &T {
        self.as_ref()
    }

    pub fn try_map_type<F, E>(self, f: F) -> Result<Self, E>
    where
        F: FnOnce(T) -> Result<T, E>,
    {
        let Self {
            bare_name,
            var_type,
        } = self;
        f(var_type).map(|new_var_type| Self::new(bare_name, new_var_type))
    }
}

impl<T: VarType> AsRef<T> for TypedName<T> {
    fn as_ref(&self) -> &T {
        &self.var_type
    }
}

impl<T: VarType> From<TypedName<T>> for (BareName, T) {
    fn from(value: TypedName<T>) -> Self {
        (value.bare_name, value.var_type)
    }
}

impl<T: VarType> AsBareName for TypedName<T> {
    fn as_bare_name(&self) -> &BareName {
        &self.bare_name
    }
}

impl<T: VarType> ToBareName for TypedName<T> {
    fn to_bare_name(self) -> BareName {
        self.bare_name
    }
}

pub trait VarType: HasExpressionType {
    fn new_built_in_compact(q: TypeQualifier) -> Self;

    fn new_built_in_extended(q: TypeQualifier) -> Self;

    fn new_user_defined(bare_name_pos: BareNamePos) -> Self;

    fn is_extended(&self) -> bool;

    fn as_user_defined_recursively(&self) -> Option<&BareNamePos>;

    fn to_qualifier_recursively(&self) -> Option<TypeQualifier>;
}

impl<T: VarType> HasExpressionType for TypedName<T> {
    fn expression_type(&self) -> ExpressionType {
        self.var_type.expression_type()
    }
}

// Used by dim_name and param_name who have almost identical parsing rules.

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
pub(super) fn var_name<T, A, P>(
    array_p: impl Parser<RcStringView, Output = A, Error = ParseError> + 'static,
    built_in_extended_factory: fn() -> P,
) -> impl Parser<RcStringView, Output = TypedName<T>, Error = ParseError>
where
    T: Clone + Default + VarType + CreateArray<ArrayDimensions = A> + 'static,
    P: Parser<RcStringView, Output = T, Error = ParseError> + 'static,
{
    Seq2::new(name_with_dots(), array_p).then_with(
        // TODO the name_chain prevents upgrading to `edition = 2024`
        move |(name, _)| name_chain(name, built_in_extended_factory),
        |(name, array), var_type| {
            let bare_name: BareName = name.to_bare_name();
            let final_type = var_type.create_array(array);
            TypedName::new(bare_name, final_type)
        },
    )
}

pub(super) trait CreateArray: VarType {
    type ArrayDimensions;

    fn create_array(self, array_dimensions: Self::ArrayDimensions) -> Self;
}

impl CreateArray for DimType {
    type ArrayDimensions = ArrayDimensions;

    fn create_array(self, array_dimensions: Self::ArrayDimensions) -> Self {
        if array_dimensions.is_empty() {
            self
        } else {
            Self::Array(array_dimensions, Box::new(self))
        }
    }
}

impl CreateArray for ParamType {
    // LParen, RParen
    type ArrayDimensions = Option<(Token, Token)>;

    fn create_array(self, array_dimensions: Self::ArrayDimensions) -> Self {
        if array_dimensions.is_none() {
            self
        } else {
            Self::Array(Box::new(self))
        }
    }
}

/// Used in combination with [var_name], produces the type of the variable
/// (e.g. [DimType] or [ParamType]).
///
/// The parameter `name` has already been parsed by [var_name].
/// The `built_in_extended_factory` parses extended types (but only built-in).
fn name_chain<T, F, P>(
    name: &Name,
    built_in_extended_factory: F,
) -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: Clone + Default + VarType + 'static,
    F: Fn() -> P + 'static,
    P: Parser<RcStringView, Output = T, Error = ParseError> + 'static,
{
    match name.qualifier() {
        // qualified name can't have an "AS" clause
        Some(q) => boxed(qualified_type(q)),
        // bare names might have an "AS" clause
        _ => {
            let allow_user_defined = !name.as_bare_name().contains('.');

            boxed(
                as_clause()
                    .and_keep_right(
                        extended(allow_user_defined, built_in_extended_factory).no_incomplete(),
                    )
                    .or(bare_type()),
            )
        }
    }
}

fn qualified_type<T>(q: TypeQualifier) -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: VarType,
{
    supplier(move || T::new_built_in_compact(q))
}

fn bare_type<T>() -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: Default,
{
    supplier(T::default)
}

fn as_clause() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    keyword(Keyword::As).surround(whitespace(), whitespace())
}

fn extended<T, F, P>(
    allow_user_defined: bool,
    built_in_extended_factory: F,
) -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: VarType + 'static,
    F: Fn() -> P,
    P: Parser<RcStringView, Output = T, Error = ParseError> + 'static,
{
    if allow_user_defined {
        boxed(any_extended(built_in_extended_factory()))
    } else {
        boxed(built_in_extended_factory())
    }
}

fn any_extended<T>(
    built_in_parser: impl Parser<RcStringView, Output = T, Error = ParseError> + 'static,
) -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: VarType + 'static,
{
    OrParser::new(vec![
        Box::new(built_in_parser),
        Box::new(user_defined_type()),
    ])
    .with_expected_message("Expected: INTEGER or LONG or SINGLE or DOUBLE or STRING or identifier")
}

fn user_defined_type<T>() -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: VarType,
{
    bare_name_without_dots()
        .with_pos()
        .map(VarType::new_user_defined)
}
