#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntSize {
    I8 = 1,
    I16 = 2,
    I32 = 4,
    I64 = 8,
    I128 = 16,
}

impl IntSize {
    pub const fn from_byte_size(byte_size: usize) -> Option<Self> {
        match byte_size {
            1 => Some(IntSize::I8),
            2 => Some(IntSize::I16),
            4 => Some(IntSize::I32),
            8 => Some(IntSize::I64),
            16 => Some(IntSize::I128),
            _ => None,
        }
    }

    pub const fn from_bit_width(bit_width: usize) -> Option<Self> {
        match bit_width {
            8 => Some(IntSize::I8),
            16 => Some(IntSize::I16),
            32 => Some(IntSize::I32),
            64 => Some(IntSize::I64),
            128 => Some(IntSize::I128),
            _ => None,
        }
    }

    pub const fn byte_size(self) -> usize {
        self as usize
    }

    pub const fn bit_width(self) -> usize {
        self.byte_size() * 8
    }
}
