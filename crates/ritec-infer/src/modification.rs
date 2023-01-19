use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
pub enum Modification {
    Ref,
    Deref,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Modifications {
    pub modifications: Vec<Modification>,
}

impl Modifications {
    pub const fn new() -> Self {
        Self {
            modifications: Vec::new(),
        }
    }

    pub fn push(&mut self, modification: Modification) {
        if self.last() == Some(&Modification::Deref) && modification == Modification::Ref {
            self.pop();

            return;
        }

        if self.last() == Some(&Modification::Ref) && modification == Modification::Deref {
            self.pop();

            return;
        }

        self.modifications.push(modification);
    }
}

impl Deref for Modifications {
    type Target = Vec<Modification>;

    fn deref(&self) -> &Self::Target {
        &self.modifications
    }
}

impl DerefMut for Modifications {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modifications
    }
}
