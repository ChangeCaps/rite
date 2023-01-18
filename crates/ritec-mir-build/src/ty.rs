use ritec_hir as hir;
use ritec_mir as mir;

pub fn build_type(ty: &hir::Type) -> mir::Type {
    match ty {
        hir::Type::Inferred(_) => unreachable!("inferred type in illegal position"),
        hir::Type::Void(_) => mir::Type::Void,
        hir::Type::Bool(_) => mir::Type::Bool,
        hir::Type::Int(ty) => mir::Type::Int(mir::IntType {
            signed: ty.signed,
            size: ty.size,
        }),
        hir::Type::Float(ty) => mir::Type::Float(mir::FloatType { size: ty.size }),
        hir::Type::Pointer(ty) => mir::Type::pointer(build_type(&ty.pointee)),
        hir::Type::Array(ty) => mir::Type::array(build_type(&ty.element), ty.size),
        hir::Type::Slice(ty) => mir::Type::slice(build_type(&ty.element)),
        hir::Type::Function(ty) => {
            let mut arguments = Vec::new();
            for argument in ty.arguments.iter() {
                arguments.push(build_type(argument));
            }

            mir::Type::function(arguments, build_type(&ty.return_type))
        }
        hir::Type::Tuple(ty) => {
            let mut fields = Vec::new();
            for field in ty.fields.iter() {
                fields.push(build_type(field));
            }

            mir::Type::tuple(fields)
        }
        hir::Type::Class(ty) => {
            let mut arguments = Vec::new();
            for argument in ty.generics.iter() {
                arguments.push(build_type(argument));
            }

            mir::Type::class(ty.class.cast(), ty.ident.clone(), arguments)
        }
        hir::Type::Generic(generic) => mir::Type::Generic(generic.clone()),
    }
}
