use ritec_hir as hir;
use ritec_mir as mir;

use crate::{InferType, InferenceTable, Instance, ItemId};

impl InferenceTable {
    pub fn register_hir(&mut self, id: hir::HirId, hir: &hir::Type) -> InferType {
        if let Some(ty) = self.get_type(id) {
            return ty.clone();
        }

        let ty = self.infer_hir(hir, &Instance::empty());
        self.register_type(id, ty.clone());
        ty
    }

    pub fn infer_hir(&mut self, hir: &hir::Type, instance: &Instance) -> InferType {
        match hir {
            hir::Type::Inferred(_) => self.new_variable(None).into(),
            hir::Type::Void(_) => InferType::apply(ItemId::Void, [], hir.span()),
            hir::Type::Bool(_) => InferType::apply(ItemId::Bool, [], hir.span()),
            hir::Type::Int(ty) => {
                let ty = mir::IntType {
                    signed: ty.signed,
                    size: ty.size,
                };

                InferType::apply(ItemId::Int(ty.clone()), [], hir.span())
            }
            hir::Type::Float(ty) => {
                let ty = mir::FloatType { size: ty.size };
                InferType::apply(ItemId::Float(ty.clone()), [], hir.span())
            }
            hir::Type::Pointer(ty) => {
                let pointee = self.infer_hir(&ty.pointee, instance);
                InferType::apply(ItemId::Pointer, [pointee], ty.span)
            }
            hir::Type::Array(ty) => {
                let element = self.infer_hir(&ty.element, instance);
                InferType::apply(ItemId::Array(ty.size), [element], ty.span)
            }
            hir::Type::Slice(ty) => {
                let element = self.infer_hir(&ty.element, instance);
                InferType::apply(ItemId::Slice, [element], ty.span)
            }
            hir::Type::Function(ty) => {
                let mut arguments = Vec::new();

                for argument in &ty.arguments {
                    arguments.push(self.infer_hir(&argument, instance));
                }

                arguments.push(self.infer_hir(&ty.return_type, instance));

                InferType::apply(ItemId::Function, arguments, ty.span)
            }
            hir::Type::Tuple(ty) => {
                let mut fields = Vec::new();

                for field in &ty.fields {
                    fields.push(self.infer_hir(&field, instance));
                }

                InferType::apply(ItemId::Tuple, fields, ty.span)
            }
            hir::Type::Generic(generic) => {
                if let Some(ty) = instance.get(generic) {
                    ty.clone()
                } else {
                    InferType::apply(ItemId::Generic(generic.clone()), [], generic.span())
                }
            }
        }
    }
}
