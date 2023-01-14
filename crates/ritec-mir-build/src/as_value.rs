use ritec_mir as mir;

use crate::{thir, Builder};

impl<'a> Builder<'a> {
    pub fn as_value(&mut self, expr: &thir::Expr) -> mir::Value {
        match expr {
            thir::Expr::Ref(expr) => {
                let place = self.as_place(&self.thir[expr.operand]);
                mir::Value::Address(place)
            }
            thir::Expr::Return(expr) => {
                let value = if let Some(value) = expr.value {
                    self.as_operand(&self.thir[value])
                } else {
                    mir::Operand::Void
                };

                self.terminate(mir::Terminator::Return(value));

                mir::Value::VOID
            }
            thir::Expr::Local(_) | thir::Expr::Deref(_) | thir::Expr::Assign(_) => {
                let operand = self.as_operand(expr);
                mir::Value::Use(operand)
            }
        }
    }
}
