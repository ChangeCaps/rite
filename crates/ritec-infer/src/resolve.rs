use ritec_hir as hir;
use ritec_mir as mir;

use crate::{Error, InferType, InferenceTable, ItemId};

impl InferenceTable {
    pub fn resolve_mir(&self, id: hir::HirId) -> Result<mir::Type, Error> {
        let Some(ty) = self.get_type(id) else {
            unreachable!("{:?} not registered", id);
        };

        self.resolve_mir_type(ty)
    }

    pub fn resolve_mir_type(&self, ty: &InferType) -> Result<mir::Type, Error> {
        match ty {
            InferType::Var(var) => {
                if let Some(ty) = self.get_substitution(var) {
                    self.resolve_mir_type(&ty)
                } else {
                    Err(Error::AmbiguousType(var.clone()))
                }
            }
            InferType::Apply(apply) => {
                let mut args = Vec::new();
                for arg in &apply.arguments {
                    args.push(self.resolve_mir_type(arg)?);
                }

                Ok(self.resolve_mir_apply(&apply.item, args))
            }
            InferType::Proj(_) => todo!(),
        }
    }

    fn resolve_mir_apply(&self, item: &ItemId, mut args: Vec<mir::Type>) -> mir::Type {
        match item {
            ItemId::Void => mir::Type::Void,
            ItemId::Bool => mir::Type::Bool,
            ItemId::Int(ty) => mir::Type::Int(mir::IntType {
                signed: ty.signed,
                size: ty.size,
            }),
            ItemId::Float(ty) => mir::Type::Float(mir::FloatType { size: ty.size }),
            ItemId::Pointer => mir::Type::pointer(args.pop().unwrap()),
            ItemId::Array(size) => mir::Type::array(args.pop().unwrap(), *size),
            ItemId::Slice => mir::Type::slice(args.pop().unwrap()),
            ItemId::Function => {
                let return_type = args.pop().unwrap();
                mir::Type::function(args, return_type)
            }
            ItemId::Tuple => mir::Type::tuple(args),
            ItemId::Generic(generic) => mir::Type::Generic(generic.clone()),
        }
    }
}