use ritec_hir as hir;
use ritec_mir as mir;

use crate::{
    Constraint, InferError, InferType, InferenceTable, ItemId, Solution, TypeVariable, Unify,
};

pub struct Solver<'a> {
    program: &'a mir::Program,
    table: InferenceTable,
    constraints: Vec<Constraint>,
    stack: Vec<Constraint>,
    overflow_depth: usize,
}

impl<'a> Solver<'a> {
    pub fn new(program: &'a mir::Program) -> Self {
        Self {
            program,
            table: InferenceTable::new(),
            constraints: Vec::new(),
            stack: Vec::new(),
            overflow_depth: 256,
        }
    }

    pub fn table(&self) -> &InferenceTable {
        &self.table
    }

    pub fn new_variable(&mut self) -> TypeVariable {
        self.table.new_variable()
    }

    pub fn finish(self) -> InferenceTable {
        self.table
    }

    pub fn get_substitution(&self, var: &TypeVariable) -> Option<InferType> {
        self.table.get_substitution(var)
    }

    fn unify(&mut self, a: &InferType, b: &InferType) -> Result<Solution, InferError> {
        let result = self.table.unify(a, b)?;
        self.constraints.extend(result.constraints);
        Ok(Solution {
            is_solved: true,
            constraint: Constraint::Unify(Unify::new(a.clone(), b.clone())),
        })
    }

    fn solve_one(
        &mut self,
        constraint: &Constraint,
        progress: &mut bool,
    ) -> Result<(), InferError> {
        let solution = self.solve(constraint.clone())?;

        todo!()
    }

    pub fn solve(&mut self, constraint: impl Into<Constraint>) -> Result<Solution, InferError> {
        let constraint = constraint.into();

        if self.stack.contains(&constraint) || self.stack.len() > self.overflow_depth {
            return Ok(Solution {
                is_solved: false,
                constraint,
            });
        }

        self.stack.push(constraint.clone());

        let result = match constraint {
            Constraint::Unify(unify) => self.unify(&unify.a, &unify.b),
            Constraint::Normalize(_) => todo!(),
        };

        self.stack.pop().unwrap();

        result
    }

    pub fn infer_type(&mut self, ty: &hir::Type) -> InferType {
        match ty {
            hir::Type::Inferred(ty) => self.infer_inferred_type(ty),
            hir::Type::Void(ty) => self.infer_void_type(ty),
            hir::Type::Bool(ty) => self.infer_bool_type(ty),
            hir::Type::Int(ty) => self.infer_int_type(ty),
            hir::Type::Float(ty) => self.infer_float_type(ty),
            hir::Type::Pointer(ty) => self.infer_pointer_type(ty),
            hir::Type::Array(ty) => self.infer_array_type(ty),
            hir::Type::Slice(ty) => self.infer_slice_type(ty),
            hir::Type::Function(ty) => self.infer_function_type(ty),
            hir::Type::Tuple(ty) => self.infer_tuple_type(ty),
            hir::Type::Generic(ty) => self.infer_generic_type(ty),
        }
    }

    pub fn infer_inferred_type(&mut self, _: &hir::InferredType) -> InferType {
        let var = self.new_variable();
        InferType::Var(var)
    }

    pub fn infer_void_type(&mut self, ty: &hir::VoidType) -> InferType {
        InferType::apply(ItemId::Void, [], ty.span)
    }

    pub fn infer_bool_type(&mut self, ty: &hir::BoolType) -> InferType {
        InferType::apply(ItemId::Bool, [], ty.span)
    }

    pub fn infer_int_type(&mut self, ty: &hir::IntType) -> InferType {
        let item = ItemId::Int(mir::IntType {
            signed: ty.signed,
            size: ty.size,
        });
        InferType::apply(item, [], ty.span)
    }

    pub fn infer_float_type(&mut self, ty: &hir::FloatType) -> InferType {
        let item = ItemId::Float(mir::FloatType { size: ty.size });
        InferType::apply(item, [], ty.span)
    }

    pub fn infer_pointer_type(&mut self, ty: &hir::PointerType) -> InferType {
        let pointee = self.infer_type(&ty.pointee);
        InferType::apply(ItemId::Pointer, [pointee], ty.span)
    }

    pub fn infer_array_type(&mut self, ty: &hir::ArrayType) -> InferType {
        let element = self.infer_type(&ty.element);
        InferType::apply(ItemId::Array(ty.size), [element], ty.span)
    }

    pub fn infer_slice_type(&mut self, ty: &hir::SliceType) -> InferType {
        let element = self.infer_type(&ty.element);
        InferType::apply(ItemId::Slice, [element], ty.span)
    }

    pub fn infer_function_type(&mut self, ty: &hir::FunctionType) -> InferType {
        let mut arguments = Vec::with_capacity(ty.arguments.len());
        arguments.push(self.infer_type(&ty.return_type));

        for argument in &ty.arguments {
            arguments.push(self.infer_type(&argument));
        }

        InferType::apply(ItemId::Function, arguments, ty.span)
    }

    pub fn infer_tuple_type(&mut self, ty: &hir::TupleType) -> InferType {
        let mut arguments = Vec::with_capacity(ty.fields.len());

        for element in &ty.fields {
            arguments.push(self.infer_type(element));
        }

        InferType::apply(ItemId::Tuple, arguments, ty.span)
    }

    pub fn infer_generic_type(&mut self, ty: &hir::GenericType) -> InferType {
        InferType::apply(ItemId::Generic(ty.generic.clone()), [], ty.span)
    }
}
