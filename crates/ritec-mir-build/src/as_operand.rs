use ritec_core::Literal;
use ritec_mir as mir;

use crate::{thir, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_operand(&mut self, expr: &thir::Expr) -> mir::Operand {
        match expr {
            thir::Expr::Literal(expr) => match &expr.literal {
                Literal::Bool(lit) => mir::Operand::Constant(mir::Constant::Bool(lit.value)),
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
            thir::Expr::Block(expr) => {
                self.build_block(&self.thir[expr.block]);
                mir::Operand::VOID
            }
            thir::Expr::If(expr) => {
                self.build_if_expr(expr);
                mir::Operand::VOID
            }
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

    pub fn build_if_expr(&mut self, expr: &thir::IfExpr) {
        let condition = self.as_operand(&self.thir[expr.condition]);

        if let Some(else_block) = expr.else_block {
            self.build_if_else_expr(condition, expr.then_block, else_block);
            return;
        }

        let current_block = self.current_block();

        let then_block = self.build_block(&self.thir[expr.then_block]);
        let then_end = self.current_block();

        let end_block = self.push_block();

        let targets = mir::SwitchTargets {
            targets: vec![(1, then_block)],
            default: end_block,
        };

        self[current_block].terminate(mir::Terminator::Switch(condition, targets));

        if !self[then_end].is_terminated() {
            self[then_end].terminate(mir::Terminator::Goto(end_block));
        }
    }

    pub fn build_if_else_expr(
        &mut self,
        condition: mir::Operand,
        then_block: thir::BlockId,
        else_expr: thir::ExprId,
    ) {
        let current_block = self.current_block();

        let then_block = self.build_block(&self.thir[then_block]);
        let then_end = self.current_block();
        let then_terminated = self.is_terminated();

        let else_block = self.next_block();
        self.as_operand(&self.thir[else_expr]);
        let else_end = self.current_block();
        let else_terminated = self.is_terminated();

        let targets = mir::SwitchTargets {
            targets: vec![(1, then_block)],
            default: else_block,
        };

        self[current_block].terminate(mir::Terminator::Switch(condition, targets));

        if then_terminated && else_terminated {
            return;
        }

        let end_block = self.push_block();

        if !self[then_end].is_terminated() {
            self[then_end].terminate(mir::Terminator::Goto(end_block));
        }

        if !self[else_end].is_terminated() {
            self[else_end].terminate(mir::Terminator::Goto(end_block));
        }
    }
}
