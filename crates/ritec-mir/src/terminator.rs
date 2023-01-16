use std::fmt::{self, Display};

use crate::{BlockId, Operand};

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchTargets {
    pub targets: Vec<(u64, BlockId)>,
    pub default: BlockId,
}

impl SwitchTargets {
    pub fn successors(&self) -> impl Iterator<Item = &BlockId> {
        let mut sucessors = Vec::with_capacity(self.targets.len() + 1);

        for (_, target) in &self.targets {
            sucessors.push(target);
        }

        sucessors.push(&self.default);

        sucessors.into_iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Terminator {
    Goto(BlockId),
    Return(Operand),
    Switch(Operand, SwitchTargets),
}

impl Terminator {
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            Self::Return(_) => Vec::new(),
            _ => Vec::new(),
        }
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Goto(target) => write!(f, "goto bb{}", target.as_raw_index()),
            Self::Return(operand) => write!(f, "return {}", operand),
            Self::Switch(operand, switch_targets) => {
                let targets: Vec<_> = switch_targets
                    .targets
                    .iter()
                    .map(|(operand, block)| format!("{} -> bb{}", operand, block.as_raw_index()))
                    .collect();

                write!(
                    f,
                    "switch ({}) [{}], default -> bb{}",
                    operand,
                    targets.join(", "),
                    switch_targets.default.as_raw_index()
                )
            }
        }
    }
}
