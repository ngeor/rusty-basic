use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{
    CaseInsensitiveString, Locatable, Location, PatchErrPos, QError, QErrorNode,
    ToErrorEnvelopeNoPos, ToLocatableError,
};
use crate::linter::casting::cast;
use crate::linter::type_resolver::{ResolveInto, TypeResolver};
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::types::ResolvedTypeDefinition;
use crate::parser::{
    BareName, BareNameNode, DeclaredName, DeclaredNameNode, DeclaredNameNodes, DefType,
    ElementType, Expression, ExpressionNode, HasQualifier, Name, NameNode, Operator, ProgramNode,
    Statement, TopLevelToken, TypeDefinition, TypeQualifier, UnaryOperator, UserDefinedType,
};
use crate::variant::Variant;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct FirstPassOuter {
    inner: Rc<RefCell<FirstPassInner>>,
    function_context: FunctionContext,
    sub_context: SubContext,
    global_constants: HashMap<BareName, Variant>,
}

impl FirstPassOuter {
    pub fn new() -> Self {
        let inner = FirstPassInner::new();
        let rc = Rc::new(RefCell::new(inner));
        let fc = FunctionContext::new(Rc::downgrade(&rc));
        let sc = SubContext::new(Rc::downgrade(&rc));
        Self {
            inner: rc,
            function_context: fc,
            sub_context: sc,
            global_constants: HashMap::new(),
        }
    }

    pub fn into_inner(self) -> (FunctionMap, SubMap, HashMap<BareName, UserDefinedType>) {
        let Self {
            function_context,
            sub_context,
            inner,
            ..
        } = self;
        let i = Rc::try_unwrap(inner).unwrap().into_inner();
        (
            function_context.implementations,
            sub_context.implementations,
            i.user_defined_types,
        )
    }

    /// Collects subprograms of the given program.
    /// Ensures that:
    /// - All declared subprograms are implemented
    /// - No duplicate implementations
    /// - No conflicts between declarations and implementations
    /// - Resolves types of parameters and functions
    pub fn parse(&mut self, program: &ProgramNode) -> Result<(), QErrorNode> {
        for Locatable {
            element: top_level_token,
            pos,
        } in program
        {
            self.top_level_token(top_level_token, *pos)
                .patch_err_pos(pos)?;
        }
        self.function_context.post_visit()?;
        self.sub_context.post_visit()
    }

    fn top_level_token(
        &mut self,
        top_level_token: &TopLevelToken,
        pos: Location,
    ) -> Result<(), QErrorNode> {
        match top_level_token {
            TopLevelToken::DefType(def_type) => self.def_type(def_type),
            TopLevelToken::FunctionDeclaration(name_node, params) => {
                self.function_declaration(name_node, params, pos)
            }
            TopLevelToken::SubDeclaration(bare_name_node, params) => {
                self.sub_declaration(bare_name_node, params, pos)
            }
            TopLevelToken::FunctionImplementation(name_node, params, _) => {
                self.function_implementation(name_node, params, pos)
            }
            TopLevelToken::SubImplementation(bare_name_node, params, _) => {
                self.sub_implementation(bare_name_node, params, pos)
            }
            TopLevelToken::Statement(s) => match s {
                Statement::Const(name_node, expression_node) => {
                    self.global_const(name_node, expression_node)
                }
                _ => Ok(()),
            },
            TopLevelToken::UserDefinedType(user_defined_type) => {
                self.user_defined_type(user_defined_type)
            }
        }
    }

    // ========================================================
    // def type
    // ========================================================

    fn def_type(&mut self, def_type: &DefType) -> Result<(), QErrorNode> {
        self.inner.borrow_mut().resolver.set(def_type);
        Ok(())
    }

    // ========================================================
    // functions
    // ========================================================

    fn function_declaration(
        &mut self,
        name_node: &NameNode,
        params: &DeclaredNameNodes,
        declaration_pos: Location,
    ) -> Result<(), QErrorNode> {
        self.function_context
            .add_declaration(name_node, params, declaration_pos)
    }

