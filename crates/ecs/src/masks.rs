use std::{
    fmt::{Debug, Display},
    ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr},
};

use nohash_hasher::IsEnabled;

// A simple mask
#[derive(Default, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Mask(pub(crate) u64);
impl IsEnabled for Mask {}

impl Mask {
    // One and zero masks
    pub fn one() -> Mask {
        Mask(1)
    }
    pub fn zero() -> Mask {
        Mask(0)
    }

    // All
    pub fn all() -> Mask {
        Mask(u64::MAX)
    }
}

impl BitAnd for Mask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Mask(self.0 & rhs.0)
    }
}

impl BitOr for Mask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Mask(self.0 | rhs.0)
    }
}

impl BitXor for Mask {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Mask(self.0 ^ rhs.0)
    }
}

impl Not for Mask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Mask(!self.0)
    }
}

impl Shl<usize> for Mask {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Mask(self.0 << rhs)
    }
}

impl Shr<usize> for Mask {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Mask(self.0 >> rhs)
    }
}

impl Display for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m{:b}", self.0)
    }
}

impl Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m{:b}", self.0)
    }
}
