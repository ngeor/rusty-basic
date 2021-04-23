use super::Context;
use crate::common::*;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::{ConverterWithImplicitVariables, ExprContext, Implicits, R};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::DimContext;
use crate::parser::*;
use crate::variant::MAX_INTEGER;

pub fn on_dim(ctx: &mut Context, dim_list: DimList, dim_context: DimContext) -> R<DimList> {
    dim_list.convert2(ctx, dim_context)
}

trait Converter2<C> {
    fn convert2(self, context: &mut Context, extra: C) -> R<Self>
    where
        Self: Sized;
}

impl<'a> ConverterWithImplicitVariables<ArrayDimension, ArrayDimension> for Context<'a> {
    fn convert_and_collect_implicit_variables(&mut self, a: ArrayDimension) -> R<ArrayDimension> {
        let (lbound, mut implicits) = self.on_opt_expression(a.lbound, ExprContext::Default)?;
        let (ubound, mut ubound_implicits) = self.on_expression(a.ubound, ExprContext::Default)?;
        implicits.append(&mut ubound_implicits);
        Ok((ArrayDimension { lbound, ubound }, implicits))
    }
}

impl Converter2<DimContext> for DimList {
    fn convert2(self, context: &mut Context, extra: DimContext) -> R<Self> {
        let Self { variables, shared } = self;
        let new_extra = (extra, shared);
        let mut converted_variables: DimNameNodes = vec![];
        let mut implicits: Implicits = vec![];
        for variable in variables {
            let (converted_variable, mut partial_implicits) =
                variable.convert2(context, new_extra)?;
            converted_variables.push(converted_variable);
            implicits.append(&mut partial_implicits);
        }
        let converted_dim_list = Self {
            variables: converted_variables,
            shared,
        };
        Ok((converted_dim_list, implicits))
    }
}

impl Converter2<(DimContext, bool)> for DimNameNode {
    fn convert2(self, context: &mut Context, extra: (DimContext, bool)) -> R<Self> {
        let (dim_context, shared) = extra;
        let Locatable {
            element: DimName {
                bare_name,
                dim_type,
            },
            pos,
        } = self;
        convert(context, bare_name, dim_type, dim_context, shared, pos)
    }
}

