use ritec_hir as hir;

use crate::{InferType, InferenceTable, ItemId};

impl InferenceTable {
    pub fn register_hir(&mut self, id: hir::HirId, hir: &hir::Type) -> InferType {
        let ty = self.infer_hir(hir);
        self.register_type(id, ty.clone());
        ty
    }

    pub fn infer_hir(&mut self, hir: &hir::Type) -> InferType {
        match hir {
            hir::Type::Inferred(_) => self.new_variable().into(),
            hir::Type::Void(_) => InferType::apply(ItemId::Void, [], hir.span()),
            hir::Type::Bool(_) => InferType::apply(ItemId::Bool, [], hir.span()),
            hir::Type::Int(ty) => InferType::apply(ItemId::Int(ty.clone()), [], ty.span),
            hir::Type::Float(ty) => InferType::apply(ItemId::Float(ty.clone()), [], ty.span),
            hir::Type::Pointer(ty) => {
                let pointee = self.infer_hir(&ty.pointee);
                InferType::apply(ItemId::Pointer, [pointee], ty.span)
            }
            hir::Type::Array(ty) => {
                let element = self.infer_hir(&ty.element);
                InferType::apply(ItemId::Array(ty.size), [element], ty.span)
            }
            hir::Type::Slice(ty) => {
                let element = self.infer_hir(&ty.element);
                InferType::apply(ItemId::Slice, [element], ty.span)
            }
            hir::Type::Function(ty) => {
                let mut arguments = Vec::new();

                arguments.push(self.infer_hir(&ty.return_type));

                for argument in &ty.arguments {
                    arguments.push(self.infer_hir(&argument));
                }

                InferType::apply(ItemId::Function, arguments, ty.span)
            }
            hir::Type::Tuple(ty) => {
                let mut fields = Vec::new();

                for field in &ty.fields {
                    fields.push(self.infer_hir(&field));
                }

                InferType::apply(ItemId::Tuple, fields, ty.span)
            }
            hir::Type::Generic(generic) => {
                InferType::apply(ItemId::Generic(generic.clone()), [], generic.span())
            }
        }
    }
}