    fn function_implementation(
        &mut self,
        name_node: &NameNode,
        params: &DeclaredNameNodes,
        implementation_pos: Location,
    ) -> Result<(), QErrorNode> {
        self.function_context
            .add_implementation(name_node, params, implementation_pos)
    }

    // ========================================================
    // subs
    // ========================================================

    fn sub_declaration(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &DeclaredNameNodes,
        declaration_pos: Location,
    ) -> Result<(), QErrorNode> {
        self.sub_context
            .add_declaration(bare_name_node, params, declaration_pos)
    }

    fn sub_implementation(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &DeclaredNameNodes,
        implementation_pos: Location,
    ) -> Result<(), QErrorNode> {
        self.sub_context
            .add_implementation(bare_name_node, params, implementation_pos)
    }

    // ========================================================
    // global constants
    // ========================================================

    fn global_const(
        &mut self,
        name_node: &NameNode,
        expression_node: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, pos } = name_node;
        let bare_name: &BareName = name.as_ref();
        if self.global_constants.contains_key(bare_name) {
            return Err(QError::DuplicateDefinition).with_err_at(pos);
        }
        let Locatable {
            element: expression,
            pos,
        } = expression_node;
        let v: Variant = self.resolve_const_value(expression).patch_err_pos(pos)?;
        match name {
            Name::Bare(b) => {
                self.global_constants.insert(b.clone(), v);
            }
            Name::Qualified { name, qualifier } => {
                let casted = cast(v, *qualifier).with_err_at(pos)?;
                self.global_constants.insert(name.clone(), casted);
            }
        }
        Ok(())
    }

    fn resolve_const_value(&self, expression: &Expression) -> Result<Variant, QErrorNode> {
        match expression {
            Expression::SingleLiteral(f) => Ok(Variant::VSingle(*f)),
            Expression::DoubleLiteral(d) => Ok(Variant::VDouble(*d)),
            Expression::StringLiteral(s) => Ok(Variant::VString(s.clone())),
            Expression::IntegerLiteral(i) => Ok(Variant::VInteger(*i)),
            Expression::LongLiteral(l) => Ok(Variant::VLong(*l)),
            Expression::VariableName(name) => match name {
                Name::Bare(name) => match self.global_constants.get(name) {
                    Some(v) => Ok(v.clone()),
                    None => Err(QError::InvalidConstant).with_err_no_pos(),
                },
                Name::Qualified { name, qualifier } => match self.global_constants.get(name) {
                    Some(v) => {
                        if v.qualifier() == *qualifier {
                            Ok(v.clone())
                        } else {
                            Err(QError::TypeMismatch).with_err_no_pos()
                        }
                    }
                    None => Err(QError::InvalidConstant).with_err_no_pos(),
                },
            },
            Expression::FunctionCall(_, _) => Err(QError::InvalidConstant).with_err_no_pos(),
            Expression::BinaryExpression(op, left, right) => {
                let Locatable { element, pos } = left.as_ref();
                let v_left = self.resolve_const_value(element).patch_err_pos(*pos)?;
                let Locatable { element, pos } = right.as_ref();
                let v_right = self.resolve_const_value(element).patch_err_pos(*pos)?;
                match *op {
                    Operator::Less => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Less).into())
                    }
                    Operator::LessOrEqual => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Less || order == Ordering::Equal).into())
                    }
                    Operator::Equal => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Equal).into())
                    }
                    Operator::GreaterOrEqual => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Greater || order == Ordering::Equal).into())
                    }
                    Operator::Greater => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Greater).into())
                    }
                    Operator::NotEqual => {
                        let order = v_left.cmp(&v_right).with_err_at(*pos)?;
                        Ok((order == Ordering::Less || order == Ordering::Greater).into())
                    }
                    Operator::Plus => v_left.plus(v_right).with_err_at(*pos),
                    Operator::Minus => v_left.minus(v_right).with_err_at(*pos),
                    Operator::Multiply => v_left.multiply(v_right).with_err_at(*pos),
                    Operator::Divide => v_left.divide(v_right).with_err_at(*pos),
                    Operator::And => v_left.and(v_right).with_err_at(*pos),
                    Operator::Or => v_left.or(v_right).with_err_at(*pos),
                }
            }
            Expression::UnaryExpression(op, child) => {
                let Locatable { element, pos } = child.as_ref();
                let v = self.resolve_const_value(element).patch_err_pos(*pos)?;
                match *op {
                    UnaryOperator::Minus => v.negate().with_err_at(*pos),
                    UnaryOperator::Not => v.unary_not().with_err_at(*pos),
                }
            }
            Expression::Parenthesis(child) => {
                let Locatable { element, pos } = child.as_ref();
                self.resolve_const_value(element).patch_err_pos(*pos)
            }
            Expression::FileHandle(_) => Err(QError::InvalidConstant).with_err_no_pos(),
        }
    }

    // ========================================================
    // user defined types
    // ========================================================

    fn user_defined_type(&mut self, user_defined_type: &UserDefinedType) -> Result<(), QErrorNode> {
        let type_name: &BareName = user_defined_type.name.as_ref();
        if self
            .inner
            .borrow()
            .user_defined_types
            .contains_key(type_name)
        {
            // duplicate type definition
            Err(QError::DuplicateDefinition).with_err_no_pos()
        } else {
            let mut seen_element_names: HashSet<BareName> = HashSet::new();
            for Locatable { element, pos } in user_defined_type.elements.iter() {
                let element_name: &BareName = &element.name;
                if seen_element_names.contains(element_name) {
                    // duplicate element name within type
                    return Err(QError::DuplicateDefinition).with_err_at(pos);
                } else {
                    seen_element_names.insert(element_name.clone());
                }
                match &element.element_type {
                    ElementType::String(str_len_expression_node) => {
                        self.validate_element_type_str_len(str_len_expression_node)?;
                    }
                    ElementType::UserDefined(Locatable {
                        element: referred_name,
                        pos,
                    }) => {
                        if !self
                            .inner
                            .borrow()
                            .user_defined_types
                            .contains_key(referred_name)
                        {
                            return Err(QError::syntax_error("Type not defined")).with_err_at(pos);
                        }
                    }
                    _ => {}
                }
            }
            self.inner
                .borrow_mut()
                .user_defined_types
                .insert(type_name.clone(), user_defined_type.clone());
            Ok(())
        }
    }

    fn validate_element_type_str_len(
        &self,
        str_len_expression_node: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: str_len_expression,
            pos,
        } = str_len_expression_node;
        match str_len_expression {
            Expression::IntegerLiteral(_i) => {
                // parser already covers this
                Ok(())
            }
            Expression::VariableName(v) => {
                // only constants allowed
                match v {
                    Name::Bare(b) => {
                        // bare name constant
                        match self.global_constants.get(b) {
                            // constant exists
                            Some(const_value) => {
                                match const_value {
                                    Variant::VInteger(i) => {
                                        if *i >= 1 {
                                            Ok(())
                                        } else {
                                            // illegal string length
                                            Err(QError::InvalidConstant).with_err_at(pos)
                                        }
                                    }
                                    _ => {
                                        // only integer constants allowed
                                        Err(QError::InvalidConstant).with_err_at(pos)
                                    }
                                }
                            }
                            // constant does not exist
                            None => Err(QError::InvalidConstant).with_err_at(pos),
                        }
                    }
                    Name::Qualified { name, qualifier } => {
                        match self.global_constants.get(name) {
                            // constant exists
                            Some(const_value) => {
                                match const_value {
                                    Variant::VInteger(i) => {
                                        if *qualifier == TypeQualifier::PercentInteger && *i >= 1 {
                                            Ok(())
                                        } else {
                                            // illegal string length or using wrong qualifier to reference the int constant
                                            Err(QError::InvalidConstant).with_err_at(pos)
                                        }
                                    }
                                    _ => {
                                        // only integer constants allowed
                                        Err(QError::InvalidConstant).with_err_at(pos)
                                    }
                                }
                            }
                            // constant does not exist
                            None => Err(QError::InvalidConstant).with_err_at(pos),
                        }
                    }
                }
            }
            _ => panic!("Unexpected string length {:?}", str_len_expression),
        }
    }
}