fn convert(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> R<DimNameNode> {
    validate2::do_validate(ctx, &bare_name, &dim_type, dim_context, shared).patch_err_pos(pos)?;
    convert2::do_convert(ctx, bare_name, dim_type, dim_context, shared, pos)
}

mod validate2 {
    use super::*;
    use crate::linter::type_resolver::TypeResolver;
    use crate::parser::DimTypeTrait;

    pub fn do_validate(
        ctx: &Context,
        bare_name: &BareName,
        dim_type: &DimType,
        dim_context: DimContext,
        shared: bool,
    ) -> Result<(), QErrorNode> {
        validate2::cannot_clash_with_subs(ctx, bare_name).with_err_no_pos()?;
        validate2::cannot_clash_with_functions(ctx, bare_name, dim_type, dim_context)
            .with_err_no_pos()?;
        validate2::user_defined_type_must_exist(ctx, dim_type)?;
        validate2::cannot_clash_with_local_constants(ctx, bare_name).with_err_no_pos()?;
        validate2::shared_validation(ctx, dim_context, shared).with_err_no_pos()
    }

    fn cannot_clash_with_subs(ctx: &Context, bare_name: &BareName) -> Result<(), QError> {
        if ctx.subs.contains_key(bare_name) {
            Err(QError::DuplicateDefinition)
        } else {
            Ok(())
        }
    }

    fn cannot_clash_with_local_constants(
        ctx: &Context,
        bare_name: &BareName,
    ) -> Result<(), QError> {
        if ctx.names.contains_const(bare_name) {
            Err(QError::DuplicateDefinition)
        } else {
            Ok(())
        }
    }

    fn cannot_clash_with_functions(
        ctx: &Context,
        bare_name: &BareName,
        dim_type: &DimType,
        dim_context: DimContext,
    ) -> Result<(), QError> {
        if dim_context == DimContext::Param {
            if let Some(func_qualifier) = ctx.function_qualifier(bare_name) {
                if dim_type.is_extended() {
                    Err(QError::DuplicateDefinition)
                } else {
                    // for some reason you can have a FUNCTION Add(Add)
                    let q = ctx.resolve_dim_name_to_qualifier(bare_name, dim_type);
                    if q == func_qualifier {
                        Ok(())
                    } else {
                        Err(QError::DuplicateDefinition)
                    }
                }
            } else {
                Ok(())
            }
        } else {
            if ctx.functions.contains_key(bare_name) {
                Err(QError::DuplicateDefinition)
            } else {
                Ok(())
            }
        }
    }

    fn user_defined_type_must_exist(ctx: &Context, dim_type: &DimType) -> Result<(), QErrorNode> {
        match dim_type {
            DimType::UserDefined(Locatable {
                element: type_name,
                pos,
            }) => {
                if ctx.user_defined_types.contains_key(type_name) {
                    Ok(())
                } else {
                    Err(QError::TypeNotDefined).with_err_at(*pos)
                }
            }
            DimType::Array(_, element_type) => {
                user_defined_type_must_exist(ctx, element_type.as_ref())
            }
            _ => Ok(()),
        }
    }

    fn shared_validation(
        ctx: &Context,
        dim_context: DimContext,
        shared: bool,
    ) -> Result<(), QError> {
        if shared {
            // this should not happen based on the parser
            debug_assert_ne!(dim_context, DimContext::Param);
            if ctx.is_in_subprogram() {
                return Err(QError::IllegalInSubFunction);
            }
        }
        Ok(())
    }
}

mod convert2 {
    use super::*;
    use crate::linter::converter::names::Visitor;
    use crate::parser::{ArrayDimensions, BareNameNode, ExpressionNode};
    use crate::variant::QBNumberCast;

    pub fn do_convert(
        ctx: &mut Context,
        bare_name: BareName,
        dim_type: DimType,
        dim_context: DimContext,
        shared: bool,
        pos: Location,
    ) -> R<DimNameNode> {
        match dim_context {
            DimContext::Default | DimContext::Param => {
                do_convert_default(ctx, bare_name, dim_type, dim_context, shared, pos)
            }
            DimContext::Redim => do_convert_redim(ctx, bare_name, dim_type, shared, pos),
        }
    }

    fn do_convert_default(
        ctx: &mut Context,
        bare_name: BareName,
        dim_type: DimType,
        dim_context: DimContext,
        shared: bool,
        pos: Location,
    ) -> R<DimNameNode> {
        debug_assert_ne!(dim_context, DimContext::Redim);
        resolve_dim_type_default(ctx, &bare_name, &dim_type, dim_context, shared, pos).map(
            |(dim_type, implicits)| {
                ctx.names.insert(bare_name.clone(), &dim_type, shared, None);
                (DimName::new(bare_name, dim_type).at(pos), implicits)
            },
        )
    }

    fn resolve_dim_type_default(
        ctx: &mut Context,
        bare_name: &BareName,
        dim_type: &DimType,
        dim_context: DimContext,
        shared: bool,
        pos: Location,
    ) -> R<DimType> {
        debug_assert_ne!(dim_context, DimContext::Redim);
        match dim_type {
            DimType::Bare => bare_to_dim_type(ctx, bare_name, pos).map(no_implicits),
            DimType::BuiltIn(q, built_in_style) => {
                built_in_to_dim_type(ctx, bare_name, *q, *built_in_style, pos).map(no_implicits)
            }
            DimType::FixedLengthString(length_expression, resolved_length) => {
                debug_assert_eq!(*resolved_length, 0, "Should not be resolved yet");
                fixed_length_string_to_dim_type(ctx, bare_name, length_expression, pos)
                    .map(no_implicits)
            }
            DimType::UserDefined(u) => {
                user_defined_to_dim_type(ctx, bare_name, u, pos).map(no_implicits)
            }
            DimType::Array(array_dimensions, element_type) => array_to_dim_type(
                ctx,
                bare_name,
                array_dimensions,
                element_type.as_ref(),
                dim_context,
                shared,
                pos,
            ),
        }
    }

    struct BuiltInCompactVisitor(TypeQualifier);

    impl Visitor for BuiltInCompactVisitor {
        fn on_compact(
            &mut self,
            q: TypeQualifier,
            _variable_info: &VariableInfo,
        ) -> Result<(), QError> {
            if self.0 == q {
                Err(QError::DuplicateDefinition)
            } else {
                Ok(())
            }
        }

        fn on_extended(&mut self, _variable_info: &VariableInfo) -> Result<(), QError> {
            Err(QError::DuplicateDefinition)
        }
    }

    fn bare_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        let resolved_q = ctx.resolve(bare_name);
        let mut visitor = BuiltInCompactVisitor(resolved_q);
        ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
        Ok(DimType::BuiltIn(resolved_q, BuiltInStyle::Compact))
    }

    struct ExtendedVisitor;

    impl Visitor for ExtendedVisitor {
        fn on_compact(
            &mut self,
            _q: TypeQualifier,
            _variable_info: &VariableInfo,
        ) -> Result<(), QError> {
            Err(QError::DuplicateDefinition)
        }

        fn on_extended(&mut self, _variable_info: &VariableInfo) -> Result<(), QError> {
            Err(QError::DuplicateDefinition)
        }
    }

    fn built_in_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        q: TypeQualifier,
        built_in_style: BuiltInStyle,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        match built_in_style {
            BuiltInStyle::Compact => {
                let mut visitor = BuiltInCompactVisitor(q);
                ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
                Ok(DimType::BuiltIn(q, BuiltInStyle::Compact))
            }
            BuiltInStyle::Extended => {
                let mut visitor = ExtendedVisitor;
                ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
                Ok(DimType::BuiltIn(q, BuiltInStyle::Extended))
            }
        }
    }

    fn fixed_length_string_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        length_expression: &ExpressionNode,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        for (_, _) in ctx.names.names_iterator(bare_name) {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }
        let string_length: u16 = resolve_string_length(ctx, length_expression)?;
        Ok(DimType::fixed_length_string(
            string_length,
            length_expression.pos(),
        ))
    }

    fn resolve_string_length(
        ctx: &Context,
        length_expression: &ExpressionNode,
    ) -> Result<u16, QErrorNode> {
        let v = ctx.names.resolve_const_value_node(length_expression)?;
        let i: i32 = v.try_cast().with_err_at(length_expression)?;
        if i >= 1 && i < MAX_INTEGER {
            Ok(i as u16)
        } else {
            Err(QError::OutOfStringSpace).with_err_at(length_expression)
        }
    }

    fn user_defined_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        user_defined_type: &BareNameNode,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        let mut visitor = ExtendedVisitor;
        ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
        Ok(DimType::UserDefined(user_defined_type.clone()))
    }

    fn array_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        array_dimensions: &ArrayDimensions,
        element_type: &DimType,
        dim_context: DimContext,
        shared: bool,
        pos: Location,
    ) -> R<DimType> {
        debug_assert!(match dim_context {
            DimContext::Default => {
                array_dimensions.len() > 0
            }
            DimContext::Param => {
                array_dimensions.is_empty()
            }
            _ => true,
        });
        // TODO optimize array_dimensions.clone()
        let (converted_array_dimensions, mut implicits) =
            convert_array_dimensions(ctx, array_dimensions.clone())?;
        let (resolved_element_dim_type, mut resolved_implicits) =
            resolve_dim_type_default(ctx, bare_name, element_type, dim_context, shared, pos)?;
        implicits.append(&mut resolved_implicits);
        let array_dim_type = DimType::Array(
            converted_array_dimensions,
            Box::new(resolved_element_dim_type),
        );
        Ok((array_dim_type, implicits))
    }

    fn convert_array_dimensions(
        ctx: &mut Context,
        array_dimensions: ArrayDimensions,
    ) -> R<ArrayDimensions> {
        ctx.convert_and_collect_implicit_variables(array_dimensions)
    }

    fn do_convert_redim(
        ctx: &mut Context,
        bare_name: BareName,
        dim_type: DimType,
        shared: bool,
        pos: Location,
    ) -> R<DimNameNode> {
        if let DimType::Array(array_dimensions, element_type) = dim_type {
            let dimension_count = array_dimensions.len();
            let (converted_array_dimensions, implicits) =
                convert_array_dimensions(ctx, array_dimensions)?;
            debug_assert_eq!(dimension_count, converted_array_dimensions.len());
            let converted_element_type = redim_to_element_dim_type(
                ctx,
                &bare_name,
                &converted_array_dimensions,
                element_type.as_ref(),
                pos,
            )?;
            let array_dim_type =
                DimType::Array(converted_array_dimensions, Box::new(converted_element_type));
            ctx.names.insert(
                bare_name.clone(),
                &array_dim_type,
                shared,
                Some(RedimInfo { dimension_count }),
            );
            Ok((DimName::new(bare_name, array_dim_type).at(pos), implicits))
        } else {
            panic!("REDIM without array")
        }
    }

    fn redim_to_element_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        array_dimensions: &ArrayDimensions,
        element_dim_type: &DimType,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        match element_dim_type {
            DimType::Bare => redim_bare_to_dim_type(ctx, bare_name, array_dimensions, pos),
            DimType::BuiltIn(q, built_in_style) => redim_built_in_to_dim_type(
                ctx,
                bare_name,
                array_dimensions,
                *q,
                *built_in_style,
                pos,
            ),
            DimType::FixedLengthString(length_expression, resolved_length) => {
                debug_assert_eq!(
                    *resolved_length, 0,
                    "REDIM string length should not be known"
                );
                redim_fixed_length_string_to_dim_type(
                    ctx,
                    bare_name,
                    array_dimensions,
                    length_expression,
                    pos,
                )
            }
            DimType::UserDefined(u) => {
                redim_user_defined_type_to_dim_type(ctx, bare_name, array_dimensions, u, pos)
            }
            DimType::Array(_, _) => {
                panic!("REDIM nested array is not supported")
            }
        }
    }

    fn redim_bare_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        array_dimensions: &ArrayDimensions,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        let mut found: Option<(BuiltInStyle, &VariableInfo)> = None;
        let q = ctx.resolve(bare_name);
        for (built_in_style, variable_info) in ctx.names.names_iterator(bare_name) {
            match &variable_info.redim_info {
                Some(r) => {
                    if r.dimension_count != array_dimensions.len() {
                        return Err(QError::WrongNumberOfDimensions).with_err_at(pos);
                    }

                    match built_in_style {
                        BuiltInStyle::Compact => {
                            let opt_q: Option<TypeQualifier> =
                                variable_info.expression_type.opt_qualifier();
                            let existing_q = opt_q.expect("Should be qualified");
                            if existing_q == q {
                                debug_assert!(found.is_none());
                                found = Some((built_in_style, variable_info));
                            }
                        }
                        BuiltInStyle::Extended => {
                            debug_assert!(found.is_none());
                            found = Some((built_in_style, variable_info));
                        }
                    }
                }
                _ => {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }
            }
        }
        match found {
            Some((built_in_style, variable_info)) => {
                if let ExpressionType::Array(element_type) = &variable_info.expression_type {
                    match element_type.as_ref() {
                        ExpressionType::BuiltIn(q) => Ok(DimType::BuiltIn(*q, built_in_style)),
                        ExpressionType::FixedLengthString(len) => {
                            Ok(DimType::fixed_length_string(*len, pos))
                        }
                        ExpressionType::UserDefined(u) => {
                            Ok(DimType::UserDefined(u.clone().at(pos)))
                        }
                        _ => {
                            panic!("REDIM with nested array or unresolved type");
                        }
                    }
                } else {
                    panic!("REDIM without array");
                }
            }
            None => Ok(DimType::BuiltIn(q, BuiltInStyle::Compact)),
        }
    }

    fn redim_built_in_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        array_dimensions: &ArrayDimensions,
        q: TypeQualifier,
        built_in_style: BuiltInStyle,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        if built_in_style == BuiltInStyle::Compact {
            for (built_in_style, variable_info) in ctx.names.names_iterator(bare_name) {
                if built_in_style == BuiltInStyle::Extended {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }

                let opt_q = variable_info.expression_type.opt_qualifier();
                if opt_q.expect("Should be qualified") == q {
                    // other compact arrays of the same name are allowed to co-exist, hence no else block here
                    require_dimension_count(variable_info, array_dimensions.len())
                        .with_err_at(pos)?;
                }
            }
        } else {
            for (built_in_style, variable_info) in ctx.names.names_iterator(bare_name) {
                if built_in_style == BuiltInStyle::Compact {
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                }

                require_built_in_array(variable_info, q).with_err_at(pos)?;
                require_dimension_count(variable_info, array_dimensions.len()).with_err_at(pos)?;
            }
        }
        Ok(DimType::BuiltIn(q, built_in_style))
    }

    fn require_built_in_array(
        variable_info: &VariableInfo,
        q: TypeQualifier,
    ) -> Result<(), QError> {
        if let ExpressionType::Array(element_type) = &variable_info.expression_type {
            if let ExpressionType::BuiltIn(existing_q) = element_type.as_ref() {
                if q == *existing_q {
                    return Ok(());
                }
            }
        }
        Err(QError::DuplicateDefinition)
    }

    fn redim_fixed_length_string_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        array_dimensions: &ArrayDimensions,
        length_expression: &ExpressionNode,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        let string_length: u16 = resolve_string_length(ctx, length_expression)?;
        for (built_in_style, variable_info) in ctx.names.names_iterator(bare_name) {
            if built_in_style == BuiltInStyle::Compact {
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }

            require_fixed_length_string_array(variable_info, string_length).with_err_at(pos)?;
            require_dimension_count(variable_info, array_dimensions.len()).with_err_at(pos)?;
        }

        Ok(DimType::fixed_length_string(string_length, pos))
    }

    fn require_fixed_length_string_array(
        variable_info: &VariableInfo,
        len: u16,
    ) -> Result<(), QError> {
        if let ExpressionType::Array(element_type) = &variable_info.expression_type {
            if let ExpressionType::FixedLengthString(existing_len) = element_type.as_ref() {
                if len == *existing_len {
                    return Ok(());
                }
            }
        }
        Err(QError::DuplicateDefinition)
    }

    struct RedimUserDefinedTypeVisitor<'a>(usize, &'a BareName);

    impl<'a> Visitor for RedimUserDefinedTypeVisitor<'a> {
        fn on_compact(
            &mut self,
            _q: TypeQualifier,
            _variable_info: &VariableInfo,
        ) -> Result<(), QError> {
            Err(QError::DuplicateDefinition)
        }

        fn on_extended(&mut self, variable_info: &VariableInfo) -> Result<(), QError> {
            require_dimension_count(variable_info, self.0)
                .and_then(|_| require_user_defined_array(variable_info, self.1))
        }
    }

    fn require_dimension_count(
        variable_info: &VariableInfo,
        dimension_count: usize,
    ) -> Result<(), QError> {
        if let ExpressionType::Array(_) = &variable_info.expression_type {
            match &variable_info.redim_info {
                Some(redim_info) => {
                    if redim_info.dimension_count == dimension_count {
                        Ok(())
                    } else {
                        Err(QError::WrongNumberOfDimensions)
                    }
                }
                _ => Err(QError::ArrayAlreadyDimensioned),
            }
        } else {
            Err(QError::DuplicateDefinition)
        }
    }

    fn require_user_defined_array(
        variable_info: &VariableInfo,
        user_defined_type: &BareName,
    ) -> Result<(), QError> {
        if let ExpressionType::Array(element_type) = &variable_info.expression_type {
            if let ExpressionType::UserDefined(u) = element_type.as_ref() {
                if u == user_defined_type {
                    return Ok(());
                }
            }
        }
        Err(QError::DuplicateDefinition)
    }

    fn redim_user_defined_type_to_dim_type(
        ctx: &mut Context,
        bare_name: &BareName,
        array_dimensions: &ArrayDimensions,
        user_defined_type: &BareNameNode,
        pos: Location,
    ) -> Result<DimType, QErrorNode> {
        let mut visitor =
            RedimUserDefinedTypeVisitor(array_dimensions.len(), user_defined_type.as_ref());
        ctx.names.visit(&bare_name, &mut visitor).with_err_at(pos)?;
        Ok(DimType::UserDefined(user_defined_type.clone()))
    }
}

