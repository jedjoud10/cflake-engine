use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::BuildHasherDefault,
    ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr},
};

use nohash_hasher::{IsEnabled, NoHashHasher};

// A simple mask
#[derive(Default, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Mask(pub(crate) u64);
impl IsEnabled for Mask {}

impl Mask {
    pub const fn one() -> Mask {
        Mask(1)
    }
    pub const fn zero() -> Mask {
        Mask(0)
    }

    pub const fn all() -> Mask {
        Mask(u64::MAX)
    }

    pub const fn from_offset(offset: usize) -> Mask {
        Mask(1 << offset)
    }

    pub const fn offset(&self) -> usize {
        self.0.trailing_zeros() as usize
    }
}

// NoHash hasher that works with Mask
type MaskHasher = BuildHasherDefault<NoHashHasher<Mask>>;
pub type MaskMap<E> = HashMap<Mask, E, MaskHasher>;
pub type MaskSet = HashSet<Mask, MaskHasher>;

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
