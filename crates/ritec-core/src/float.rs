#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatSize {
    F16 = 2,
    F32 = 4,
    F64 = 8,
}

impl FloatSize {
    pub const fn from_byte_size(byte_size: usize) -> Option<Self> {
        match byte_size {
            2 => Some(FloatSize::F16),
            4 => Some(FloatSize::F32),
            8 => Some(FloatSize::F64),
            _ => None,
        }
    }

    pub const fn from_bit_width(bit_width: usize) -> Option<Self> {
        match bit_width {
            16 => Some(FloatSize::F16),
            32 => Some(FloatSize::F32),
            64 => Some(FloatSize::F64),
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
