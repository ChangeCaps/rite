use std::ops::Index;

use ritec_core::Generic;

use crate::InferType;

pub struct Instance {
    pub generics: Vec<Generic>,
    pub types: Vec<InferType>,
}

impl Instance {
    pub const fn empty() -> Self {
        Self {
            generics: Vec::new(),
            types: Vec::new(),
        }
    }

    pub fn new(generics: Vec<Generic>, types: Vec<InferType>) -> Self {
        Self { generics, types }
    }

    pub fn get(&self, generic: &Generic) -> Option<&InferType> {
        let index = self.generics.iter().position(|g| g == generic)?;
        self.types.get(index)
    }
}

impl Index<&Generic> for Instance {
    type Output = InferType;

    fn index(&self, index: &Generic) -> &Self::Output {
        self.get(index).unwrap()
    }
}
