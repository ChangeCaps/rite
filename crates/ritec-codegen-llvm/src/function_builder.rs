use std::collections::HashMap;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    types::{BasicType, BasicTypeEnum, FunctionType},
    values::{BasicValueEnum, CallableValue, FunctionValue, PointerValue},
    AddressSpace,
};
use ritec_core::{FloatSize, Generic};
use ritec_mir as mir;

use crate::CodegenCx;

pub struct FunctionBuilder<'a, 'c> {
    pub cx: &'a mut CodegenCx<'c>,
    pub builder: Builder<'c>,
    pub function: &'a mir::Function,
    pub fn_value: Option<FunctionValue<'c>>,
    pub generics: HashMap<Generic, mir::Type>,
    pub locals: HashMap<mir::LocalId, PointerValue<'c>>,
    pub blocks: HashMap<mir::BlockId, BasicBlock<'c>>,
}

impl<'a, 'c> FunctionBuilder<'a, 'c> {
    pub fn new(
        cx: &'a mut CodegenCx<'c>,
        function: &'a mir::Function,
        generics: &[mir::Type],
    ) -> Self {
        let builder = cx.context.create_builder();

        let mut generic_map = HashMap::new();
        for (generic, ty) in function.generics.iter().zip(generics) {
            generic_map.insert(generic.clone(), ty.clone());
        }

        Self {
            cx,
            builder,
            function,
            fn_value: None,
            generics: generic_map,
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

    pub fn build_function_type(&self, ty: &mir::FunctionType) -> FunctionType<'c> {
        let return_type = self.build_type(&ty.return_type);

        let mut arguments = Vec::new();
        for argument in &ty.arguments {
            arguments.push(self.build_type(argument).into());
        }

        return_type.fn_type(&arguments, true)
    }

    pub fn build_type(&self, ty: &mir::Type) -> BasicTypeEnum<'c> {
        match ty {
            mir::Type::Void => self.cx().struct_type(&[], false).into(),
            mir::Type::Bool => self.cx().i8_type().into(),
            mir::Type::Int(ty) => {
                if let Some(size) = ty.size {
                    (self.cx().custom_width_int_type(size.bit_width() as u32)).into()
                } else {
                    (self.cx().ptr_sized_int_type(&self.cx.target_data(), None)).into()
                }
            }
            mir::Type::Float(ty) => match ty.size {
                FloatSize::F16 => self.cx().f16_type().into(),
                FloatSize::F32 => self.cx().f32_type().into(),
                FloatSize::F64 => self.cx().f64_type().into(),
            },
            mir::Type::Pointer(ty) => self
                .build_type(&ty.pointee)
                .ptr_type(AddressSpace::Generic)
                .into(),
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
            mir::Type::Generic(generic) => {
                let value = self.generics.get(generic).unwrap();
                self.build_type(value)
            }
        }
    }

    pub fn build(&mut self) -> FunctionValue<'c> {
        // add function
        let function_name = format!("{}", self.function.ident);

        let fn_type = self.build_function_type(&self.function.ty());
        let fn_value = self.cx.module.add_function(&function_name, fn_type, None);
        self.fn_value = Some(fn_value);

        // create entry block
        let block = self.cx.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(block);

        // allocate locals on the stack
        for (local_id, local) in self.function.body.locals.iter() {
            let ty = self.build_type(&local.ty);
            let name = format!("_{}", local_id.as_raw_index());
            let value = self.builder.build_alloca(ty, &name);
            self.locals.insert(local_id, value);
        }

        // store arguments in locals
        for (i, argument) in self.function.arguments.iter().enumerate() {
            let local = self.locals[&argument.local];
            let value = fn_value.get_nth_param(i as u32).unwrap();
            self.builder.build_store(local, value);
        }

        // allocate blocks
        for block_id in self.function.body.blocks.keys() {
            let name = format!("bb{}", block_id.as_raw_index());
            let block = self.cx().append_basic_block(fn_value, &name);
            self.blocks.insert(block_id, block);
        }

        // jump to the first block
        let first_block = self.function.body.blocks.keys().next().unwrap();
        let first_block = self.blocks[&first_block];
        self.builder.build_unconditional_branch(first_block);

        // build blocks
        for (block_id, block) in self.function.body.blocks.iter() {
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
            mir::Constant::Function(id, generics) => {
                let mut resolved_generics = Vec::new();
                for generic in generics {
                    resolved_generics.push(generic.clone());
                }

                self.cx
                    .build_function(*id, &generics)
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
        }
    }

    pub fn build_value(&mut self, value: &mir::Value) -> BasicValueEnum<'c> {
        match value {
            mir::Value::Use(operand) => self.build_operand(operand),
            mir::Value::Address(place) => self.build_place(place).into(),
            mir::Value::BinaryOp(op, lhs, rhs) => self.build_binary_op(*op, lhs, rhs),
            mir::Value::Call(callee, args) => self.build_call(callee, args),
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
            mir::Terminator::Return(operand) => {
                let value = self.build_operand(operand);
                self.builder.build_return(Some(&value));
            }
        }
    }
}
