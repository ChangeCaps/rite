use ritec_core::{BinOp, UnaryOp};
use ritec_mir as mir;

use crate::{thir, unpack, BlockAnd, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn as_value(&mut self, mut block: mir::BlockId, expr: &thir::Expr) -> BlockAnd<mir::Value> {
        match expr {
            thir::Expr::As(expr) => {
                let value = unpack!(block = self.as_operand(block, &self.thir[expr.expr]));
                let from = self.thir[expr.expr].ty();
                let to = expr.ty.clone();

                let intrinsic = match (from, &to) {
                    (_, _) if *from == to => return BlockAnd::new(block, mir::Value::Use(value)),
                    (mir::Type::Pointer(from), mir::Type::Pointer(to)) => {
                        mir::Intrinsic::PtrToPtr(value, from.clone(), to.clone())
                    }
                    (mir::Type::Pointer(from), mir::Type::Int(to)) => {
                        mir::Intrinsic::PtrToInt(value, from.clone(), to.clone())
                    }
                    (mir::Type::Int(from), mir::Type::Pointer(to)) => {
                        mir::Intrinsic::IntToPtr(value, from.clone(), to.clone())
                    }
                    (mir::Type::Int(from), mir::Type::Int(to)) => {
                        mir::Intrinsic::IntToInt(value, from.clone(), to.clone())
                    }
                    (mir::Type::Float(from), mir::Type::Float(to)) => {
                        mir::Intrinsic::FloatToFloat(value, from.clone(), to.clone())
                    }
                    (mir::Type::Float(from), mir::Type::Int(to)) => {
                        mir::Intrinsic::FloatToInt(value, from.clone(), to.clone())
                    }
                    (mir::Type::Int(from), mir::Type::Float(to)) => {
                        mir::Intrinsic::IntToFloat(value, from.clone(), to.clone())
                    }
                    _ => unreachable!(),
                };

                let value = mir::Value::Intrinsic(intrinsic);
                BlockAnd::new(block, value)
            }
            thir::Expr::Bitcast(expr) => {
                let value = unpack!(block = self.as_operand(block, &self.thir[expr.expr]));
                let bitcast = mir::Intrinsic::Bitcast(value, expr.ty.clone());
                BlockAnd::new(block, mir::Value::Intrinsic(bitcast))
            }
            thir::Expr::Sizeof(expr) => {
                let size = mir::Intrinsic::Sizeof(expr.ty.clone());
                BlockAnd::new(block, mir::Value::Intrinsic(size))
            }
            thir::Expr::Alignof(expr) => {
                let align = mir::Intrinsic::Alignof(expr.ty.clone());
                BlockAnd::new(block, mir::Value::Intrinsic(align))
            }
            thir::Expr::Malloc(expr) => {
                let count = unpack!(block = self.as_operand(block, &self.thir[expr.count]));
                let malloc = mir::Intrinsic::Malloc(count, expr.item.clone());
                BlockAnd::new(block, mir::Value::Intrinsic(malloc))
            }
            thir::Expr::Free(expr) => {
                let ptr = unpack!(block = self.as_operand(block, &self.thir[expr.expr]));
                let free = mir::Intrinsic::Free(ptr);
                BlockAnd::new(block, mir::Value::Intrinsic(free))
            }
            thir::Expr::Memcpy(expr) => {
                let dst = unpack!(block = self.as_operand(block, &self.thir[expr.dst]));
                let src = unpack!(block = self.as_operand(block, &self.thir[expr.src]));
                let size = unpack!(block = self.as_operand(block, &self.thir[expr.size]));
                let memcpy = mir::Intrinsic::Memcpy(dst, src, size);
                BlockAnd::new(block, mir::Value::Intrinsic(memcpy))
            }
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Ref => {
                let place = unpack!(block = self.as_place(block, &self.thir[expr.operand]));
                BlockAnd::new(block, mir::Value::Address(place))
            }
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Neg => {
                let op = match expr.ty {
                    mir::Type::Int(_) => mir::UnaryOp::IntNeg,
                    mir::Type::Float(_) => mir::UnaryOp::FloatNeg,
                    _ => unreachable!("{}", expr.ty),
                };

                let value = unpack!(block = self.as_operand(block, &self.thir[expr.operand]));
                BlockAnd::new(block, mir::Value::UnaryOp(op, value))
            }
            thir::Expr::Unary(expr) if expr.operator == UnaryOp::Not => {
                let value = unpack!(block = self.as_operand(block, &self.thir[expr.operand]));
                BlockAnd::new(block, mir::Value::UnaryOp(mir::UnaryOp::IntNot, value))
            }
            thir::Expr::Binary(expr) => {
                let lhs = unpack!(block = self.as_operand(block, &self.thir[expr.lhs]));
                let rhs = unpack!(block = self.as_operand(block, &self.thir[expr.rhs]));

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

                BlockAnd::new(block, mir::Value::BinaryOp(op, lhs, rhs))
            }
            thir::Expr::Call(expr) => {
                let callee = unpack!(block = self.as_operand(block, &self.thir[expr.callee]));
                let mut arguments = Vec::new();
                for &argument in &expr.arguments {
                    let argument = unpack!(block = self.as_operand(block, &self.thir[argument]));
                    arguments.push(argument);
                }

                BlockAnd::new(block, mir::Value::Call(callee, arguments))
            }
            thir::Expr::StaticCall(expr) => {
                let mut arguments = Vec::new();
                for &argument in &expr.arguments {
                    let argument = unpack!(block = self.as_operand(block, &self.thir[argument]));
                    arguments.push(argument);
                }

                let callee = mir::Operand::Constant(mir::Constant::Function(
                    expr.callee.cast(),
                    expr.generics.clone(),
                ));
                BlockAnd::new(block, mir::Value::Call(callee, arguments))
            }
            thir::Expr::Local(_)
            | thir::Expr::Literal(_)
            | thir::Expr::Function(_)
            | thir::Expr::ClassInit(_)
            | thir::Expr::Field(_)
            | thir::Expr::Unary(_)
            | thir::Expr::Assign(_)
            | thir::Expr::Return(_)
            | thir::Expr::Break(_)
            | thir::Expr::Block(_)
            | thir::Expr::If(_)
            | thir::Expr::Loop(_) => {
                let operand = unpack!(block = self.as_operand(block, expr));
                BlockAnd::new(block, mir::Value::Use(operand))
            }
        }
    }
}
