use std::collections::HashMap;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    types::{BasicType, BasicTypeEnum, FunctionType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use ritec_core::{FloatSize, Generic};
use ritec_mir as mir;

use crate::CodegenCx;

pub struct FunctionBuilder<'a, 'c> {
    pub cx: &'a CodegenCx<'c>,
    pub builder: Builder<'c>,
    pub function: &'a mir::Function,
    pub fn_value: Option<FunctionValue<'c>>,
    pub generics: HashMap<Generic, mir::Type>,
    pub locals: HashMap<mir::LocalId, PointerValue<'c>>,
    pub blocks: HashMap<mir::BlockId, BasicBlock<'c>>,
}

impl<'a, 'c> FunctionBuilder<'a, 'c> {
    pub fn new(cx: &'a CodegenCx<'c>, function: &'a mir::Function) -> Self {
        Self {
            cx,
            builder: cx.context.create_builder(),
            function,
            fn_value: None,
            generics: HashMap::new(),
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
            mir::Type::Bool => self.cx().bool_type().into(),
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

    pub fn build(&mut self) {
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
            mir::Operand::Void => self.void_value(),
        }
    }

    pub fn build_value(&mut self, value: &mir::Value) -> BasicValueEnum<'c> {
        match value {
            mir::Value::Use(operand) => self.build_operand(operand),
            mir::Value::Address(place) => self.build_place(place).into(),
        }
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
