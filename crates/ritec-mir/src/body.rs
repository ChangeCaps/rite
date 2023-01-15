use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use ritec_core::Arena;

use crate::{Block, BlockId, Local, LocalId, Operand, Place, Projection, Type, Value};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    pub locals: Arena<Local>,
    pub blocks: Arena<Block>,
}

impl Body {
    pub const fn new() -> Self {
        Self {
            locals: Arena::new(),
            blocks: Arena::new(),
        }
    }

    pub fn get_value_type(&self, value: &Value) -> Type {
        match value {
            Value::Use(operand) => self.get_operand_type(operand),
            Value::Address(place) => Type::pointer(self.get_place_type(place)),
            Value::BinaryOp(value) => match value.op {
                _ => self.get_operand_type(&value.lhs),
            },
        }
    }

    pub fn get_operand_type(&self, operand: &Operand) -> Type {
        match operand {
            Operand::Copy(place) => self.get_place_type(place),
            Operand::Move(place) => self.get_place_type(place),
            Operand::Constant(constant) => constant.ty(),
        }
    }

    pub fn get_place_type(&self, place: &Place) -> Type {
        let mut ty = self.locals[place.local].ty.clone();

        for proj in place.proj.iter() {
            ty = proj.apply_type(ty);
        }

        ty
    }
}

impl Index<LocalId> for Body {
    type Output = Local;

    fn index(&self, index: LocalId) -> &Self::Output {
        &self.locals[index]
    }
}

impl IndexMut<LocalId> for Body {
    fn index_mut(&mut self, index: LocalId) -> &mut Self::Output {
        &mut self.locals[index]
    }
}

impl Index<BlockId> for Body {
    type Output = Block;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index]
    }
}

impl IndexMut<BlockId> for Body {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, local) in self.locals.iter() {
            writeln!(
                f,
                "\tlet _{}: {}; {}",
                id.as_raw_index(),
                local.ty,
                local.comment()
            )?;
        }

        for (id, block) in self.blocks.iter() {
            writeln!(f)?;
            write!(f, "\tbb{}: {}", id.as_raw_index(), block)?;
        }

        Ok(())
    }
}