/// Inner mutable members
/// - resolver is needed to resolve function names and bare name parameters
/// - user_defined_types is needed to validate the existence of extended user defined declared parameters
#[derive(Debug)]
struct FirstPassInner {
    resolver: TypeResolverImpl,
    user_defined_types: HashMap<BareName, UserDefinedType>,
}

impl FirstPassInner {
    pub fn new() -> Self {
        Self {
            resolver: TypeResolverImpl::default(),
            user_defined_types: HashMap::new(),
        }
    }

    // ========================================================
    // resolving function/sub parameters
    // ========================================================

    pub fn parameters(
        &self,
        params: &DeclaredNameNodes,
    ) -> Result<Vec<ResolvedTypeDefinition>, QErrorNode> {
        params.iter().map(|p| self.parameter(p)).collect()
    }

    fn parameter(&self, param: &DeclaredNameNode) -> Result<ResolvedTypeDefinition, QErrorNode> {
        let Locatable {
            element: declared_name,
            pos,
        } = param;
        let DeclaredName {
            name,
            type_definition,
        } = declared_name;
        match type_definition {
            TypeDefinition::Bare => {
                let q: TypeQualifier = self.resolver.resolve(name);
                Ok(ResolvedTypeDefinition::CompactBuiltIn(q))
            }
            TypeDefinition::CompactBuiltIn(q) => Ok(ResolvedTypeDefinition::CompactBuiltIn(*q)),
            TypeDefinition::ExtendedBuiltIn(q) => Ok(ResolvedTypeDefinition::ExtendedBuiltIn(*q)),
            TypeDefinition::UserDefined(u) => {
                if self.user_defined_types.contains_key(u) {
                    Ok(ResolvedTypeDefinition::UserDefined(u.clone()))
                } else {
                    Err(QError::syntax_error("Type not defined")).with_err_at(pos)
                }
            }
        }
    }
}

