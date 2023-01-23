use std::collections::HashMap;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    types::{BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, PointerType},
    values::{BasicValueEnum, CallableValue, FunctionValue, PointerValue},
    AddressSpace, FloatPredicate, IntPredicate,
};
use mir::GenericMap;
use ritec_core::FloatSize;
use ritec_mir as mir;

use crate::CodegenCx;

pub struct FunctionBuilder<'a, 'c> {
    pub cx: &'a mut CodegenCx<'c>,
    pub builder: Builder<'c>,
    pub function: mir::FunctionId,
    pub fn_value: Option<FunctionValue<'c>>,
    pub generics: Vec<mir::Type>,
    pub locals: HashMap<mir::LocalId, PointerValue<'c>>,
    pub blocks: HashMap<mir::BlockId, BasicBlock<'c>>,
}

impl<'a, 'c> FunctionBuilder<'a, 'c> {
    pub fn new(
        cx: &'a mut CodegenCx<'c>,
        function: mir::FunctionId,
        generics: Vec<mir::Type>,
    ) -> Self {
        let builder = cx.context.create_builder();

        Self {
            cx,
            builder,
            function,
            fn_value: None,
            generics,
            locals: HashMap::new(),
            blocks: HashMap::new(),
        }
    }

    pub fn cx(&self) -> &'c Context {
        self.cx.context
    }

    pub fn void_type(&self) -> BasicTypeEnum<'c> {
        self.cx().struct_type(&[], false).into()
    }

    pub fn void_value(&self) -> BasicValueEnum<'c> {
        self.void_type().const_zero()
    }

    pub fn usize_type(&self) -> IntType<'c> {
        self.cx().ptr_sized_int_type(&self.cx.target_data(), None)
    }

    pub fn int_type(&self, ty: mir::IntType) -> IntType<'c> {
        if let Some(size) = ty.size {
            (self.cx().custom_width_int_type(size.bit_width() as u32)).into()
        } else {
            self.usize_type().into()
        }
    }

    pub fn float_type(&self, ty: mir::FloatType) -> FloatType<'c> {
        match ty.size {
            FloatSize::F16 => self.cx().f16_type(),
            FloatSize::F32 => self.cx().f32_type(),
            FloatSize::F64 => self.cx().f64_type(),
        }
    }

    pub fn pointer_type(&self, ty: &mir::PointerType) -> PointerType<'c> {
        let ty = self.build_type(ty.pointee());
        ty.ptr_type(AddressSpace::Generic).into()
    }

    pub fn build_function_type(&self, ty: &mir::FunctionType) -> FunctionType<'c> {
        let return_type = self.build_type(&ty.return_type);

        let mut arguments = Vec::new();
        for argument in &ty.arguments {
            arguments.push(self.build_type(argument).into());
        }

        return_type.fn_type(&arguments, false)
    }

    pub fn function(&self) -> &'a mir::Function {
        &self.cx.program[self.function]
    }

    pub fn build_type(&self, ty: &mir::Type) -> BasicTypeEnum<'c> {
        match ty {
            mir::Type::Void => self.cx().struct_type(&[], false).into(),
            mir::Type::Bool => self.cx().bool_type().into(),
            mir::Type::Int(ty) => self.int_type(ty.clone()).into(),
            mir::Type::Float(ty) => self.float_type(ty.clone()).into(),
            mir::Type::Pointer(ty) => self.pointer_type(ty).into(),
            mir::Type::Array(ty) => self
                .build_type(&ty.element)
                .array_type(ty.size as u32)
                .into(),
            mir::Type::Slice(_) => todo!(),
            mir::Type::Function(ty) => self
                .build_function_type(ty)
                .ptr_type(AddressSpace::Generic)
                .into(),
            mir::Type::Tuple(ty) => {
                let mut fields = Vec::new();
                for field in &ty.fields {
                    fields.push(self.build_type(field));
                }

                self.cx().struct_type(&fields, false).into()
            }
            mir::Type::Class(ty) => {
                let class = &self.cx.program[ty.class];

                let generic_map = GenericMap::new(&class.generics, &ty.generics);

                let mut fields = Vec::new();
                for field in &class.fields {
                    let mut field_type = field.ty.clone();
                    field_type.instantiate(&generic_map);

                    fields.push(self.build_type(&field_type));
                }

                self.cx().struct_type(&fields, false).into()
            }
            mir::Type::Generic(generic) => {
                for (i, fn_generic) in self.function().generics.iter().enumerate() {
                    if fn_generic == generic {
                        return self.build_type(&self.generics[i]);
                    }
                }

                unreachable!("generic type not found {:?}", generic)
            }
        }
    }

    pub fn build(&mut self) -> FunctionValue<'c> {
        // add function
        let function = &self.cx.program[self.function];
        let generics: Vec<_> = self.generics.iter().map(mir::Type::to_string).collect();
        let function_name = if !generics.is_empty() {
            format!("{}<{}>", function.ident, generics.join(", "))
        } else {
            format!("{}", function.ident)
        };

        let fn_type = self.build_function_type(&function.ty());
        let fn_value = self.cx.module.add_function(&function_name, fn_type, None);
        self.fn_value = Some(fn_value);

        let instance = (self.function, self.generics.clone());
        self.cx.functions.insert(instance, fn_value);

        // create entry block
        let block = self.cx.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(block);

        // allocate locals on the stack
        for (local_id, local) in function.body.locals.iter() {
            let ty = self.build_type(&local.ty);
            let name = format!("_{}", local_id.as_raw_index());
            let value = self.builder.build_alloca(ty, &name);
            self.locals.insert(local_id, value);
        }

        // store arguments in locals
        for (i, argument) in function.arguments.iter().enumerate() {
            let local = self.locals[&argument.local];
            let value = fn_value.get_nth_param(i as u32).unwrap();
            self.builder.build_store(local, value);
        }

        // allocate blocks
        for block_id in function.body.blocks.keys() {
            let name = format!("bb{}", block_id.as_raw_index());
            let block = self.cx().append_basic_block(fn_value, &name);
            self.blocks.insert(block_id, block);
        }

        // jump to the first block
        let first_block = function.body.blocks.keys().next().unwrap();
        let first_block = self.blocks[&first_block];
        self.builder.build_unconditional_branch(first_block);

        // build blocks
        for (block_id, block) in function.body.blocks.iter() {
            self.builder.position_at_end(self.blocks[&block_id]);
            self.build_block(block);
        }

        fn_value
    }

    pub fn build_block(&mut self, block: &mir::Block) {
        for statement in &block.statements {
            self.build_statement(statement);
        }

        if let Some(ref term) = block.terminator {
            self.build_terminator(term);
        }
    }

    pub fn build_statement(&mut self, statement: &mir::Statement) {
        match statement {
            mir::Statement::Assign(assign) => {
                let place = self.build_place(&assign.place);
                let value = self.build_value(&assign.value);
                self.builder.build_store(place, value);
            }
            mir::Statement::Drop(value) => {
                let _ = self.build_value(value);
            }
        }
    }

    pub fn build_place(&mut self, place: &mir::Place) -> PointerValue<'c> {
        let mut value = self.locals[&place.local];

        for proj in place.proj.iter() {
            value = self.build_projection(value, proj);
        }

        value
    }

    pub fn build_projection(
        &mut self,
        ptr: PointerValue<'c>,
        proj: &mir::Projection,
    ) -> PointerValue<'c> {
        match proj {
            mir::Projection::Deref => self.builder.build_load(ptr, "deref").into_pointer_value(),
            mir::Projection::Field(field) => {
                let index = field.as_raw_index() as u32;
                self.builder.build_struct_gep(ptr, index, "field").unwrap()
            }
        }
    }

    pub fn build_operand(&mut self, operand: &mir::Operand) -> BasicValueEnum<'c> {
        match operand {
            mir::Operand::Copy(place) => {
                let ptr = self.build_place(place);
                self.builder.build_load(ptr, "copy")
            }
            mir::Operand::Move(place) => {
                let ptr = self.build_place(place);
                self.builder.build_load(ptr, "move")
            }
            mir::Operand::Constant(constant) => self.build_constant(constant),
        }
    }

    pub fn build_constant(&mut self, constant: &mir::Constant) -> BasicValueEnum<'c> {
        match constant {
            mir::Constant::Void => self.void_value(),
            mir::Constant::Null(ty) => self
                .build_type(ty)
                .ptr_type(AddressSpace::Generic)
                .const_zero()
                .into(),
            mir::Constant::Function(id, generics) => {
                let generic_map = GenericMap::new(&self.function().generics, &self.generics);

                let mut resolved = Vec::new();
                for mut generic in generics.iter().cloned() {
                    generic.instantiate(&generic_map);
                    resolved.push(generic);
                }

                self.cx
                    .build_function(*id, &resolved)
                    .as_global_value()
                    .as_pointer_value()
                    .into()
            }
            mir::Constant::Integer(i, ty) => {
                let ty = match ty.size {
                    Some(size) => self.cx().custom_width_int_type(size.bit_width() as u32),
                    None => self.cx().ptr_sized_int_type(&self.cx.target_data(), None),
                };

                ty.const_int(*i as u64, false).into()
            }
            mir::Constant::Float(f, ty) => match ty.size {
                FloatSize::F16 => self.cx().f16_type().const_float(*f).into(),
                FloatSize::F32 => self.cx().f32_type().const_float(*f).into(),
                FloatSize::F64 => self.cx().f64_type().const_float(*f).into(),
            },
            mir::Constant::Bool(b) => self.cx().bool_type().const_int(*b as u64, false).into(),
        }
    }

    pub fn build_value(&mut self, value: &mir::Value) -> BasicValueEnum<'c> {
        match value {
            mir::Value::Use(operand) => self.build_operand(operand),
            mir::Value::Address(place) => self.build_place(place).into(),
            mir::Value::UnaryOp(op, value) => match op {
                mir::UnaryOp::IntNot => {
                    let value = self.build_operand(value).into_int_value();
                    self.builder.build_not(value, "int_not").into()
                }
                mir::UnaryOp::IntNeg => {
                    let value = self.build_operand(value).into_int_value();
                    self.builder.build_int_neg(value, "int_neg").into()
                }
                mir::UnaryOp::FloatNeg => {
                    let value = self.build_operand(value).into_float_value();
                    self.builder.build_float_neg(value, "float_neg").into()
                }
            },
            mir::Value::BinaryOp(op, lhs, rhs) => self.build_binary_op(*op, lhs, rhs),
            mir::Value::Call(callee, args) => self.build_call(callee, args),
            mir::Value::Intrinsic(intrinsic) => self.build_intrinsic(intrinsic),
        }
    }

    pub fn build_binary_op(
        &mut self,
        op: mir::BinOp,
        lhs: &mir::Operand,
        rhs: &mir::Operand,
    ) -> BasicValueEnum<'c> {
        let lhs = self.build_operand(&lhs);
        let rhs = self.build_operand(&rhs);

        match op {
            mir::BinOp::IntAdd => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder.build_int_add(lhs, rhs, "add").into()
            }
            mir::BinOp::IntSub => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder.build_int_sub(lhs, rhs, "sub").into()
            }
            mir::BinOp::IntMul => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder.build_int_mul(lhs, rhs, "mul").into()
            }
            mir::BinOp::IntDivSigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder.build_int_signed_div(lhs, rhs, "div").into()
            }
            mir::BinOp::IntDivUnsigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder.build_int_unsigned_div(lhs, rhs, "div").into()
            }
            mir::BinOp::IntEq => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq")
                    .into()
            }
            mir::BinOp::IntNe => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::NE, lhs, rhs, "ne")
                    .into()
            }
            mir::BinOp::IntLtSigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt")
                    .into()
            }
            mir::BinOp::IntLtUnsigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::ULT, lhs, rhs, "lt")
                    .into()
            }
            mir::BinOp::IntLeSigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::SLE, lhs, rhs, "le")
                    .into()
            }
            mir::BinOp::IntLeUnsigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::ULE, lhs, rhs, "le")
                    .into()
            }
            mir::BinOp::IntGtSigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt")
                    .into()
            }
            mir::BinOp::IntGtUnsigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::UGT, lhs, rhs, "gt")
                    .into()
            }
            mir::BinOp::IntGeSigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge")
                    .into()
            }
            mir::BinOp::IntGeUnsigned => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                self.builder
                    .build_int_compare(IntPredicate::UGE, lhs, rhs, "ge")
                    .into()
            }
            mir::BinOp::FloatAdd => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder.build_float_add(lhs, rhs, "add").into()
            }
            mir::BinOp::FloatSub => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder.build_float_sub(lhs, rhs, "sub").into()
            }
            mir::BinOp::FloatMul => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder.build_float_mul(lhs, rhs, "mul").into()
            }
            mir::BinOp::FloatDiv => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder.build_float_div(lhs, rhs, "div").into()
            }
            mir::BinOp::FloatEq => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder
                    .build_float_compare(FloatPredicate::OEQ, lhs, rhs, "eq")
                    .into()
            }
            mir::BinOp::FloatNe => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder
                    .build_float_compare(FloatPredicate::ONE, lhs, rhs, "ne")
                    .into()
            }
            mir::BinOp::FloatLt => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder
                    .build_float_compare(FloatPredicate::OLT, lhs, rhs, "lt")
                    .into()
            }
            mir::BinOp::FloatLe => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder
                    .build_float_compare(FloatPredicate::OLE, lhs, rhs, "le")
                    .into()
            }
            mir::BinOp::FloatGt => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder
                    .build_float_compare(FloatPredicate::OGT, lhs, rhs, "gt")
                    .into()
            }
            mir::BinOp::FloatGe => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                self.builder
                    .build_float_compare(FloatPredicate::OGE, lhs, rhs, "ge")
                    .into()
            }
        }
    }

    pub fn build_intrinsic(&mut self, intrinsic: &mir::Intrinsic) -> BasicValueEnum<'c> {
        match intrinsic {
            mir::Intrinsic::Sizeof(ty) => {
                let ty = self.build_type(ty);
                ty.size_of().unwrap().into()
            }
            mir::Intrinsic::Alignof(ty) => {
                let ty = self.build_type(ty);

                let align = match ty {
                    BasicTypeEnum::ArrayType(ty) => ty.get_alignment(),
                    BasicTypeEnum::FloatType(ty) => ty.get_alignment(),
                    BasicTypeEnum::IntType(ty) => ty.get_alignment(),
                    BasicTypeEnum::PointerType(ty) => ty.get_alignment(),
                    BasicTypeEnum::StructType(ty) => ty.get_alignment(),
                    BasicTypeEnum::VectorType(ty) => ty.get_alignment(),
                };

                align.into()
            }
            mir::Intrinsic::Bitcast(operand, ty) => {
                let operand = self.build_operand(operand);
                let ty = self.build_type(ty);
                self.builder.build_bitcast(operand, ty, "bitcast").into()
            }
            mir::Intrinsic::Malloc(size, ty) => {
                let size = self.build_operand(size);
                let ptr = self.builder.build_array_malloc(
                    self.build_type(ty),
                    size.into_int_value(),
                    "malloc",
                );
                ptr.unwrap().into()
            }
            mir::Intrinsic::Free(ptr) => {
                let ptr = self.build_operand(ptr);
                self.builder.build_free(ptr.into_pointer_value());
                self.void_value()
            }
            mir::Intrinsic::Memcpy(dst, src, size) => {
                let dst = self.build_operand(dst);
                let src = self.build_operand(src);
                let size = self.build_operand(size);
                let res = self.builder.build_memcpy(
                    dst.into_pointer_value(),
                    1,
                    src.into_pointer_value(),
                    1,
                    size.into_int_value(),
                );
                res.unwrap();
                self.void_value()
            }
            mir::Intrinsic::PtrToInt(ptr, _, to) => {
                let ptr = self.build_operand(ptr);
                let ptr = ptr.into_pointer_value();

                let to = self.int_type(to.clone());

                let ptr = self.builder.build_ptr_to_int(ptr, to, "ptrtoint");
                self.builder.build_int_cast(ptr, to, "intcast").into()
            }
            mir::Intrinsic::IntToPtr(int, _, to) => {
                let int = self.build_operand(int);
                let int = int.into_int_value();

                let to = self.pointer_type(to);

                self.builder.build_int_to_ptr(int, to, "inttoptr").into()
            }
            mir::Intrinsic::PtrToPtr(ptr, _, to) => {
                let ptr = self.build_operand(ptr);
                let ptr = ptr.into_pointer_value();

                let to = self.pointer_type(to);

                self.builder.build_pointer_cast(ptr, to, "ptrtoptr").into()
            }
            mir::Intrinsic::IntToInt(int, _, to) => {
                let int = self.build_operand(int);
                let int = int.into_int_value();

                let to = self.int_type(to.clone());

                self.builder.build_int_cast(int, to, "intcast").into()
            }
            mir::Intrinsic::FloatToFloat(float, _, to) => {
                let float = self.build_operand(float);
                let float = float.into_float_value();

                let to = self.float_type(to.clone());

                self.builder.build_float_cast(float, to, "floatcast").into()
            }
            mir::Intrinsic::FloatToInt(float, _, to) => {
                let float = self.build_operand(float);
                let float = float.into_float_value();

                let ty = self.int_type(to.clone());

                if to.signed {
                    self.builder
                        .build_float_to_signed_int(float, ty, "floattoint")
                        .into()
                } else {
                    self.builder
                        .build_float_to_unsigned_int(float, ty, "floattoint")
                        .into()
                }
            }
            mir::Intrinsic::IntToFloat(int, from, to) => {
                let int = self.build_operand(int);
                let int = int.into_int_value();

                let to = self.float_type(to.clone());

                if from.signed {
                    self.builder
                        .build_signed_int_to_float(int, to, "inttofloat")
                        .into()
                } else {
                    self.builder
                        .build_unsigned_int_to_float(int, to, "inttofloat")
                        .into()
                }
            }
        }
    }

    pub fn build_call(
        &mut self,
        callee: &mir::Operand,
        arguments: &[mir::Operand],
    ) -> BasicValueEnum<'c> {
        let callee = self.build_operand(callee);
        let mut args = Vec::new();
        for arg in arguments {
            args.push(self.build_operand(arg).into());
        }

        let callee: CallableValue = callee.into_pointer_value().try_into().unwrap();

        self.builder
            .build_call(callee, &args, "call")
            .try_as_basic_value()
            .left()
            .unwrap()
    }

    pub fn build_terminator(&mut self, terminator: &mir::Terminator) {
        match terminator {
            mir::Terminator::Goto(block) => {
                self.builder.build_unconditional_branch(self.blocks[block]);
            }
            mir::Terminator::Return(operand) => {
                let value = self.build_operand(operand);
                self.builder.build_return(Some(&value));
            }
            mir::Terminator::Switch(value, targets) => {
                let value = self.build_operand(value).into_int_value();

                let mut cases = Vec::new();
                for (case, target) in targets.targets.iter() {
                    let case = value.get_type().const_int(*case as u64, false);
                    cases.push((case, self.blocks[target]));
                }

                let default = self.blocks[&targets.default];
                self.builder.build_switch(value, default, &cases);
            }
        }
    }
}
