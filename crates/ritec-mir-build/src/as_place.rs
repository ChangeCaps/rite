use ritec_mir as mir;

use crate::{thir, Builder};

impl<'a> Builder<'a> {
    pub fn as_place(&mut self, expr: &thir::Expr) -> mir::Place {
        match expr {
            thir::Expr::Local(expr) => mir::Place {
                local: expr.local,
                proj: vec![],
            },
            thir::Expr::Deref(expr) => {
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
            thir::Expr::Ref(_) | thir::Expr::Return(_) => {
                let value = self.as_value(expr);
                let temp = self.push_temp(expr.ty().clone());
                self.push_assign(temp.clone(), value);

                temp
            }
        }
    }
}
