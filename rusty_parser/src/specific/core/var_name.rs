use std::marker::PhantomData;

use rusty_pc::*;

use crate::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::specific::core::name::{bare_name_without_dots, name_with_dots};
use crate::specific::*;

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
pub(super) fn var_name<T, A, B, AP, BP>(
    opt_array_parser_factory: A,
    extended_type_parser_factory: B,
) -> impl Parser<RcStringView, Output = TypedName<T>, Error = ParseError>
where
    T: Default + VarType + CreateArray<ArrayDimensions = AP::Output> + 'static,
    A: Fn() -> AP,
    AP: Parser<RcStringView, Error = ParseError> + 'static,
    B: Fn(bool) -> BP,
    BP: Parser<RcStringView, Output = T, Error = ParseError> + 'static,
{
    VarNameParser {
        opt_array_parser_factory,
        extended_type_parser_factory,
        _phantom: PhantomData,
    }
}

struct VarNameParser<T, A, B> {
    opt_array_parser_factory: A,
    extended_type_parser_factory: B,
    _phantom: PhantomData<T>,
}

impl<T, A, B, AP, BP> Parser<RcStringView> for VarNameParser<T, A, B>
where
    T: Default + VarType + CreateArray<ArrayDimensions = AP::Output> + 'static,
    A: Fn() -> AP,
    AP: Parser<RcStringView, Error = ParseError> + 'static,
    B: Fn(bool) -> BP,
    BP: Parser<RcStringView, Output = T, Error = ParseError> + 'static,
{
    type Output = TypedName<T>;
    type Error = ParseError;

    fn parse(&self, input: RcStringView) -> ParseResult<RcStringView, Self::Output, Self::Error> {
        let (input, (name, array)) =
            name_with_opt_array((self.opt_array_parser_factory)()).parse(input)?;

        match name.qualifier() {
            // qualified name can't have an "AS" clause
            Some(q) => Ok((
                input,
                create_typed_name(name, array, T::new_built_in_compact(q)),
            )),
            // bare names might have an "AS" clause
            _ => {
                let allow_user_defined = !name.as_bare_name().contains('.');
                let extended_type_parser =
                    (self.extended_type_parser_factory)(allow_user_defined).no_incomplete();
                match as_clause()
                    .and_keep_right(extended_type_parser)
                    .parse(input)
                {
                    Ok((input, var_type)) => Ok((input, create_typed_name(name, array, var_type))),
                    Err((false, input, _)) => {
                        Ok((input, create_typed_name(name, array, T::default())))
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}

fn name_with_opt_array<P>(
    opt_array_parser: P,
) -> impl Parser<RcStringView, Output = (Name, P::Output), Error = ParseError>
where
    P: Parser<RcStringView, Error = ParseError> + 'static,
{
    Seq2::new(name_with_dots(), opt_array_parser)
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

fn as_clause() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    keyword(Keyword::As).surround(whitespace(), whitespace())
}

pub(crate) fn user_defined_type<T>() -> impl Parser<RcStringView, Output = T, Error = ParseError>
where
    T: VarType,
{
    bare_name_without_dots()
        .with_pos()
        .map(VarType::new_user_defined)
}
