use rusty_pc::and::TupleCombiner;
use rusty_pc::*;

use crate::core::name::{bare_name_without_dots, name_p};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::{
    ArrayDimensions, AsBareName, BareName, BareNamePos, DimType, ExpressionType, HasExpressionType, Keyword, Name, ParamType, ParserError, ToBareName, TypeQualifier
};

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

/// The context used by the `var_name` parser.
/// The first part is the optional qualifier of the parser var name.
/// The second part is whether user defined types are allowed.
pub(super) type VarNameCtx = (Option<TypeQualifier>, bool);

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
pub(super) fn var_name<T, A, B, AP, BP>(
    opt_array_parser_factory: A,
    extended_type_parser_factory: B,
) -> impl Parser<StringView, Output = TypedName<T>, Error = ParserError>
where
    T: Default + VarType + CreateArray<ArrayDimensions = AP::Output>,
    A: Fn() -> AP,
    AP: Parser<StringView, Error = ParserError> + 'static,
    B: Fn() -> BP + 'static,
    BP: Parser<StringView, VarNameCtx, Output = T, Error = ParserError> + 'static,
{
    name_with_opt_array(opt_array_parser_factory())
        .then_with_in_context(
            var_type_parser(extended_type_parser_factory()).map_ctx(
                |(name, _array): &(Name, _)| (name.qualifier(), !name.as_bare_name().contains('.')),
            ),
            TupleCombiner,
        )
        .map(|((name, array), var_type)| create_typed_name(name, array, var_type))
}

fn var_type_parser<T, BP>(
    extended_type_parser: BP,
) -> impl Parser<StringView, VarNameCtx, Output = T, Error = ParserError>
where
    T: Default + VarType,
    BP: Parser<StringView, VarNameCtx, Output = T, Error = ParserError> + 'static,
{
    qualified().or(extended(extended_type_parser)).or(bare())
}

fn qualified<T>() -> impl Parser<StringView, VarNameCtx, Output = T, Error = ParserError>
where
    T: Default + VarType,
{
    // get the context
    ctx_parser()
        // if the parsed name is qualified, return the result, otherwise soft error,
        // so that the next option (extended) can be invoked
        .and_then(|(opt_q, _)| match opt_q {
            Some(q) => Ok(T::new_built_in_compact(q)),
            None => Err(ParserError::default()),
        })
        // the following `and` will be invoked only if we had a result
        // we want to validate that a qualified variable is not
        // followed by an `AS` clause, in order to simulate the error that QBasic
        // throws when that happens
        .and_tuple(as_clause().peek().to_option().no_context())
        .and_then(|(result, has_opt_clause)| {
            if has_opt_clause.is_some() {
                Err(ParserError::syntax_error(
                    "Identifier cannot end with %, &, !, #, or $",
                ))
            } else {
                Ok(result)
            }
        })
}

fn extended<T, BP>(
    extended_type_parser: BP,
) -> impl Parser<StringView, VarNameCtx, Output = T, Error = ParserError>
where
    T: Default + VarType,
    BP: Parser<StringView, VarNameCtx, Output = T, Error = ParserError>,
{
    let extended_type_parser = demand_lead_ws_ctx(extended_type_parser.to_fatal());
    as_clause()
        .no_context()
        .and_keep_right(extended_type_parser)
}

fn bare<T>() -> impl Parser<StringView, VarNameCtx, Output = T, Error = ParserError>
where
    T: Default + VarType,
{
    supplier(T::default)
}

fn name_with_opt_array<P>(
    opt_array_parser: P,
) -> impl Parser<StringView, Output = (Name, P::Output), Error = ParserError>
where
    P: Parser<StringView, Error = ParserError> + 'static,
{
    seq2(name_p(), opt_array_parser, |name, array| (name, array))
}

fn create_typed_name<T, A>(name: Name, array: A, var_type: T) -> TypedName<T>
where
    T: VarType + CreateArray<ArrayDimensions = A>,
{
    let bare_name: BareName = name.to_bare_name();
    let final_type = var_type.create_array(array);
    TypedName::new(bare_name, final_type)
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

fn as_clause() -> impl Parser<StringView, Output = (), Error = ParserError> {
    lead_ws(keyword_ignoring(Keyword::As))
}

pub(crate) fn user_defined_type<T>() -> impl Parser<StringView, Output = T, Error = ParserError>
where
    T: VarType,
{
    bare_name_without_dots()
        .with_pos()
        .map(VarType::new_user_defined)
}
