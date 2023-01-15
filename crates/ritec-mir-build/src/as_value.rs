use ritec_core::UnaryOp;
use ritec_mir as mir;

use crate::{thir, Builder};

impl<'a> Builder<'a> {
    pub fn as_value(&mut self, expr: &thir::Expr) -> mir::Value {
        match expr {
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Ref => {
                let place = self.as_place(&self.thir[expr.operand]);
                mir::Value::Address(place)
            }
            thir::Expr::Binary(expr) => {
                let binary = mir::BinaryOpValue {
                    op: expr.operator.clone(),
                    lhs: self.as_operand(&self.thir[expr.lhs]),
                    rhs: self.as_operand(&self.thir[expr.rhs]),
                };

                mir::Value::BinaryOp(binary)
            }
            thir::Expr::Return(expr) => {
                let value = if let Some(value) = expr.value {
                    self.as_operand(&self.thir[value])
                } else {
                    mir::Operand::Constant(mir::Constant::Void)
                };

                self.terminate(mir::Terminator::Return(value));

                mir::Value::VOID
            }
            thir::Expr::Local(_)
            | thir::Expr::Literal(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Assign(_) => {
                let operand = self.as_operand(expr);
                mir::Value::Use(operand)
            }
        }
    }
}
