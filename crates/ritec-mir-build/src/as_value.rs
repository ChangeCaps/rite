use ritec_core::{BinOp, UnaryOp};
use ritec_mir as mir;

use crate::{thir, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_value(&mut self, expr: &thir::Expr) -> mir::Value {
        match expr {
            thir::Expr::Bitcast(expr) => {
                let value = self.as_operand(&self.thir[expr.expr]);
                mir::Value::Cast(mir::Cast::Bit(expr.ty.clone()), value)
            }
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Ref => {
                let place = self.as_place(&self.thir[expr.operand]);
                mir::Value::Address(place)
            }
            thir::Expr::Binary(expr) => {
                let lhs = self.as_operand(&self.thir[expr.lhs]);
                let rhs = self.as_operand(&self.thir[expr.rhs]);

                let op = match self.thir[expr.lhs].ty() {
                    mir::Type::Int(ref t) => match expr.operator {
                        BinOp::Add => mir::BinOp::IntAdd,
                        BinOp::Sub => mir::BinOp::IntSub,
                        BinOp::Mul => mir::BinOp::IntMul,
                        BinOp::Div if t.signed => mir::BinOp::IntDivSigned,
                        BinOp::Div => mir::BinOp::IntDivUnsigned,
                        BinOp::Eq => mir::BinOp::IntEq,
                        BinOp::Ne => mir::BinOp::IntNe,
                        BinOp::Lt if t.signed => mir::BinOp::IntLtSigned,
                        BinOp::Lt => mir::BinOp::IntLtUnsigned,
                        BinOp::Le if t.signed => mir::BinOp::IntLeSigned,
                        BinOp::Le => mir::BinOp::IntLeUnsigned,
                        BinOp::Gt if t.signed => mir::BinOp::IntGtSigned,
                        BinOp::Gt => mir::BinOp::IntGtUnsigned,
                        BinOp::Ge if t.signed => mir::BinOp::IntGeSigned,
                        BinOp::Ge => mir::BinOp::IntGeUnsigned,
                    },
                    mir::Type::Float(_) => match expr.operator {
                        BinOp::Add => mir::BinOp::FloatAdd,
                        BinOp::Sub => mir::BinOp::FloatSub,
                        BinOp::Mul => mir::BinOp::FloatMul,
                        BinOp::Div => mir::BinOp::FloatDiv,
                        BinOp::Eq => mir::BinOp::FloatEq,
                        BinOp::Ne => mir::BinOp::FloatNe,
                        BinOp::Lt => mir::BinOp::FloatLt,
                        BinOp::Le => mir::BinOp::FloatLe,
                        BinOp::Gt => mir::BinOp::FloatGt,
                        BinOp::Ge => mir::BinOp::FloatGe,
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
            thir::Expr::Local(_)
            | thir::Expr::Literal(_)
            | thir::Expr::Function(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Assign(_)
            | thir::Expr::Return(_)
            | thir::Expr::Break(_)
            | thir::Expr::Block(_)
            | thir::Expr::If(_)
            | thir::Expr::Loop(_) => {
                let operand = self.as_operand(expr);
                mir::Value::Use(operand)
            }
        }
    }
}
