use ritec_core::Literal;
use ritec_mir as mir;

use crate::{thir, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_operand(&mut self, expr: &thir::Expr) -> mir::Operand {
        match expr {
            thir::Expr::Literal(expr) => match &expr.literal {
                Literal::Bool(lit) => mir::Operand::Constant(mir::Constant::Integer(
                    lit.value as i64,
                    mir::IntType::I8,
                )),
                Literal::Int(lit) => {
                    let mir::Type::Int(ty) = &expr.ty else {
                        unreachable!()
                    };

                    mir::Operand::Constant(mir::Constant::Integer(lit.value as i64, ty.clone()))
                }
                Literal::Float(lit) => {
                    let mir::Type::Float(ty) = &expr.ty else {
                        unreachable!()
                    };

                    mir::Operand::Constant(mir::Constant::Float(lit.value, ty.clone()))
                }
            },
            thir::Expr::Function(expr) => mir::Operand::Constant(mir::Constant::Function(
                expr.function.cast(),
                expr.generics.clone(),
            )),
            thir::Expr::Local(_)
            | thir::Expr::Call(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Binary(_)
            | thir::Expr::Assign(_)
            | thir::Expr::Return(_) => {
                let place = self.as_place(expr);
                mir::Operand::Move(place)
            }
        }
    }
}
