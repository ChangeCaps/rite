use ritec_core::Literal;
use ritec_mir as mir;

use crate::{thir, unpack, BlockAnd, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_operand(
        &mut self,
        mut block: mir::BlockId,
        expr: &thir::Expr,
    ) -> BlockAnd<mir::Operand> {
        if self[block].is_terminated() {
            return BlockAnd::new(block, mir::Operand::VOID);
        }

        match expr {
            thir::Expr::Literal(expr) => match &expr.literal {
                Literal::Null(_) => {
                    let mir::Type::Pointer(ref ty) = expr.ty else {
                        unreachable!("expected pointer type");
                    };

                    let null = mir::Constant::Null(ty.pointee().clone());
                    let value = mir::Operand::Constant(null);
                    BlockAnd::new(block, value)
                }
                Literal::Bool(lit) => {
                    let constant = mir::Operand::Constant(mir::Constant::Bool(lit.value));
                    BlockAnd::new(block, constant)
                }
                Literal::Int(lit) => {
                    let mir::Type::Int(ref ty) = expr.ty else {
                        unreachable!("{}", expr.ty);
                    };

                    let constant = mir::Operand::Constant(mir::Constant::Integer(
                        lit.value as i64,
                        ty.clone(),
                    ));
                    BlockAnd::new(block, constant)
                }
                Literal::Float(lit) => {
                    let mir::Type::Float(ref ty) = expr.ty else {
                        unreachable!("{}", expr.ty)
                    };

                    let constant =
                        mir::Operand::Constant(mir::Constant::Float(lit.value, ty.clone()));
                    BlockAnd::new(block, constant)
                }
            },
            thir::Expr::Function(expr) => {
                let constant = mir::Operand::Constant(mir::Constant::Function(
                    expr.function.cast(),
                    expr.generics.clone(),
                ));

                BlockAnd::new(block, constant)
            }
            thir::Expr::Return(expr) => {
                let value = if let Some(value) = expr.value {
                    unpack!(block = self.as_operand(block, &self.thir[value]))
                } else {
                    mir::Operand::Constant(mir::Constant::Void)
                };

                self[block].terminate_return(value);

                BlockAnd::new(block, mir::Operand::VOID)
            }
            thir::Expr::Break(_) => {
                let break_block = self.break_block.expect("break outside of loop");
                self[block].terminate_goto(break_block);

                BlockAnd::new(block, mir::Operand::VOID)
            }
            thir::Expr::Block(expr) => {
                block = self.build_block(block, &self.thir[expr.block]);
                BlockAnd::new(block, mir::Operand::VOID)
            }
            thir::Expr::If(expr) => self.build_if_expr(block, expr),
            thir::Expr::Loop(expr) => {
                let loop_block = self.new_block();
                let exit_block = self.new_block();

                self[block].terminate_goto(loop_block);

                let break_block = self.break_block.take();
                self.break_block = Some(exit_block);

                let block = self.build_block(loop_block, &self.thir[expr.block]);
                self[block].terminate_goto(loop_block);

                self.break_block = break_block;

                BlockAnd::new(exit_block, mir::Operand::VOID)
            }
            thir::Expr::Local(_)
            | thir::Expr::ClassInit(_)
            | thir::Expr::Field(_)
            | thir::Expr::As(_)
            | thir::Expr::Bitcast(_)
            | thir::Expr::Sizeof(_)
            | thir::Expr::Alignof(_)
            | thir::Expr::Malloc(_)
            | thir::Expr::Free(_)
            | thir::Expr::Memcpy(_)
            | thir::Expr::Call(_)
            | thir::Expr::StaticCall(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Binary(_)
            | thir::Expr::Assign(_) => {
                let place = unpack!(block = self.as_place(block, expr));
                BlockAnd::new(block, mir::Operand::Move(place))
            }
        }
    }

    pub fn build_if_expr(
        &mut self,
        mut block: mir::BlockId,
        expr: &thir::IfExpr,
    ) -> BlockAnd<mir::Operand> {
        let condition = unpack!(block = self.as_operand(block, &self.thir[expr.condition]));

        if self[block].is_terminated() {
            return BlockAnd::new(block, mir::Operand::VOID);
        }

        if let Some(else_expr) = expr.else_expr {
            return self.build_if_else_expr(block, condition, expr.then_expr, else_expr);
        }

        let then_block = self.new_block();
        self.as_value(then_block, &self.thir[expr.then_expr]);

        let end_block = self.new_block();
        self[then_block].terminate_goto(end_block);

        let targets = mir::SwitchTargets {
            targets: vec![(1, then_block)],
            default: end_block,
        };

        self[block].terminate_switch(condition, targets);

        BlockAnd::new(end_block, mir::Operand::VOID)
    }

    pub fn build_if_else_expr(
        &mut self,
        block: mir::BlockId,
        condition: mir::Operand,
        then_expr: thir::ExprId,
        else_expr: thir::ExprId,
    ) -> BlockAnd<mir::Operand> {
        let then_block = self.new_block();
        self.as_value(then_block, &self.thir[then_expr]);

        let else_block = self.new_block();
        self.as_value(else_block, &self.thir[else_expr]);

        let targets = mir::SwitchTargets {
            targets: vec![(1, then_block)],
            default: else_block,
        };

        self[block].terminate_switch(condition, targets);

        if self[then_block].is_terminated() && self[else_block].is_terminated() {
            return BlockAnd::new(block, mir::Operand::VOID);
        }

        let end_block = self.new_block();
        self[then_block].terminate_goto(end_block);
        self[else_block].terminate_goto(end_block);

        BlockAnd::new(end_block, mir::Operand::VOID)
    }
}
