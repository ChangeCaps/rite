use ritec_core::UnaryOp;
use ritec_mir as mir;

use crate::{thir, unpack, BlockAnd, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_place(&mut self, mut block: mir::BlockId, expr: &thir::Expr) -> BlockAnd<mir::Place> {
        match expr {
            thir::Expr::Local(expr) => {
                let place = mir::Place {
                    local: expr.local,
                    proj: vec![],
                };

                BlockAnd::new(block, place)
            }
            thir::Expr::Init(expr) => {
                let place = self.push_temp(expr.ty.clone());

                for (field, init) in expr.fields.iter() {
                    let mut place = place.clone();
                    place.proj.push(mir::Projection::Field(*field));

                    let value = unpack!(block = self.as_value(block, &self.thir[*init]));
                    self[block].push_assign(place, value);
                }

                BlockAnd::new(block, place)
            }
            thir::Expr::Field(expr) => {
                let mut place = unpack!(block = self.as_place(block, &self.thir[expr.class]));
                place.proj.push(mir::Projection::Field(expr.field));

                BlockAnd::new(block, place)
            }
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Deref => {
                let mut place = unpack!(block = self.as_place(block, &self.thir[expr.operand]));
                place.proj.push(mir::Projection::Deref);
                BlockAnd::new(block, place)
            }
            thir::Expr::Assign(expr) => {
                let temp = self.push_temp(expr.ty.clone());
                let place = unpack!(block = self.as_place(block, &self.thir[expr.lhs]));
                let value = unpack!(block = self.as_operand(block, &self.thir[expr.rhs]));

                self[block].push_assign(temp.clone(), mir::Operand::Move(place.clone()));
                self[block].push_assign(place, value);

                BlockAnd::new(block, temp)
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
                let value = unpack!(block = self.as_value(block, expr));
                let temp = self.push_temp(expr.ty().clone());
                self[block].push_assign(temp.clone(), value);

                BlockAnd::new(block, temp)
            }
        }
    }
}