pub type ParamTypes = Vec<ResolvedTypeDefinition>;
pub type FunctionMap = HashMap<CaseInsensitiveString, (TypeQualifier, ParamTypes, Location)>;

#[derive(Debug)]
struct FunctionContext {
    declarations: FunctionMap,
    implementations: FunctionMap,
    first_pass: Weak<RefCell<FirstPassInner>>,
}

impl FunctionContext {
    pub fn new(first_pass: Weak<RefCell<FirstPassInner>>) -> Self {
        Self {
            declarations: FunctionMap::new(),
            implementations: FunctionMap::new(),
            first_pass,
        }
    }

    pub fn add_declaration(
        &mut self,
        name_node: &NameNode,
        params: &DeclaredNameNodes,
        declaration_pos: Location,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let first_pass = self.first_pass.upgrade().unwrap();
        let q_params: ParamTypes = first_pass.borrow().parameters(params)?;
        let q_name: TypeQualifier = name.resolve_into(&first_pass.borrow().resolver);
        let bare_name: &BareName = name.as_ref();
        self.check_implementation_type(bare_name, &q_name, &q_params)?;
        match self.declarations.get(bare_name) {
            Some(_) => self
                .check_declaration_type(bare_name, &q_name, &q_params)
                .with_err_no_pos(),
            None => {
                self.declarations
                    .insert(bare_name.clone(), (q_name, q_params, declaration_pos));
                Ok(())
            }
        }
    }

    pub fn add_implementation(
        &mut self,
        name_node: &NameNode,
        params: &DeclaredNameNodes,
        implementation_pos: Location,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = name_node;

        // type must match declaration
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let first_pass = self.first_pass.upgrade().unwrap();
        let q_params: ParamTypes = first_pass.borrow().parameters(params)?;
        let q_name: TypeQualifier = name.resolve_into(&first_pass.borrow().resolver);
        let bare_name: &BareName = name.as_ref();
        match self.implementations.get(bare_name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_no_pos(),
            None => {
                self.check_declaration_type(bare_name, &q_name, &q_params)
                    .with_err_no_pos()?;
                self.implementations
                    .insert(bare_name.clone(), (q_name, q_params, implementation_pos));
                Ok(())
            }
        }
    }

