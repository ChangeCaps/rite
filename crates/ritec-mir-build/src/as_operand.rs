use ritec_mir as mir;

use crate::{thir, Builder};

impl<'a> Builder<'a> {
    pub fn as_operand(&mut self, expr: &thir::Expr) -> mir::Operand {
        let place = self.as_place(expr);
        mir::Operand::Move(place)
    }
}
