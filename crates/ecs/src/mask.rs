use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::BuildHasherDefault,
    ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr},
};

use nohash_hasher::{IsEnabled, NoHashHasher};

// A simple mask
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Mask(pub(crate) u64);
impl IsEnabled for Mask {}

impl Mask {
    // 0, 1
    pub const fn one() -> Mask {
        Mask(1)
    }
    pub const fn zero() -> Mask {
        Mask(0)
    }

    // All bits set
    pub const fn all() -> Mask {
        Mask(u64::MAX)
    }

    // Calculate mask from left shift bit offset
    pub const fn from_offset(offset: usize) -> Mask {
        Mask(1 << offset)
    }
    pub const fn offset(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    // Check if a mask is empty
    pub fn empty(&self) -> bool {
        *self == Self::zero()
    }

    // Count
    pub const fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }
    pub const fn count_zeros(&self) -> u32 {
        self.0.count_zeros()
    }

    // Set a single bit to either true or false
    pub fn set(&mut self, offset: usize, enabled: bool) {
        *self = if enabled {
            // Or
            *self | (Mask::one() << offset)
        } else {
            // Negate and And
            *self & !(Mask::one() << offset)
        }
    }

    // Get a specific bit using an offset
    pub const fn get(&self, offset: usize) -> bool {
        (self.0 >> offset) & 1 == 1
    }

    // Check if we have at least one corresponding bit with Other
    pub fn one_corresponding_bit(&self, other: Self) -> bool {
        *self & other != Mask::zero()
    }

    // Check if all the bits from Other are present within Self
    // other: 0100
    // self:  1111
    // true
    pub fn contains(&self, other: Self) -> bool {
        *self & other == other
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
