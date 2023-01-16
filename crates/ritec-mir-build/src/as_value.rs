use ritec_core::{BinaryOp, UnaryOp};
use ritec_mir as mir;

use crate::{thir, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_value(&mut self, expr: &thir::Expr) -> mir::Value {
        match expr {
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Ref => {
                let place = self.as_place(&self.thir[expr.operand]);
                mir::Value::Address(place)
            }
            thir::Expr::Binary(expr) => {
                let lhs = self.as_operand(&self.thir[expr.lhs]);
                let rhs = self.as_operand(&self.thir[expr.rhs]);

                let op = match self.thir[expr.lhs].ty() {
                    mir::Type::Int(ref t) => match expr.operator {
                        BinaryOp::Add => mir::BinOp::IntAdd,
                        BinaryOp::Sub => mir::BinOp::IntSub,
                        BinaryOp::Mul => mir::BinOp::IntMul,
                        BinaryOp::Div if t.signed => mir::BinOp::IntDivSigned,
                        BinaryOp::Div => mir::BinOp::IntDivUnsigned,
                        BinaryOp::Eq => mir::BinOp::IntEq,
                    },
                    mir::Type::Float(_) => match expr.operator {
                        BinaryOp::Add => mir::BinOp::FloatAdd,
                        BinaryOp::Sub => mir::BinOp::FloatSub,
                        BinaryOp::Mul => mir::BinOp::FloatMul,
                        BinaryOp::Div => mir::BinOp::FloatDiv,
                        BinaryOp::Eq => mir::BinOp::FloatEq,
                    },
                    _ => unreachable!("{}", expr.ty),
                };

                mir::Value::BinaryOp(op, lhs, rhs)
            }
            thir::Expr::Call(expr) => {
                let callee = self.as_operand(&self.thir[expr.callee]);
                let mut arguments = Vec::new();
                for &argument in &expr.arguments {
                    arguments.push(self.as_operand(&self.thir[argument]));
                }

                mir::Value::Call(callee, arguments)
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
            | thir::Expr::Function(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Assign(_)
            | thir::Expr::Block(_)
            | thir::Expr::If(_) => {
                let operand = self.as_operand(expr);
                mir::Value::Use(operand)
            }
        }
    }
}