pub fn on_params(ctx: &mut Context, params: ParamNameNodes) -> Result<ParamNameNodes, QErrorNode> {
    params
        .into_iter()
        .map(|x| convert_param_name_node(ctx, x))
        .collect()
}

// TODO remove the dance between params and dim nodes
fn convert_param_name_node(
    ctx: &mut Context,
    param_name_node: ParamNameNode,
) -> Result<ParamNameNode, QErrorNode> {
    // destruct param_name_node
    let Locatable {
        element: ParamName {
            bare_name,
            param_type,
        },
        pos,
    } = param_name_node;
    // construct dim_list
    let dim_type = DimType::from(param_type);
    let dim_list: DimList = DimNameBuilder::new()
        .bare_name(bare_name)
        .dim_type(dim_type)
        .build_list(pos);
    // convert
    let (mut converted_dim_list, implicits) = on_dim(ctx, dim_list, DimContext::Param)?;
    debug_assert!(
        implicits.is_empty(),
        "Should not have introduced implicit variables via parameter"
    );
    let Locatable {
        element: DimName {
            bare_name,
            dim_type,
        },
        ..
    } = converted_dim_list
        .variables
        .pop()
        .expect("Should have one converted variable");
    let param_type = ParamType::from(dim_type);
    let param_name = ParamName::new(bare_name, param_type);
    Ok(param_name.at(pos))
}

fn no_implicits<T>(value: T) -> (T, Implicits) {
    (value, vec![])
}
