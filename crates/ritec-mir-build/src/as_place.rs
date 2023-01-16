use ritec_core::UnaryOp;
use ritec_mir as mir;

use crate::{thir, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_place(&mut self, expr: &thir::Expr) -> mir::Place {
        match expr {
            thir::Expr::Local(expr) => mir::Place {
                local: expr.local,
                proj: vec![],
            },
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Deref => {
                let mut place = self.as_place(&self.thir[expr.operand]);
                place.proj.push(mir::Projection::Deref);
                place
            }
            thir::Expr::Assign(expr) => {
                let temp = self.push_temp(expr.ty.clone());
                let place = self.as_place(&self.thir[expr.lhs]);
                let value = self.as_operand(&self.thir[expr.rhs]);

                self.push_assign(temp.clone(), mir::Operand::Move(place.clone()));
                self.push_assign(place, value);

                temp
            }
            thir::Expr::Literal(_)
            | thir::Expr::Function(_)
            | thir::Expr::Bitcast(_)
            | thir::Expr::Call(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Binary(_)
            | thir::Expr::Return(_)
            | thir::Expr::Break(_)
            | thir::Expr::Block(_)
            | thir::Expr::If(_)
            | thir::Expr::Loop(_) => {
                let value = self.as_value(expr);
                let temp = self.push_temp(expr.ty().clone());
                self.push_assign(temp.clone(), value);

                temp
            }
        }
    }
}