    fn check_declaration_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_name: &TypeQualifier,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.declarations.get(name) {
            Some((e_name, e_params, _)) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    fn check_implementation_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_name: &TypeQualifier,
        q_params: &ParamTypes,
    ) -> Result<(), QErrorNode> {
        match self.implementations.get(name) {
            Some((e_name, e_params, _)) => {
                if e_name != q_name || e_params != q_params {
                    return Err(QError::TypeMismatch).with_err_no_pos();
                }
            }
            None => (),
        }
        Ok(())
    }

    pub fn post_visit(&self) -> Result<(), QErrorNode> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v.2);
            }
        }

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInFunction> = k.into();
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v.2);
            }
        }

        Ok(())
    }
}

pub type SubMap = HashMap<CaseInsensitiveString, (ParamTypes, Location)>;

#[derive(Debug)]
struct SubContext {
    declarations: SubMap,
    implementations: SubMap,
    first_pass: Weak<RefCell<FirstPassInner>>,
}

impl SubContext {
    pub fn new(first_pass: Weak<RefCell<FirstPassInner>>) -> Self {
        Self {
            declarations: SubMap::new(),
            implementations: SubMap::new(),
            first_pass,
        }
    }

    pub fn add_declaration(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &DeclaredNameNodes,
        declaration_pos: Location,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = bare_name_node;
        // name does not have to be unique (duplicate identical declarations okay)
        // conflicting declarations to previous declaration or implementation not okay
        let first_pass = self.first_pass.upgrade().unwrap();
        let q_params: ParamTypes = first_pass.borrow().parameters(params)?;
        self.check_implementation_type(name, &q_params)
            .with_err_no_pos()?;
        match self.declarations.get(name) {
            Some(_) => self
                .check_declaration_type(name, &q_params)
                .with_err_no_pos(),
            None => {
                self.declarations
                    .insert(name.clone(), (q_params, declaration_pos));
                Ok(())
            }
        }
    }

    pub fn add_implementation(
        &mut self,
        bare_name_node: &BareNameNode,
        params: &DeclaredNameNodes,
        implementation_pos: Location,
    ) -> Result<(), QErrorNode> {
        let Locatable { element: name, .. } = bare_name_node;
        // param count must match declaration
        // param types must match declaration
        // name needs to be unique
        let first_pass = self.first_pass.upgrade().unwrap();
        let q_params: ParamTypes = first_pass.borrow().parameters(params)?;
        match self.implementations.get(name) {
            Some(_) => Err(QError::DuplicateDefinition).with_err_no_pos(),
            None => {
                self.check_declaration_type(name, &q_params)
                    .with_err_no_pos()?;
                self.implementations
                    .insert(name.clone(), (q_params, implementation_pos));
                Ok(())
            }
        }
    }

    fn check_declaration_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.declarations.get(name) {
            Some((e_params, _)) => {
                if e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    fn check_implementation_type(
        &mut self,
        name: &CaseInsensitiveString,
        q_params: &ParamTypes,
    ) -> Result<(), QError> {
        match self.implementations.get(name) {
            Some((e_params, _)) => {
                if e_params != q_params {
                    return Err(QError::TypeMismatch);
                }
            }
            None => (),
        }
        Ok(())
    }

    pub fn post_visit(&self) -> Result<(), QErrorNode> {
        for (k, v) in self.declarations.iter() {
            if !self.implementations.contains_key(k) {
                return Err(QError::SubprogramNotDefined).with_err_at(v.1);
            }
        }

        for (k, v) in self.implementations.iter() {
            let opt_built_in: Option<BuiltInSub> = k.into();
            if opt_built_in.is_some() {
                return Err(QError::DuplicateDefinition).with_err_at(v.1);
            }
        }

        Ok(())
    }
}
